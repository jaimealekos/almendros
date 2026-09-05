# Almendros de escritorio

Un ejecutable pequeño para Windows y macOS que abre el `index.html` de Almendros en una
ventana propia, con el motor web del sistema, y escribe en su carpeta directamente. Sin
barra de direcciones, sin pestañas y —lo que importa— **sin el permiso de carpeta que el
navegador pide en cada arranque**.

## ⬇ Descargar

**[`Almendros.exe`](https://github.com/jaimealekos/almendros/releases/latest/download/Almendros.exe)** — Windows, 9 MB.

**En esta carpeta solo está el código fuente.** El ejecutable no vive en el árbol de
archivos —los binarios no entran en git— sino en
[la última versión publicada](https://github.com/jaimealekos/almendros/releases/latest).
Para macOS hay que compilarlo: [más abajo](#compilar).

## Cómo se usa

Pon `Almendros.exe` (o `Almendros.app`) en la carpeta donde viven `index.html` y tus datos, y
ábrelo. Eso es todo: lee y escribe ahí mismo, con el mismo archivo, el mismo formato y las
mismas copias diarias que en el navegador. Actualizar la aplicación sigue siendo copiar
`index.html`.

No está firmado: Windows avisará de «editor desconocido» la primera vez.

## Cómo funciona

- **Tauri 2**, sin Node y sin empaquetar nada: el ejecutable no lleva `index.html` dentro,
  lo lee del disco al lado suyo.
- Registra el esquema propio `almendros` y abre `almendros://localhost/index.html`. El
  protocolo **solo sirve ese archivo**; cualquier otra ruta es 404, así que el JSON de datos
  nunca viaja por él. El origen es estable, que es lo que hace persistir `localStorage`.
- Expone ocho comandos a la página: `nombre_carpeta`, `existe`, `lee_texto`,
  `escribe_texto`, `lista_carpeta`, `borra_archivo`, `exporta_texto` (diálogo nativo de
  guardar, para la copia `.json` y el CSV) y `abre_enlace` (los escaneos, en el navegador
  del sistema). Todos los de archivo reciben una ruta **relativa** a la carpeta de datos y
  rechazan la vacía, la absoluta y cualquiera con `..`, `.` o `\`. La escritura es atómica:
  archivo temporal y renombrado.
- La carpeta de datos es la del ejecutable; en macOS, la que contiene `Almendros.app`.
- `index.html` lo detecta por `window.__TAURI__` y desvía por ahí el acceso a la carpeta,
  las descargas y los enlaces externos. **La costura es aditiva:** sin ejecutable, cada
  función conserva intacta su rama de navegador y la aplicación se comporta exactamente
  igual. No convertirla en un `if` que sustituya la rama vieja.

Los módulos son `src/almacen.rs` (resolver la carpeta y la entrada/salida acotada),
`src/servidor.rs` (qué responde el protocolo), `src/comandos.rs` (los ocho comandos) y
`src/lib.rs` (arranque y ventana). El modelo de datos y los invariantes de la aplicación
están en [`../ARQUITECTURA.md`](../ARQUITECTURA.md).

## Compilar

**Windows**, una vez. Hacen falta las Build Tools de Visual Studio con «Desktop development
with C++»; la cadena GNU no compila WebView2, así que se fija MSVC para esta carpeta:

```
rustup toolchain install stable-x86_64-pc-windows-msvc
rustup override set stable-x86_64-pc-windows-msvc
cargo install tauri-cli --version ^2 --locked
```

**macOS**, una vez: `xcode-select --install`, Rust por rustup y el mismo `cargo install`.

Y ya, desde esta carpeta:

```
cargo tauri build --no-bundle
```

El ejecutable queda en `src-tauri/target/release/`. En macOS, `cargo tauri build` sin más
deja el paquete en `src-tauri/target/release/bundle/macos/`. **Sin probar todavía en macOS.**

Los iconos se rehacen con `cargo tauri icon icono.svg -o src-tauri/icons`.

## Probar

- `cargo test` en `src-tauri` — la carpeta acotada y el protocolo, sin red.
- `python pruebas/costura.py` — la costura de `index.html` contra un anfitrión falso en
  memoria, con Chrome headless. El Edge de algunas máquinas no vuelca el DOM con
  `--dump-dom`; por eso prueba Chrome primero.
- El selector de carpeta y los diálogos nativos no se pueden automatizar: eso se mira a
  mano.

---

# Almendros on the desktop (English)

A small executable for Windows and macOS that opens Almendros' `index.html` in its own
window, using the system's web engine, and writes to its own folder directly — no address
bar, no tabs and no folder permission prompt at every start.

**Download [`Almendros.exe`](https://github.com/jaimealekos/almendros/releases/latest/download/Almendros.exe)** — Windows, 9 MB.
**This folder holds the source only**: the binary lives in
[the latest release](https://github.com/jaimealekos/almendros/releases/latest),
not in the file tree. Build the macOS one yourself.

**Use it** by putting `Almendros.exe` (or `Almendros.app`) in the folder where `index.html`
and your data live, and opening it. Same file, same format, same daily backups. It isn't
signed, so Windows warns about an unknown publisher the first time.

**How it works.** Tauri 2, no Node, nothing bundled: the executable reads the `index.html`
sitting next to it. It registers the `almendros` scheme and serves **only that file** —
anything else is a 404, so the data JSON never travels over it. It exposes eight commands to
the page (folder name, exists, read, write, list, delete, a native save dialog for exports,
and http(s) links through the system browser). Every file command takes a path **relative**
to the data folder and rejects empty, absolute, `..`, `.` and `\`; writes are atomic.
`index.html` detects it through `window.__TAURI__`. **The seam is additive:** without the
executable every function keeps its browser branch and nothing changes.

**Build** with `cargo tauri build --no-bundle` from this folder. On Windows you need the
Visual Studio Build Tools and the MSVC toolchain (`rustup override set
stable-x86_64-pc-windows-msvc`) — the GNU chain can't build WebView2. On macOS, the Xcode
command line tools. **Not tested on macOS yet.**

**Test** with `cargo test` in `src-tauri` and `python pruebas/costura.py`.
