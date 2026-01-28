# kboard

Pequeña utilidad para mapear eventos HID de un dispositivo (teclas y rueda)
a comandos del sistema según una configuración en YAML.

**Uso rápido**
- **Build:** `cargo build`
- **Run:** `cargo run --release` o `cargo run` para modo debug

**Configuración**
- El programa busca `config.yaml` en el directorio actual; si no existe,
  busca en `$HOME/.config/kboard/config.yaml`.
- Se prefiere el archivo en el directorio actual si ambos existen.

Ejemplo mínimo de `config.yaml`:

```yaml
keys:
  3: "xdg-open ~/.config/kboard/some-app.desktop"
  4: "notify-send 'Hola'"
wheel:
  1: "xdg-open https://example.org"
```

- Las claves y valores deben ser números (u8) y strings con el comando.

**Logging / Tracing**
- `tracing` está habilitado. Usa la variable de entorno `RUST_LOG`
  para controlar el nivel de logs.

Ejemplos:

```bash
# Ver logs informativos
export RUST_LOG=info
cargo run --release

# Ver debug detallado
export RUST_LOG=debug
cargo run
```

**Archivos relevantes**
- Código principal: [src/main.rs](src/main.rs)
- Archivo de configuración buscado: `config.yaml` o [~/.config/kboard/config.yaml](.config/kboard/config.yaml)

**Notas**
- Si `config.yaml` no se encuentra, la aplicación sigue ejecutándose sin acciones.
- Los comandos se ejecutan con la shell (`sh -c`) y se lanzan en background.

