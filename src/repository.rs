use git2::Repository;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    pub url: String,
    pub user_name: String,
    pub password: String,
    pub refresh_interval: u64,
    pub credentials: Option<Vec<Credential>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Credential {
    pub user_name: String,
    pub password: String,
}

impl Repo {
    /// Clones the provided repository to the specified location and refreshes it according to the configuration refreshInterval
    pub fn create_watcher(self, location: PathBuf) -> JoinHandle<()> {
        return thread::spawn(move || {
            let loc = location.join(&self.name);
            info!("Cloning '{}' to {}", &self.url, loc.to_str().unwrap());
            let repo = match Repository::clone(&self.url, loc) {
                Ok(repo) => repo,
                Err(e) => {
                    error!(
                        repository = &self.url,
                        "Unable clone repo '{}': {} - this repository will be skipped",
                        &self.url,
                        e
                    );
                    return ();
                }
            };
            loop {
                thread::sleep(Duration::from_millis(self.refresh_interval));
                debug!(repository = &self.url, "Refreshing '{}'", &self.url);
                let mut origin_remote = repo.find_remote("origin").unwrap();
                origin_remote.fetch(&["main"], None, None).unwrap();
            }
        });
    }
}
