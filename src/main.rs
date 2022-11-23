use std::thread;

use actix_web::{App, HttpServer};
use tempfile::tempdir;
use tracing::{info, Level};

use crate::config::Configuration;

mod config;
mod middleware;
mod repo;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .pretty()
        .init();

    let path = config::resolve_path().unwrap();
    let path_str = path.to_str().unwrap();
    info!(path = path_str, "Loading configuration from '{}'", path_str);

    let configuration: Configuration =
        config::load(&path).expect(&format!("Cannot read configuration from {}", path_str));

    let repositories = configuration.repositories.clone();
    let temp_dir = tempdir().unwrap().into_path();

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    for repo in repositories {
        handles.push(repo::watch(repo, temp_dir.clone()));
    }

    let host = configuration.network.host.to_owned();
    let port = configuration.network.port;
    let task = HttpServer::new(move || {
        App::new().wrap(middleware::ConfigServer::new(configuration.clone()))
    })
    .bind((host, port))?
    .run()
    .await;

    for thread in handles {
        thread.join().unwrap();
    }
    task
}
