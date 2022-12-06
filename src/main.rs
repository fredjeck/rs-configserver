use std::{thread};

use actix_web::{App, HttpServer, web::{self, post, Bytes}, HttpResponse};

use tempfile::tempdir;
use tracing::{info, Level};

use crate::configuration::Configuration;

mod configuration;
mod middleware;
mod repository;
mod crypt;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .pretty()
        .init();

    let path = configuration::resolve_path().unwrap();
    let path_str = path.to_str().unwrap();
    info!(path = path_str, "Loading configuration from '{}'", path_str);

    let configuration: Configuration =
        configuration::load(&path).expect(&format!("Cannot read configuration from {}", path_str));

    

    

    let repositories = configuration.repositories.clone();
    let temp_dir = tempdir().unwrap().into_path();

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    for repo in repositories {
        handles.push(repo.create_watcher(temp_dir.clone()));
    }

    let data = web::Data::new(configuration.clone());
    let host = configuration.network.host.to_owned();
    let port = configuration.network.port;
    let task = HttpServer::new(move || {
        App::new().wrap(middleware::ConfigServer::new(configuration.clone(), temp_dir.clone()))
        .route("/encrypt", post().to(encryption_handler))
        .app_data(data.clone())
    })
    .bind((host, port))?
    .run()
    .await;

    for thread in handles {
        thread.join().unwrap();
    }
    task
}



async fn encryption_handler(bytes: Bytes, data: web::Data<Configuration>) -> HttpResponse {
    let body = match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(_) => return HttpResponse::BadRequest().finish()
    };
    HttpResponse::Ok().body(crypt::encrypt_str(&data.encryption_key, &body))
}