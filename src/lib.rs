use std::collections::HashMap;

use schemars::generate::SchemaSettings;
use schemars::transform::AddNullable;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::Serialize;
use serde_json::{Map, Value};

// re-export
pub use schemars;

// Change the alias to use `Box<dyn error::Error>`.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
struct OpenApiPlaceHolderError(String);

impl std::fmt::Display for OpenApiPlaceHolderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OpenApiPlaceHolderError: {}", self.0)
    }
}

impl std::error::Error for OpenApiPlaceHolderError {}

impl OpenApiPlaceHolderError {
    pub fn new(message: &str) -> Self {
        OpenApiPlaceHolderError(message.to_string())
    }
}

pub struct OpenApiPlaceHolderBuilder {
    pub schema_settings: SchemaSettings,
}

pub struct OpenApiPlaceHolder {
    ph: HashMap<String, String>,
    definitions: Map<String, Value>,
    schema_generator: SchemaGenerator,
}

impl Default for OpenApiPlaceHolderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenApiPlaceHolderBuilder {
    pub fn new() -> Self {
        Self {
            // Same semantics as the pre-1.0 `option_nullable` / `option_add_null_type`
            // settings: annotations like `nullable: true` instead of null type arrays,
            // while keeping draft-07 `$ref` paths.
            schema_settings: SchemaSettings::draft07().with_transform(AddNullable::default()),
        }
    }

    pub fn with_schema_settings(self, schema_settings: SchemaSettings) -> Self {
        Self { schema_settings }
    }

    pub fn build(self) -> OpenApiPlaceHolder {
        OpenApiPlaceHolder {
            ph: HashMap::new(),
            definitions: Map::new(),
            schema_generator: self.schema_settings.into_generator(),
        }
    }
}

impl OpenApiPlaceHolder {
    pub fn builder() -> OpenApiPlaceHolderBuilder {
        OpenApiPlaceHolderBuilder::new()
    }

    pub fn new() -> Self {
        Self::builder().build()
    }

    fn describe_struct<T: JsonSchema>(&mut self) -> Schema {
        let generator = self.schema_generator.clone();

        let definitions_path = generator
            .settings()
            .definitions_path
            .trim_start_matches('#')
            .trim_end_matches('/')
            .to_owned();

        let mut schema = generator.into_root_schema_for::<T>();

        // move the generated definitions from the root schema
        // to the shared definitions list
        if let Some(definitions) = Self::take_definitions_at(&mut schema, &definitions_path) {
            for (name, definition) in definitions {
                self.definitions.insert(name, definition);
            }
        }

        // the old RootSchema meta_schema was never rendered
        schema.remove("$schema");

        schema
    }

    fn to_yaml(schema: &impl Serialize) -> Result<String> {
        Ok(serde_saphyr::to_string(schema)?
            .replace("---\n", "")
            .trim()
            .to_owned())
    }

    pub fn substitute(mut self, name: &str, value: &str) -> Self {
        self.ph.insert(name.to_owned(), value.to_owned());
        self
    }

    pub fn query_params<T: JsonSchema>(mut self, name: &str) -> Result<Self> {
        let mut root_schema = self.describe_struct::<T>();

        let Some(Value::Object(properties)) = root_schema.remove("properties") else {
            return Err(Box::new(OpenApiPlaceHolderError::new(
                "Only object type supported",
            )));
        };

        let mut result: Vec<String> = Vec::new();

        for (property_name, schema) in properties {
            let (required, description) = match &schema {
                Value::Object(o) => (
                    if o.get("nullable").is_some() {
                        "false"
                    } else {
                        "true"
                    },
                    o.get("description").and_then(|v| v.as_str()),
                ),
                _ => ("false", None),
            };

            let description_entry = if let Some(desc) = description {
                // quoted, so multi-line or special characters keep the YAML valid
                format!("  description: {}\n", serde_json::to_string(desc.trim())?)
            } else {
                "".to_string()
            };

            // do not include the description in the schema block,
            // it is written right above the schema entry
            let mut schema_block = schema;
            if let Value::Object(properties) = &mut schema_block {
                properties.remove("description");
            }

            result.push(format!(
                "- in: query\n  name: {name}\n{description}  required: {required}\n  schema:\n    {schema}",
                name=property_name,
                required=required,
                description=description_entry,
                schema=Self::with_indent("    ", &Self::to_yaml(&schema_block)?),
            ))
        }

        self.ph.insert(name.to_owned(), result.join("\n"));

        Ok(self)
    }

