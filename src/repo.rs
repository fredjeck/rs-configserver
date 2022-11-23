use crate::config::Repo;
use git2::Repository;
use tracing::{debug, error, info};

use std::{
    path::{PathBuf},
    thread::{self, JoinHandle},
    time::Duration,
};

/// Clones the provided repository to the specified location and refreshes it according to the configuration refreshInterval
pub fn watch(conf: Repo, location: PathBuf) -> JoinHandle<()> {
    return thread::spawn(move || {
        let loc = location.join(&conf.name);
        info!("Cloning '{}' to {}", &conf.url, loc.to_str().unwrap());
        let repo = match Repository::clone(&conf.url, loc) {
            Ok(repo) => repo,
            Err(e) => {
                error!(
                    repository = &conf.url,
                    "Unable clone repo '{}': {} - this repository will be skipped", &conf.url, e
                );
                return ();
            }
        };
        loop {
            thread::sleep(Duration::from_millis(conf.refresh_interval));
            debug!(repository = &conf.url, "Refreshing '{}'", &conf.url);
            let mut origin_remote = repo.find_remote("origin").unwrap();
            origin_remote.fetch(&["main"], None, None).unwrap();
        }
    });
}
