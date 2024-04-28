use ntex::web;

#[web::get("/")]
async fn hello() -> impl web::Responder{
    "Welcome to the API"
}

#[ntex::main]
async fn main()->std::io::Result<()> {
    web::HttpServer::new(|| {
        web::App::new()
        .service(hello)
    })
    .bind(("localhost",80))?
    .run()
    .await
}
