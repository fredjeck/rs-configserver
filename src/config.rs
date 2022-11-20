use std::{
    env,
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use simple_error::bail;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub repositories: Option<Vec<Repo>>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    pub url: String,
    pub user_name: String,
    pub password: String,
    pub refresh_interval: u64,
}

pub fn load(path: &PathBuf) -> Result<Configuration, Box<dyn Error>> {
    let f = File::open(path)?;
    let f = BufReader::new(f);

    let values: Configuration = serde_yaml::from_reader(f)?;
    println!("{:?}", values);
    Ok(values)
}

/// Tries to locate the configserver.yml file either in the local directory or in a folder pointed by the CONFIGSERVER_HOME environment variable
pub fn path() -> Result<PathBuf, Box<dyn Error>> {
    let path = env::current_dir()?;
    let mut config = path.join("configserver.yml");

    if config.exists() {
        return Ok(config);
    }

    config = match env::var("CONFIGSEVER_HOME") {
        Ok(val) => Path::new(val.as_str()).join("configserver.yml"),
        Err(_) => bail!("configserver.yml was neither found in the current directory neither in the folder pointed by the CONFIGSERVER_HOME environment variable") 
    };

    if config.exists() {
        Ok(config)
    } else {
        bail!("'{}': file not found", config.to_str().unwrap())
    }
}
