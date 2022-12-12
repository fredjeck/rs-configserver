use std::{
    env,
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use simple_error::bail;

use crate::repository::GitRepository;

/// Environment variable pointing to the configserver yaml configuration file
const CONFIGSERVER_CFG: &str = "CONFIGSERVER_CFG";
/// Environment variable pointing to the directory where the configserver yaml configuration is to be found
const CONFIGSERVER_HOME: &str = "CONFIGSERVER_HOME";
/// Default name of the configserver configuation file
const CONFIGSERVER_YML: &str = "configserver.yml";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Configuration {
    /// Name of the configserver insatnce
    pub name: String,
    /// Key used to encrypt sensitive data
    pub encryption_key: String,
    /// Network configuration
    pub network: Net,
    /// List of git repositories to serve
    pub repositories: Vec<GitRepository>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Net {
    /// Host on which the configserver will listen for incoming requests
    pub host: String,
    /// Port on which the configserver will listen for incoming requests
    pub port: u16,
}

impl Configuration {
    /// Finds a repository matching the provided name in the configuration
    pub fn repository(&self, name: &str) -> Option<&GitRepository> {
        self.repositories
            .iter()
            .find(|&x| x.name.eq_ignore_ascii_case(name))
    }
}

/// Loads the configserver yaml configuration file from the provided path
pub fn load(path: &PathBuf) -> Result<Configuration, Box<dyn Error>> {
    let f = File::open(path)?;
    let f = BufReader::new(f);

    let values: Configuration = serde_yaml::from_reader(f)?;
    Ok(values)
}

/// Tries to locate the configserver.yml file's path either located:
/// * in the current directory
/// * in a folder pointed by the CONFIGSERVER_HOME environment variable
/// * directly by the CONFIGSERVER_CFG environment variable
pub fn resolve_path() -> Result<PathBuf, Box<dyn Error>> {
    let err = "Configuration not found, search order is: $CONFIGSEVER_CFG, $CONFIGSEVER_HOME/configserver.yml, cwd";

    let config = match env::var(CONFIGSERVER_CFG) {
        Ok(val) => {
            let mut pb = PathBuf::new();
            pb.push(val);
            pb
        }
        _ => match env::var(CONFIGSERVER_HOME) {
            Ok(val) => Path::new(val.as_str()).join(CONFIGSERVER_YML),
            _ => {
                let path = env::current_dir()?;
                path.join(CONFIGSERVER_YML)
            }
        },
    };

    if config.exists() {
        return Ok(config);
    }
    bail!(err)
}

impl GitRepository {
    /// Checks if the provided user and password can access this repository
    /// TODO Improve with password encryption - as for now this is only for testing purposes
    pub fn is_granted_for(&self, user: &str, password: &str) -> bool {
        let users = match &self.credentials {
            Some(c) => c,
            None => return true, // No credentials means the repo can be accessed by anyone
        };

        let grant = match users
            .iter()
            .find(|&x| x.user_name.eq_ignore_ascii_case(user))
        {
            Some(c) => c,
            None => return false,
        };

        grant.password == password
    }
}
