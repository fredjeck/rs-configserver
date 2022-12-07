use git2::Repository;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use std::{
    path::{PathBuf, Path},
    thread::{self, JoinHandle},
    time::Duration, fs, str::FromStr,
};

use crate::crypto::{decrypt_base64_string};

/// A structure holding GIT repositories configurations
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GitRepository {
    /// Repository name
    pub name: String,
    /// URL at which the repository can be cloned 
    pub url: String, 
    /// Optional username to be used while cloning the repository
    pub user_name: String, 
    /// Optional password to be used while cloning the repository
    pub password: String,
     // Repository refresh interval
    pub refresh_interval: u64,
    /// Credentials allowed to access this repository via the ConfigServer API
    pub credentials: Option<Vec<Credential>>,
}

/// A structure defining user credentials allowed to access a repository
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Credential {
    pub user_name: String,
    pub password: String,
}

impl GitRepository {
    /// Clones the provided repository to the specified location and refreshes it according to the configured refreshInterval
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

/// Loads a file from the specified repository taking care of decrypting sensitive data
pub fn load_file(base_path: &str, repository_path: &str, file_path : &str, key: &str) ->Option<String>{
    let path = Path::new(base_path).join(repository_path).join(file_path);
    let path_log = path.clone();
    debug!("Loading {:?}", &path_log);

    let mut content = match fs::read_to_string(path) {
        Ok(txt) => txt,
        Err(_) => {
            error!("Failed to load '{:?}', file does not exist", &path_log);
            return None
        },
    };

    let re = Regex::new(r"(\{enc:.*?\})").unwrap();

    for cap in re.captures_iter(&content.clone()) {
        let enc = &cap[1][5..&cap[1].len()-1];
        let dec = decrypt_base64_string(key, &enc).unwrap_or(String::from_str(enc).unwrap());

        content = content.replace(&cap[1], &dec);
    }

    Some(content)
}