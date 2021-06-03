use anyhow::{bail, Result};
use std::collections::HashMap;

use schemars::{gen::SchemaSettings, schema::RootSchema, schema::Schema, JsonSchema, Map};
use serde::Serialize;

// re-export
pub use schemars;

pub struct OpenApiPlaceHolder {
    ph: HashMap<String, String>,
    definitions: Map<String, Schema>,
}

impl OpenApiPlaceHolder {
    pub fn new() -> Self {
        OpenApiPlaceHolder {
            ph: HashMap::new(),
            definitions: Map::new(),
        }
    }

    fn describe_struct<T: JsonSchema>(&mut self) -> RootSchema {
        let settings = SchemaSettings::draft07().with(|s| {
            s.option_nullable = true;
            s.option_add_null_type = false;
        });
        let gen = settings.into_generator();
        let root_schema = gen.into_root_schema_for::<T>();

        // populate definitions
        for (k, v) in root_schema.clone().definitions {
            self.definitions.insert(k, v);
        }

        root_schema
    }

    fn to_yaml(schema: &impl Serialize) -> Result<String> {
        Ok(serde_yaml::to_string(schema)?
            .replace("---\n", "")
            .trim()
            .to_owned())
    }

    pub fn query_params<T: JsonSchema>(mut self, name: &str) -> Result<Self> {
        let root_schema = self.describe_struct::<T>();
        let parameters = match root_schema.schema.object {
            Some(object) => {
                let mut result: Vec<String> = Vec::new();

                for (name, schema) in object.properties.iter() {
                    let (required, description) = match schema {
                        Schema::Object(o) => (
                            if o.extensions.get("nullable").is_some() {
                                "false"
                            } else {
                                "true"
                            },
                            if let Some(metadata) = o.metadata.as_ref() {
                                metadata.description.as_ref()
                            } else {
                                None
                            },
                        ),
                        _ => ("false", None),
                    };

                    let description_entry = if let Some(desc) = description {
                        format!("  description: {}\n", desc.trim()).to_string()
                    } else {
                        "".to_string()
                    };

                    // remove description from schema block
                    let schema = Self::to_yaml(&schema)?
                        .split('\n')
                        .filter(|line| !line.contains("description:"))
                        .collect::<Vec<&str>>()
                        .join("\n");

                    result.push(format!(
                        "- in: query\n  name: {name}\n{description}  required: {required}\n  schema:\n    {schema}",
                        name=name,
                        required=required,
                        description=description_entry,
                        schema=Self::with_indent("    ", &schema),
                    ))
                }

                result.join("\n")
            }
            _ => bail!("Only object type supported"),
        };

        self.ph.insert(name.to_owned(), parameters);

        Ok(self)
    }

    pub fn schema<T: JsonSchema>(mut self, name: &str) -> Result<Self> {
        let root_schema = self.describe_struct::<T>();

        self.ph
            .insert(name.to_owned(), Self::to_yaml(&root_schema.schema)?);

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

                        line.replace(&pattern, &Self::with_indent(indent, &v))
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

    pub fn swagger_ui_html(openapi_yaml_url: &str) -> String {
        include_str!("../misc/swagger-ui.html").replace("{{openapi_yaml_url}}", openapi_yaml_url)
    }

    pub fn redoc_ui_html(openapi_yaml_url: &str) -> String {
        include_str!("../misc/redoc-ui.html").replace("{{openapi_yaml_url}}", openapi_yaml_url)
    }

    pub fn render_to_file<P: AsRef<std::path::Path>>(self, template: &str, path: P) -> Result<()> {
        std::fs::write(path, &self.render_to(template)?)?;
        Ok(())
    }

    pub fn swagger_ui_html_to_file<P: AsRef<std::path::Path>>(
        openapi_yaml_url: &str,
        path: P,
    ) -> Result<()> {
        std::fs::write(path, &Self::swagger_ui_html(openapi_yaml_url))?;
        Ok(())
    }

    pub fn redoc_ui_html_to_file<P: AsRef<std::path::Path>>(
        openapi_yaml_url: &str,
        path: P,
    ) -> Result<()> {
        std::fs::write(path, &Self::redoc_ui_html(openapi_yaml_url))?;
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
