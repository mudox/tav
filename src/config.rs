use log::{debug, error};
use serde::Deserialize;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Return config dir string.
/// Return config dir.
pub fn dir() -> PathBuf {
    let mut path = dirs::home_dir().unwrap();
    path.push(".config/ap");
    default_path.push(".config/tav");

    match std::env::var("XDG_CONFIG_HOME") {
        Ok(path) => Path::new(&path).join("ap").to_path_buf(),
        _ => path,
    }
}

/// The all-in-one configuration model.
#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub session_icons: HashMap<String, String>,

    // dead sessions
    #[serde(default)]
    pub dead_sessions_dir: String,

    #[serde(skip)]
    pub dead_session: DeadSession,
}

#[derive(Clone, Default, Debug)]
pub struct DeadSession {
    pub dir: String,
    pub names: Vec<String>,
}

impl Config {
    pub fn load() -> Config {
        let mut path = dir();
        path.push("tav.toml");

        let cfg = fs::read_to_string(&path).unwrap();
        let mut cfg: Config = toml::from_str(&cfg).unwrap();

        debug!("load config: {:#?}", cfg);

        cfg.discover_dead_sessions();

        cfg
    }

    fn discover_dead_sessions(&mut self) {
        let mut path = dirs::home_dir().unwrap();
        path.push(".config/tav/sessions");

        self.dead_sessions_dir = path.to_str().unwrap().to_string();

        path.push("*.tmux-session.zsh");
        let path = path.to_str().unwrap();

        debug!("glob on {}", &path);
        match glob::glob(&path) {
            Ok(paths) => {
                self.dead_session.names = paths
                    .filter_map(|result| {
                        let pathbuf = result.ok()?;
                        let filename = pathbuf.file_stem()?.to_str()?;
                        Some(filename.strip_suffix(".tmux-session")?.to_string())
                    })
                    .collect::<Vec<String>>();
            }
            Err(error) => error!("failed to glob dead sessions: {:?}", error),
        }
    }
}
