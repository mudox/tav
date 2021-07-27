use log::{debug, error};

#[derive(Clone)]
pub struct Config {
    pub dead_sessions_dir: String,
    pub dead_sessions: Vec<String>,
}

impl Config {
    pub fn load() -> Config {
        let mut cfg = Config {
            dead_sessions_dir: "".to_string(),
            dead_sessions: Vec::new(),
        };

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
                self.dead_sessions = paths
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
