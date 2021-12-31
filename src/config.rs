use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{crate_authors, crate_description, crate_name, crate_version, App, AppSettings};
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

#[derive(Clone, Default, Debug)]
pub struct DeadSession {
    pub dir: String,
    pub names: Vec<String>,
}

impl DeadSession {
    pub fn max_name_width(&self) -> usize {
        self.names.iter().map(|n| n.len()).max().unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub enum Task {
    List,
    Fzf,
    Popup,
}

impl Default for Task {
    fn default() -> Self {
        Self::Popup
    }
}

/// The all-in-one configuration model.
#[derive(Clone, Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub session_icons: HashMap<String, String>,

    #[serde(default)]
    pub sessions_dir: String,

    #[serde(skip)]
    pub dead_session: DeadSession,

    #[serde(skip)]
    pub task: Task,
}

impl Config {
    pub fn load() -> Config {
        let mut cfg = Self::load_from_config().unwrap_or_else(|error| {
            warn!("failed to load config: {:#?}", error,);
            Config::default()
        });

        debug!("load config: {:#?}", cfg);

        cfg.parse_args();
        cfg.discover_dead_sessions();

        cfg
    }

    fn load_from_config() -> Result<Config, Box<dyn Error>> {
        let mut path = dir();
        path.push("tav.toml");

        let text = fs::read_to_string(&path);
        Ok(toml::from_str(&text?)?)
    }

    fn parse_args(&mut self) {
        let list_cmd = App::new("list")
            .aliases(&["l", "ls"])
            .about("Print out colored feed for `fzf`, for debug purpose")
            .setting(AppSettings::Hidden);

        let fzf_cmd = App::new("fzf")
            .visible_alias("f")
            .about("Run fzf directly to show tmux tree, not in tmux popup window")
            .setting(AppSettings::Hidden);

        let matches = App::new(crate_name!())
            .author(crate_authors!())
            .version(crate_version!())
            .about(crate_description!())
            .subcommand(list_cmd)
            .subcommand(fzf_cmd)
            .get_matches();

        self.task = if let Some(_matches) = matches.subcommand_matches("list") {
            Task::List
        } else if let Some(_matches) = matches.subcommand_matches("fzf") {
            Task::Fzf
        } else {
            Task::Popup
        };
    }

    fn discover_dead_sessions(&mut self) {
        let mut path = dirs::home_dir().unwrap();
        path.push(".config/tav/sessions");

        self.sessions_dir = path.to_str().unwrap().to_string();

        path.push("*.tmux-session.zsh");
        let path = path.to_str().unwrap();

        debug!("glob on {}", &path);
        match glob::glob(path) {
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
