use std::{thread, str::FromStr, sync::Mutex};

use actix_web::{App, HttpServer, web::{self}, post};
use futures_util::StreamExt;

use tempfile::tempdir;
use tracing::{info, Level};

use crate::configuration::Configuration;

use magic_crypt::{new_magic_crypt, MagicCryptTrait};

mod configuration;
mod middleware;
mod repository;

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
        .service(encrypt)
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

#[post("/encrypt")]
async fn encrypt(mut body: web::Payload, data: web::Data<Configuration>) -> actix_web::Result<String> {

    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item?);
    }

    let body = match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(_) => String::from_str("").unwrap()
    };

    let mc = new_magic_crypt!(&data.encryptionKey, 256);
    let base64 = mc.encrypt_str_to_base64(body);
    

    Ok(base64)
}