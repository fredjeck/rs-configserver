use crate::config::Repo;
use git2::Repository;
use tempfile::tempdir;
use tracing::{error, debug};

use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

pub fn watch(conf: Repo) -> JoinHandle<()> {
    return thread::spawn(move || {
        let tempdir = tempdir().unwrap();
        debug!(
            repository = &conf.url,
            "Clonig '{}'", &conf.url
        );
        let repo = match Repository::clone(&conf.url, tempdir) {
            Ok(repo) => repo,
            Err(e) => {
                error!(
                    repository = &conf.url,
                    "Unable clone repo '{}': {}", &conf.url, e
                );
                return ();
            }
        };
        loop {
            thread::sleep(Duration::from_millis(conf.refresh_interval));
            debug!(
                repository = &conf.url,
                "Refreshing '{}'", &conf.url
            );
            let mut origin_remote = repo.find_remote("origin").unwrap();
            origin_remote.fetch(&["main"], None, None).unwrap();
        }
    });
}
