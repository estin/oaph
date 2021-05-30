use ntex::web::{self, App, HttpRequest, HttpResponse};
use ntex_files as fs;
use oaph::OpenApiPlaceHolder;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;


#[allow(dead_code)]
#[derive(Serialize, Deserialize, JsonSchema)]
struct SearchQuery {
    /// search pattern 
    q: String,
    /// search in archive too
    flag: Option<bool>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, JsonSchema)]
struct User {
    id: usize,
    staff: bool,
    tags: Vec<String>,
}


#[allow(dead_code)]
#[derive(Serialize, Deserialize, JsonSchema)]
struct Item {
    id: usize,
    width: Option<f64>,
    owner: User,
}


/// Search result 
#[allow(dead_code)]
#[derive(Serialize, Deserialize, JsonSchema)]
struct SearchResponse {
    /// if `true` is no errors occur 
    success: bool,
    /// total found items
    count: usize,
    items: Vec<Item>,
}

async fn search(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(&SearchResponse {
        success: true,
        count: 1,
        items: vec![Item {
            id: 1,
            width: None,
            owner: User {
                id: 1006,
                staff: false,
                tags: vec![],
            }
        }]
    })
}

fn generate_openapi_files() -> Result<(), Box<dyn std::error::Error>> {
    // render openapi3 yaml to temporary file
    OpenApiPlaceHolder::new()
        .query_params::<SearchQuery>("SearchQuery")?
        .schema::<SearchResponse>("SearchResponse")?
        .render_to_file(include_str!("../misc/openapi3.yaml"), "/tmp/openapi3.yaml")?;

    // render swagger ui html to temporary file
    OpenApiPlaceHolder::swagger_ui_html_to_file("/openapi3.yaml", "/tmp/swagger-ui.html")?;

    Ok(())
}


#[ntex::main]
async fn main() -> std::io::Result<()> {

    generate_openapi_files().expect("On generate openapi files");
    
    let fut = web::server(move || {
        App::new().service((
            web::resource("/search").to(search),
            // serve openapi3 yaml and ui from files
            fs::Files::new("/openapi3.yaml", "/tmp").index_file("openapi3.yaml"),
            fs::Files::new("/swagger", "/tmp").index_file("swagger-ui.html"),
        ))})
        .bind("127.0.0.1:8080")?
        .run();

    println!("swagger-ui: http://127.0.0.1:8080/swagger");

    fut.await
}
