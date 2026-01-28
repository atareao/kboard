use serde::Deserialize;
use std::collections::HashMap;
use std::{fs, env, path::PathBuf};
use tracing::{info, warn, error};


#[derive(Debug, Deserialize)]
pub struct Config {
    pub keys: Option<HashMap<u8, String>>,
    pub wheel: Option<HashMap<u8, String>>,
}

fn load_config<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Config> {
    info!("📝 Cargando configuración desde {:?}", path.as_ref());
    let s = fs::read_to_string(path)?;
    let cfg: Config = serde_yaml::from_str(&s)?;
    Ok(cfg)
}

impl Config {
    pub fn load_config() -> Config {
        match find_config() {
            Some(path) => match load_config(&path) {
                Ok(c) => c,
                Err(e) => {
                    error!("Se encontró '{}' pero no se pudo cargar: {}. Continuando sin acciones.", path, e);
                    Config {
                        keys: None,
                        wheel: None,
                    }
                }
            },
            None => {
                warn!("No se encontró config.yaml en el directorio actual ni en ~/.config/kboard/. Continuando sin acciones configuradas.");
                Config {
                    keys: None,
                    wheel: None,
                }
            }
        }
    }
}

fn find_config() -> Option<String> {
    // Prefer `config.yaml` in current directory
    let cwd = std::path::Path::new("config.yaml");
    if cwd.exists() {
        return Some("config.yaml".to_string());
    }

    // Fallback to $HOME/.config/kboard/config.yaml
    if let Some(home) = env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".config/kboard/config.yaml");
        if p.exists() {
            return Some(p.to_string_lossy().into_owned());
        }
    }

    None
}