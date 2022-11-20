use std::thread;

use tracing::{info, Level};


mod config;
mod repo;

fn main() {
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).pretty().init();

    let path = config::path().unwrap();
    let path_str = path.to_str().unwrap();
    info!(path = path_str, "Loading configuration from '{}'", path_str);

    let values =
        config::load(&path).expect(&format!("Cannot read configuration from {}", path_str));

    println!("{:?}", values);

    let repositories = values
        .repositories
        .expect("No repositories are defined in the configserver.yml file");

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    for repo in repositories {
        handles.push(repo::watch(repo));
    }

    for thread in handles {
        thread.join().unwrap();
    }

}
