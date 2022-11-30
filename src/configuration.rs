use std::{
    env,
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use simple_error::bail;

use crate::repository::Repo;

/// Environment variable pointing to the configserver yaml configuration file
static CONFIGSERVER_CFG: &str = "CONFIGSEVER_CFG";
/// Environment variable pointing to the directory where the configserver yaml configuration is to be found
static CONFIGSERVER_HOME: &str = "CONFIGSEVER_HOME";
/// Default name of the configserver configuation file
static CONFIGSERVER_YML: &str = "configserver.yml";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub name: String,
    pub network: Net,
    pub repositories: Vec<Repo>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Net {
    pub host: String,
    pub port: u16,
}

impl Configuration {
    /// Finds a repository matching the provided name in the configuration
    pub fn repository(&self, name: &str) -> Option<&Repo> {
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

impl Repo {
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
