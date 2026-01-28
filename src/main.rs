mod models;

use hidapi::HidApi;
use std::process::Command;
use tracing::{debug, error, info};
use tracing_subscriber;
use std::sync::{mpsc, Arc};
use std::thread;

use models::{
    Config,
    DeviceEvent,
    Hdi,
};


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
    let cfg = Arc::new(Config::load_config());
    let hdi = Arc::new(Hdi::new().expect("Error al inicializar Hdi"));
    let (tx, rx) = mpsc::channel();

    // 2. Hilo para las TECLAS
    let tx_keys = tx.clone();
    let hdi_keys = hdi.clone();
    thread::spawn(move || {
        let api = HidApi::new().unwrap();
        let p_keys = &hdi_keys.p_keys;
        if let Ok(dev) = api.open_path(p_keys) {
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
    let hdi_wheel = hdi.clone();
    thread::spawn(move || {
        let api = HidApi::new().unwrap();
        let p_wheel = &hdi_wheel.p_wheel;
        if let Ok(dev) = api.open_path(p_wheel) {
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

#[cfg(test)]
mod tests {
    use super::models::Config;
    use std::fs;
    use std::env;
    use std::sync::{Mutex, OnceLock};

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn test_lock<'a>() -> &'a Mutex<()> {
        TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_config_loads_empty_when_no_file() {
        let _guard = test_lock().lock().unwrap();
        // Test that Config::load_config() returns empty config when no file exists
        let orig_dir = env::current_dir().unwrap();
        let orig_home = env::var_os("HOME");
        let tmp = tempfile::tempdir().expect("tempdir");
        env::set_current_dir(tmp.path()).unwrap();
        unsafe { env::remove_var("HOME"); } // Ensure no HOME fallback
        
        let cfg = Config::load_config();
        assert!(cfg.keys.is_none());
        assert!(cfg.wheel.is_none());
        
        // Restore original state
        env::set_current_dir(orig_dir).unwrap();
        if let Some(home) = orig_home {
            unsafe { env::set_var("HOME", home); }
        }
    }

    #[test]
    fn test_config_prefers_cwd() {
        let _guard = test_lock().lock().unwrap();
        let orig_dir = env::current_dir().unwrap();
        let tmp = tempfile::tempdir().expect("tempdir");
        env::set_current_dir(tmp.path()).unwrap();
        
        // Create config.yaml in current directory
        let yaml = "keys:\n  3: \"echo test\"\nwheel:\n  1: \"echo wheel\"\n";
        fs::write(tmp.path().join("config.yaml"), yaml).unwrap();
        
        let cfg = Config::load_config();
        assert!(cfg.keys.is_some());
        assert_eq!(cfg.keys.unwrap().get(&3).map(String::as_str), Some("echo test"));
        
        env::set_current_dir(orig_dir).unwrap();
    }
}
