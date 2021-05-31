# OAPH

Helps to subtituated query params and schema definitions to openapi3 yaml.

This is feature-less simple crate with no ambitions to support whole openapi 3 specs and cover all cases (at least cover my personal use-cases).

```rust
use serde::{Serialize, Deserialize};
use oaph::{OpenApiPlaceHolder, schemars::{self, JsonSchema}};


#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct SearchQuery {
    /// some description for this flag (see https://graham.cool/schemars/examples/6-doc_comments/)
    flag: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct SearchResponse {
    success: bool,
    count: usize,
    items: Vec<Item>,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct Item {
    id: usize,
    value: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let openapi3_yaml = OpenApiPlaceHolder::new()
        .query_params::<SearchQuery>("SearchQuery")?
        .schema::<SearchResponse>("SearchResponse")?
        .render_to(r#"
openapi: 3.0.0
info:
  title: oaph example
  version: 1.0.0
paths:
  /search:
    get:
      tags:
      - demo
      description: demo api
      parameters:
        {{SearchQuery}}
      responses:
        '201':
          content:
            application/json:
              schema:
                {{SearchResponse}}
definitions:
  {{oaph::definitions}}
"#)?;

    println!("{}", openapi3_yaml);
    Ok(())
}
```

And output would be

```yaml
openapi: 3.0.0
info:
  title: oaph example
  version: 1.0.0
paths:
  /search:
    get:
      tags:
      - demo
      description: demo api
      parameters:
        - in: query
          name: flag
          description: some description for this flag (see https://graham.cool/schemars/examples/6-doc_comments/)
          required: true
          schema:
            type: boolean
      responses:
        '201':
          content:
            application/json:
              schema:
                title: SearchResponse
                type: object
                required:
                  - count
                  - items
                  - success
                properties:
                  success:
                    type: boolean
                  count:
                    type: integer
                    format: uint
                    minimum: 0.0
                  items:
                    type: array
                    items:
                      $ref: "#/definitions/Item"
definitions:
  Item:
    type: object
    required:
      - id
      - value
    properties:
      id:
        type: integer
        format: uint
        minimum: 0.0
      value:
        type: string
```

Check [example](examples/simple/src/main.rs) to serve docs on [ntex](https://github.com/ntex-rs/ntex) stack.


## Thanks to

 - [serde](https://github.com/serde-rs/serde) and buddies crates
 - [schemars](https://github.com/GREsau/schemars)

## License

This project is licensed under

* MIT license ([LICENSE](LICENSE) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
