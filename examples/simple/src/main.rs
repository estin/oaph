use ntex::web::{self, App, HttpRequest, HttpResponse};
use ntex_files as fs;
use oaph::{OpenApiPlaceHolder, schemars::{self, JsonSchema}};
use serde::{Serialize, Deserialize};


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
        .render_to_file(include_str!("../misc/openapi3.yaml"), std::env::temp_dir().join("openapi3.yaml"))?;

    // render swagger ui html to temporary file
    OpenApiPlaceHolder::swagger_ui_html_to_file("/openapi3.yaml", "OAPH example", std::env::temp_dir().join("swagger-ui.html"))?;

    // render redoc ui html to temporary file
    OpenApiPlaceHolder::redoc_ui_html_to_file("/openapi3.yaml", "OAPH example", std::env::temp_dir().join("redoc-ui.html"))?;

    Ok(())
}


#[ntex::main]
async fn main() -> std::io::Result<()> {

    generate_openapi_files().expect("On generate openapi files");
    
    let fut = web::server(move || {
        App::new().service((
            web::resource("/search").to(search),
            // serve openapi3 yaml and ui from files
            fs::Files::new("/openapi3.yaml", std::env::temp_dir()).index_file("openapi3.yaml"),
            fs::Files::new("/swagger", std::env::temp_dir()).index_file("swagger-ui.html"),
            fs::Files::new("/redoc", std::env::temp_dir()).index_file("redoc-ui.html"),
        ))})
        .bind("127.0.0.1:8080")?
        .run();

    println!("swagger-ui: http://127.0.0.1:8080/swagger");
    println!("redoc-ui: http://127.0.0.1:8080/redoc");

    fut.await
}