    // take a map out of the schema by a slash-separated path,
    // for example the `definitions` or `$defs` schema property
    fn take_definitions_at(schema: &mut Schema, path: &str) -> Option<Map<String, Value>> {
        let mut keys: Vec<String> = path
            .trim_matches('/')
            .split('/')
            .filter(|x| !x.is_empty())
            .map(|x| x.replace("~1", "/").replace("~0", "~"))
            .collect();
        let last = keys.pop()?;

        let mut object = schema.as_object_mut()?;
        for key in keys {
            object = object.get_mut(&key)?.as_object_mut()?;
        }

        match object.remove(&last)? {
            Value::Object(definitions) => Some(definitions),
            _ => None,
        }
    }

    pub fn schema<T: JsonSchema>(mut self, name: &str) -> Result<Self> {
        let root_schema = self.describe_struct::<T>();

        self.ph
            .insert(name.to_owned(), Self::to_yaml(&root_schema)?);

        Ok(self)
    }

    fn with_indent(indent: &str, value: &str) -> String {
        value
            .split('\n')
            .enumerate()
            .map(|(index, line)| {
                // don't indent first line
                if index == 0 {
                    return line.to_owned();
                }
                format!("{}{}", indent, line)
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn render_to(mut self, template: &str) -> Result<String> {
        let mut result = template.to_owned();
        self.ph.insert(
            "oaph::definitions".to_string(),
            if !self.definitions.is_empty() {
                Self::to_yaml(&self.definitions)?
            } else {
                "".to_owned()
            },
        );
        for (k, v) in self.ph.iter() {
            // split to lines
            result = result
                .split('\n')
                .map(|line| {
                    // find placeholder on each line
                    let pattern = format!("{{{{{}}}}}", k);
                    if let Some(position) = line.find(&pattern) {
                        // calc indent
                        let (indent, _) = line.split_at(position);

                        line.replace(&pattern, &Self::with_indent(indent, v))
                            .trim_end()
                            .to_owned()
                    } else {
                        line.to_owned()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
        }
        Ok(result)
    }

    pub fn swagger_ui_html(openapi_yaml_url: &str, title: &str) -> String {
        include_str!("../misc/swagger-ui.html")
            .replace("{{openapi_yaml_url}}", openapi_yaml_url)
            .replace("{{title}}", title)
    }

    pub fn redoc_ui_html(openapi_yaml_url: &str, title: &str) -> String {
        include_str!("../misc/redoc-ui.html")
            .replace("{{openapi_yaml_url}}", openapi_yaml_url)
            .replace("{{title}}", title)
    }

    pub fn render_to_file<P: AsRef<std::path::Path>>(self, template: &str, path: P) -> Result<()> {
        std::fs::write(path, self.render_to(template)?)?;
        Ok(())
    }

    pub fn swagger_ui_html_to_file<P: AsRef<std::path::Path>>(
        openapi_yaml_url: &str,
        title: &str,
        path: P,
    ) -> Result<()> {
        std::fs::write(path, Self::swagger_ui_html(openapi_yaml_url, title))?;
        Ok(())
    }

    pub fn redoc_ui_html_to_file<P: AsRef<std::path::Path>>(
        openapi_yaml_url: &str,
        title: &str,
        path: P,
    ) -> Result<()> {
        std::fs::write(path, Self::redoc_ui_html(openapi_yaml_url, title))?;
        Ok(())
    }
}

impl Default for OpenApiPlaceHolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(doctest)]
mod test_readme {

    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}
