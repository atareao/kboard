# kboard

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/atareao/kboard)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-linux-lightgrey.svg)](https://www.linux.org)

Pequeña utilidad para mapear eventos HID de un dispositivo (teclas y rueda) a comandos del sistema según una configuración en YAML.

## 🚀 Características

- ✅ Detección automática de dispositivos HID (Vendor ID: `0x1189`, Product ID: `0x8890`)
- ✅ Configuración flexible mediante archivos YAML
- ✅ Búsqueda automática de configuración en múltiples ubicaciones
- ✅ Sistema de logging avanzado con `tracing`
- ✅ Ejecución de comandos en background
- ✅ Pruebas unitarias incluidas

## 🛠️ Instalación y Uso

### Compilación

```bash
# Compilar en modo debug
cargo build

# Compilar en modo release (recomendado)
cargo build --release
```

### Ejecución

```bash
# Ejecutar directamente
cargo run --release

# O usar el binario compilado
./target/release/kboard
```

## ⚙️ Configuración

El programa busca automáticamente el archivo `config.yaml` en:

1. **Directorio actual** (prioridad alta)
2. **`$HOME/.config/kboard/config.yaml`** (fallback)

### Ejemplo de configuración

```yaml
keys:
  3: "xdg-open ~/.config/kboard/some-app.desktop"
  4: "notify-send 'Tecla 4 presionada'"
  5: "pactl set-sink-volume @DEFAULT_SINK@ +5%"
wheel:
  1: "xdg-open https://example.org"
  2: "pactl set-sink-volume @DEFAULT_SINK@ -5%"
  3: "pactl set-sink-mute @DEFAULT_SINK@ toggle"
```

### Formato

- **Claves**: Números enteros (`u8`) que corresponden a los códigos de tecla/rueda
- **Valores**: Comandos de shell que se ejecutarán

## 📊 Logging / Tracing

El sistema de logging está basado en `tracing` y se controla mediante la variable de entorno `RUST_LOG`:

```bash
# Logs informativos (recomendado)
RUST_LOG=info cargo run --release

# Debug detallado
RUST_LOG=debug cargo run

# Solo errores
RUST_LOG=error cargo run --release

# Logs específicos del módulo
RUST_LOG=kboard=debug cargo run
```

### Niveles disponibles
- `error`: Solo errores críticos
- `warn`: Advertencias y errores
- `info`: Información general, advertencias y errores
- `debug`: Información detallada para depuración
- `trace`: Máximo nivel de detalle

## 🧪 Pruebas

```bash
# Ejecutar todas las pruebas
cargo test

# Ejecutar pruebas con salida detallada
cargo test -- --nocapture

# Ejecutar pruebas específicas
cargo test test_config_loads_empty_when_no_file
```

## 📁 Estructura del Proyecto

```
kboard/
├── src/
│   ├── main.rs              # Punto de entrada y lógica principal
│   └── models/              # Módulos de datos
│       ├── mod.rs           # Exportaciones del módulo
│       ├── config.rs        # Manejo de configuración YAML
│       ├── device_event.rs  # Eventos de dispositivo
│       └── hdi.rs          # Interfaz con dispositivos HID
├── Cargo.toml              # Dependencias y configuración del proyecto
├── config.yaml             # Archivo de configuración (opcional)
└── README.md               # Este archivo
```

## 🔧 Dependencias Principales

- **`hidapi`**: Interfaz con dispositivos HID
- **`serde`** + **`serde_yaml`**: Serialización/deserialización de YAML
- **`tracing`** + **`tracing-subscriber`**: Sistema de logging estructurado
- **`anyhow`**: Manejo de errores mejorado

## ⚠️ Requisitos del Sistema

- **Rust 1.70+**
- **Linux** (probado en distribuciones modernas)
- **Permisos de acceso a dispositivos HID** (puede requerir udev rules o ejecutar como root)

## 🐛 Resolución de Problemas

### El dispositivo no se detecta

1. Verificar que el dispositivo esté conectado:
   ```bash
   lsusb | grep 1189
   ```

2. Verificar permisos de acceso a `/dev/hidraw*`

3. Añadir regla udev si es necesario:
   ```bash
   echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1189", ATTRS{idProduct}=="8890", MODE="0666"' | sudo tee /etc/udev/rules.d/99-kboard.rules
   sudo udevadm control --reload-rules
   ```

### Los comandos no se ejecutan

1. Verificar que el archivo `config.yaml` tenga la sintaxis correcta
2. Comprobar que los comandos funcionen manualmente en terminal
3. Revisar logs con `RUST_LOG=debug`

## 📝 Notas

- Si no se encuentra `config.yaml`, la aplicación continúa ejecutándose sin acciones configuradas
- Los comandos se ejecutan usando la shell (`sh -c`) y se lanzan en background
- No se espera a que terminen los comandos ejecutados

## 📄 Licencia

MIT License - ver el archivo [LICENSE](LICENSE) para más detalles.

