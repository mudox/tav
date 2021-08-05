use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::logging::*;

/// Return config dir string.
/// Return config dir.
pub fn dir() -> PathBuf {
    let mut default_path = dirs::home_dir().unwrap();
    default_path.push(".config/tav");

    match std::env::var("XDG_CONFIG_HOME") {
        Ok(path) => Path::new(&path).join("ap"),
        _ => default_path,
    }
}

/// Colors
#[derive(Clone, Deserialize)]
pub struct Color {}

/// The all-in-one configuration model.
#[derive(Clone, Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub session_icons: HashMap<String, String>,

    // dead sessions
    #[serde(default)]
    pub sessions_dir: String,

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

        let text = fs::read_to_string(&path);

        if let Err(error) = text {
            warn!(
                "failed to read from config file:\n  path: {:?}\n  error: {:#?}",
                &path, error,
            );

            let mut cfg = Config::default();
            cfg.discover_dead_sessions();

            return cfg;
        }

        let mut cfg: Config = toml::from_str(&text.unwrap()).unwrap_or_else(|error| {
            warn!(
                "failed to deserialize from file:\n path: {:?}\n  error: {:#?}",
                &path, error,
            );

            let mut cfg = Config::default();
            cfg.discover_dead_sessions();

            return cfg;
        });

        debug!("load config: {:#?}", cfg);

        cfg.discover_dead_sessions();

        cfg
    }

    fn discover_dead_sessions(&mut self) {
        let mut path = dirs::home_dir().unwrap();
        path.push(".config/tav/sessions");

        self.sessions_dir = path.to_str().unwrap().to_string();

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
