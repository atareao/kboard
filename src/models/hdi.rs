
use hidapi::HidApi;

pub struct Hdi {
    // Definimos los IDs del dispositivo
    pub p_keys: std::ffi::CString,
    pub p_wheel: std::ffi::CString,
}

// Definimos los IDs del dispositivo
const VENDOR_ID: u16 = 0x1189;
const PRODUCT_ID: u16 = 0x8890;


impl Hdi {
    pub fn new() -> Result<Self, anyhow::Error> {
        let api = HidApi::new()?;
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

        let p_keys = path_keys.ok_or_else(|| anyhow::anyhow!("No se encontró la interfaz de teclas (Int 0)"))?;
        let p_wheel = path_wheel.ok_or_else(|| anyhow::anyhow!("No se encontró la interfaz de la rueda (Int 2)"))?;

        Ok(Hdi { p_keys, p_wheel })
    }
}
