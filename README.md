# VOXSecurityTUI

Una interfaz de terminal (TUI) para auditoría de seguridad autorizada. Construida en Rust con [ratatui](https://ratatui.rs).

VOXSecurityTUI es un **orquestador**: detecta qué herramientas de análisis de seguridad están instaladas en el host, las presenta en un catálogo y ejecuta la que elijas contra un objetivo, mostrando su salida en vivo. No reempaqueta ni enlaza ninguna herramienta; sólo invoca los binarios que ya tenés instalados.

> Uso autorizado. Esta herramienta está pensada para auditoría con permiso expreso del objetivo y para uso defensivo. Usarla sin autorización puede ser ilegal. Las corridas (herramienta, objetivo, comando y resultado), las instalaciones y los errores quedan registrados en el log de la aplicación (ver [Logs](#logs)).

## Características

- Catálogo de ~30 herramientas (reconocimiento, escaneo de red, web, contraseñas, wireless, hardening, forense y tráfico) con su paquete por distribución y licencia, tomado de `SPEC.md`.
- Detección periódica de qué herramientas están instaladas, con recarga en caliente del intervalo.
- Instalación asistida desde el propio catálogo (tecla `I`), usando el gestor de paquetes detectado (apt, dnf, yum, zypper, pacman, paru, yay, brew).
- Ejecución de una herramienta contra un objetivo con argumentos editables, y salida (stdout+stderr) transmitida línea a línea.
- Timeout por comando: al vencer se termina la herramienta junto con todos sus procesos hijos (SIGTERM y, tras 2 s, SIGKILL). Lo mismo ocurre al salir de la app con una corrida en curso.
- Confirmación explícita (`S`/`Y`) antes de lanzar argumentos que modifican o eliminan archivos del objetivo (p. ej. `clamscan --remove`/`--move`, escritura de metadatos con `exiftool`).
- Historial de corridas con su estado y salida, acotado a las últimas 50 corridas y 1 MiB / 5000 líneas de salida por corrida (líneas de más de 4 KiB se recortan).
- Resumen del host (CPU/RAM/disco) y aviso legal siempre visible.
- Interfaz en español e inglés, conmutable en caliente.
- Modo fixtures (datos simulados, sin ejecutar binarios) para probar la UI sin herramientas instaladas.

## Requisitos

- Linux x86_64 para el binario precompilado (estático, sin dependencias). En otras plataformas, compilá desde el código fuente.
- Las herramientas de seguridad que quieras usar, instaladas y en el `PATH`. La app detecta cuáles están presentes y te guía para instalar las que falten.
- Algunas herramientas (captura wireless, sockets crudos) requieren `root` o capabilities; la app corre los binarios con tus privilegios.

## Instalación

### Script de instalación (recomendado)

Descarga el binario de la última release desde GitHub, verifica su checksum SHA-256 y lo instala en `/usr/local/bin` (usa `sudo` si hace falta):

```sh
curl -fsSL https://raw.githubusercontent.com/savagz/vox-security-tui/main/install.sh | sh
```

Requiere `curl` o `wget`, `tar` y `sha256sum`.

Opciones (se pasan con `sh -s --`):

| Opción             | Descripción                                               |
|--------------------|-----------------------------------------------------------|
| `--prefix <dir>`   | Directorio de instalación (por defecto `/usr/local/bin`)  |
| `--version <tag>`  | Versión a instalar, p. ej. `v1.0.0` (por defecto la última) |
| `--uninstall`      | Elimina el binario instalado                              |
| `-h`, `--help`     | Muestra la ayuda                                          |

La variable de entorno `VOXSECURITY_VERSION` equivale a `--version`.

Instalar una versión concreta sin `sudo`, en el directorio del usuario:

```sh
curl -fsSL https://raw.githubusercontent.com/savagz/vox-security-tui/main/install.sh \
  | sh -s -- --version v1.0.0 --prefix "$HOME/.local/bin"
```

Desinstalar (usá el mismo `--prefix` con el que instalaste):

```sh
curl -fsSL https://raw.githubusercontent.com/savagz/vox-security-tui/main/install.sh | sh -s -- --uninstall
```

### Descarga manual

Cada [release](https://github.com/savagz/vox-security-tui/releases) incluye `voxsecurity-tui-<versión>-x86_64-linux.tar.gz` y su `.sha256`:

```sh
v=v1.0.0
base=https://github.com/savagz/vox-security-tui/releases/download/$v
curl -fsSLO "$base/voxsecurity-tui-$v-x86_64-linux.tar.gz"
curl -fsSLO "$base/voxsecurity-tui-$v-x86_64-linux.tar.gz.sha256"
sha256sum -c "voxsecurity-tui-$v-x86_64-linux.tar.gz.sha256"
tar -xzf "voxsecurity-tui-$v-x86_64-linux.tar.gz"
sudo install -m 0755 "voxsecurity-tui-$v-x86_64-linux/voxsecurity-tui" /usr/local/bin/
```

### Desde el código fuente

Requiere un toolchain de Rust con soporte para la edición 2024.

```sh
git clone https://github.com/savagz/vox-security-tui
cd vox-security-tui
cargo run --release
```

Para compilar sólo el binario:

```sh
cargo build --release
./target/release/voxsecurity-tui
```

### Línea de comandos

```
voxsecurity-tui [OPCIONES]

    -V, --version    Muestra la versión y sale
    -h, --help       Muestra la ayuda y sale
```

### Modo fixtures

`VOXSECURITY_FIXTURES=1` fuerza datos simulados; las corridas no ejecutan binarios reales. Se activa solo en macOS cuando no hay ninguna herramienta del catálogo instalada.

```sh
VOXSECURITY_FIXTURES=1 cargo run
```

## Pestañas

| Tecla | Pestaña        |
|-------|----------------|
| `1`   | Resumen        |
| `2`   | Catálogo       |
| `3`   | Ejecutar       |
| `4`   | Historial      |
| `5`   | Verificación   |

`Tab` y `Shift+Tab` recorren las pestañas.

## Atajos

Pulsá `h` en cualquier momento para la ayuda en pantalla. Los más usados:

| Tecla                   | Acción                                      |
|-------------------------|---------------------------------------------|
| `q`, `Ctrl+C`           | Salir                                       |
| `j`/`k`, flechas        | Bajar / subir                               |
| `g`/`G`, `Home`/`End`   | Ir al inicio / fin                          |
| `r`                     | Re-detectar herramientas                    |
| `c`, `F1`               | Abrir configuración                         |
| `h`                     | Ayuda                                       |
| `/`                     | Filtrar el catálogo                         |
| `i`                     | Detalle de la herramienta (Catálogo)        |
| `I`                     | Instalar la herramienta (Catálogo)          |
| `Enter`                 | Elegir herramienta / abrir corrida          |
| `n`                     | Configurar y lanzar una corrida (Ejecutar)  |
| `R`                     | Repetir la última corrida (Ejecutar)        |
| `x`                     | Limpiar la salida (Ejecutar)                |

## Configuración

VOXSecurityTUI funciona sin archivo de configuración. Los ajustes se editan desde el overlay que abre `c` y se aplican al instante. El archivo se busca en:

1. `config/voxsecurity.toml` relativo al directorio actual (modo dev, ignorado por git).
2. `config.toml` en el directorio de config del usuario (resuelto por [`directories`](https://crates.io/crates/directories)).

Ver [`config/voxsecurity.example.toml`](config/voxsecurity.example.toml).

## Logs

Los logs se escriben en un archivo diario `voxsecurity-tui.log.<AAAA-MM-DD>` en el directorio de datos del usuario (`~/.local/share/voxsecurity-tui/` en Linux), nunca a la terminal. Se registran el inicio y fin de sesión (versión, usuario), cada corrida con su herramienta, objetivo, comando, si es destructiva, código de salida, duración y timeout, las instalaciones de paquetes y los errores al leer o guardar la configuración. El nivel por defecto es `info`; cambialo con `RUST_LOG`:

```sh
RUST_LOG=debug cargo run
```

## Desarrollo

```sh
cargo test
cargo clippy
```

## Estructura del proyecto

```
src/
  main.rs        Punto de entrada, loop de eventos, despacho de acciones
  app.rs         Estado de la aplicación y manejo de teclas
  config.rs      Carga y guardado de configuración
  deps.rs        Chequeos del sistema (gestor de paquetes, privilegios)
  event.rs       Eventos de la aplicación
  i18n.rs        Textos en español e inglés
  keys.rs        Mapeo de teclas a acciones
  poller.rs      Re-detección periódica de herramientas
  setup.rs       Estado de la verificación del sistema
  theme.rs       Paleta y glifos del lenguaje visual
  tools/         Backend: catálogo, detección, ejecución (runner)
  sysmon/        Métricas del host
  ui/            Render: pestañas y widgets
```
