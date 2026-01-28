use hidapi::HidApi;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::env;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use tracing_subscriber;
use std::sync::{mpsc, Arc};
use std::thread;

// Definimos los IDs del dispositivo
const VENDOR_ID: u16 = 0x1189;
const PRODUCT_ID: u16 = 0x8890;

#[derive(Debug)]
enum DeviceEvent {
    Key(u8),
    Wheel(u8),
}

#[derive(Debug, Deserialize)]
struct Config {
    keys: Option<HashMap<u8, String>>,
    wheel: Option<HashMap<u8, String>>,
}

fn load_config<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Config> {
    info!("📝 Cargando configuración desde {:?}", path.as_ref());
    let s = fs::read_to_string(path)?;
    let cfg: Config = serde_yaml::from_str(&s)?;
    Ok(cfg)
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

fn try_exec(cmd: &str) {
    // Ejecuta el comando usando la shell para permitir pipelines, redirecciones, etc.
    match Command::new("sh").arg("-c").arg(cmd).spawn() {
        Ok(child) => {
            // No esperamos al comando, lo dejamos correr en background
            info!("Ejecutando: '{}' (pid={})", cmd, child.id());
        }
        Err(e) => error!("Error al ejecutar '{}': {}", cmd, e),
    }
}

fn main() {
    // Inicializar tracing (puede usar RUST_LOG para filtrar)
    tracing_subscriber::fmt::init();

    // Cargamos la configuración YAML. Buscamos en el directorio actual primero,
    // luego en `$HOME/.config/kboard/config.yaml`.
    let cfg = match find_config() {
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
    };

    let cfg = Arc::new(cfg);

    let api = HidApi::new().expect("Error al inicializar HIDAPI");
    let (tx, rx) = mpsc::channel();

    // 1. Buscamos los paths dinámicamente
    let mut path_keys = None;
    let mut path_wheel = None;

    for device in api.device_list() {
        if device.vendor_id() == VENDOR_ID && device.product_id() == PRODUCT_ID {
            match device.interface_number() {
                0 => path_keys = Some(device.path().to_owned()),
                2 => path_wheel = Some(device.path().to_owned()),
                _ => {}
            }
        }
    }

    let p_keys = path_keys.expect("No se encontró la interfaz de teclas (Int 0)");
    let p_wheel = path_wheel.expect("No se encontró la interfaz de la rueda (Int 2)");

    // 2. Hilo para las TECLAS
    let tx_keys = tx.clone();
    thread::spawn(move || {
        let api = HidApi::new().unwrap();
        if let Ok(dev) = api.open_path(&p_keys) {
            let mut buf = [0u8; 64];
            loop {
                if let Ok(res) = dev.read(&mut buf) {
                    if res > 3 && buf[3] != 0 {
                        let _ = tx_keys.send(DeviceEvent::Key(buf[3]));
                    }
                }
            }
        }
    });

    // 3. Hilo para la RUEDA
    let tx_wheel = tx.clone();
    thread::spawn(move || {
        let api = HidApi::new().unwrap();
        if let Ok(dev) = api.open_path(&p_wheel) {
            let mut buf = [0u8; 64];
            loop {
                if let Ok(res) = dev.read(&mut buf) {
                    // Buscamos el Report ID 2 y que el byte 1 tenga contenido
                    if res >= 2 && buf[0] == 2 && buf[1] != 0 {
                        let _ = tx_wheel.send(DeviceEvent::Wheel(buf[1]));
                    }
                }
            }
        }
    });

    info!("✅ Dispositivo vinculado correctamente.");
    info!("🚀 Escuchando eventos... ");

    // 4. Bucle principal de ejecución
    for event in rx {
        match event {
            DeviceEvent::Key(code) => {
                if let Some(cmd) = cfg.keys.as_ref().and_then(|m| m.get(&code)).cloned() {
                    try_exec(&cmd);
                } else {
                    debug!("Tecla sin acción configurada: {}", code);
                }
            }
            DeviceEvent::Wheel(val) => {
                if let Some(cmd) = cfg.wheel.as_ref().and_then(|m| m.get(&val)).cloned() {
                    try_exec(&cmd);
                } else {
                    debug!("Rueda sin acción configurada: {}", val);
                }
            }
        }
    }
}
