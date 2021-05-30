use anyhow::Result;
use oaph::OpenApiPlaceHolder;
use schemars::JsonSchema;
use serde::Deserialize;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn query_params() -> Result<()> {
    init();

    #[allow(dead_code)]
    #[derive(Deserialize, JsonSchema)]
    struct Query {
        /// input description
        input: String,
        tags: Vec<String>,
        extra: Option<usize>,
    }

    assert_eq!(
        include_str!("../tests/misc/query_params.yaml").trim(),
        OpenApiPlaceHolder::new()
            .query_params::<Query>("Query")?
            .render_to("{{Query}}")?
    );
    Ok(())
}

#[test]
fn schema() -> Result<()> {
    init();

    #[allow(dead_code)]
    #[derive(Deserialize, JsonSchema)]
    struct Request {
        username: String,
    }

    #[allow(dead_code)]
    #[derive(Deserialize, JsonSchema)]
    struct User {
        id: usize,
        username: String,
        tags: Vec<String>,
    }

    #[allow(dead_code)]
    #[derive(Deserialize, JsonSchema)]
    struct Item {
        id: usize,
        width: Option<f64>,
        owner: User,
    }

    #[allow(dead_code)]
    #[derive(Deserialize, JsonSchema)]
    struct Response {
        success: bool,
        count: usize,
        items: Vec<Item>,
    }

    assert_eq!(
        include_str!("../tests/misc/schema-result.yaml"),
        OpenApiPlaceHolder::new()
            .schema::<Request>("Request")?
            .schema::<Response>("Response")?
            .render_to(include_str!("../tests/misc/schema-oa3.yaml"))?
    );
    Ok(())
}
