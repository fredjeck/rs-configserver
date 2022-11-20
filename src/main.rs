use actix_web::{
    get, http::header, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use tracing::{info, Level};

use crate::config::Configuration;

mod config;
mod repo;
mod middleware;

#[get("/")]
async fn hello(_req: HttpRequest) -> impl Responder {
    if !_req.headers().contains_key(header::AUTHORIZATION) {
        return HttpResponse::Unauthorized()
            .append_header((
                header::WWW_AUTHENTICATE,
                "Basic realm=\"ConfigServer\", charset=\"UTF-8\"",
            ))
            .finish();
    } else {
        let auth = _req.headers().get(header::AUTHORIZATION).unwrap();
        print!("{:?}", auth);
        return HttpResponse::Ok().body("Hey there!");
    }
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .pretty()
        .init();

    let path = config::path().unwrap();
    let path_str = path.to_str().unwrap();
    info!(path = path_str, "Loading configuration from '{}'", path_str);

    let configuration: Configuration=
        config::load(&path).expect(&format!("Cannot read configuration from {}", path_str));

    println!("{:?}", configuration);

    let _ = configuration
        .repositories;

    let host = configuration.network.host.to_owned();
    let port = configuration.network.port;
    HttpServer::new( move || {
        App::new()
            .wrap(middleware::ConfigServer::new(configuration.clone()))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind((host, port))?
    .run()
    .await
}
