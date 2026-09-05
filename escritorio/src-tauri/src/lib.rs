//! Almendros de escritorio. Los módulos y sus contratos están en ../ARQUITECTURA.md.
pub mod almacen;
pub mod servidor;
pub mod comandos;

use almacen::Almacen;
use tauri::http::Response;
use tauri::Builder;
use tauri::Manager;

fn responde_protocolo(
    contexto: tauri::UriSchemeContext<'_, tauri::Wry>,
    peticion: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    let almacen = contexto.app_handle().state::<Almacen>();
    let ruta = peticion.uri().path();
    let respuesta = servidor::responde(&almacen, ruta);

    Response::builder()
        .status(respuesta.estado)
        .header("Content-Type", respuesta.tipo)
        .body(respuesta.cuerpo)
        .expect("respuesta con estado y Content-Type constantes")
}

fn crea_ventana(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::CustomProtocol("almendros://localhost/index.html".parse()?),
    )
    .title("Almendros")
    .inner_size(1280.0, 860.0)
    .min_inner_size(720.0, 520.0)
    .center()
    .disable_drag_drop_handler()
    .build()?;

    Ok(())
}

/// Punto de entrada de la aplicación. Inicializa Tauri, registra el protocolo,
/// crea la ventana principal y expone los comandos.
pub fn run() {
    let exe_path = std::env::current_exe().map_err(|e| format!("Error obteniendo ruta del ejecutable: {}", e));

    let exe_path = match exe_path {
        Ok(path) => path,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let almacen = match Almacen::desde_ejecutable(&exe_path) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let resultado = Builder::default()
        .manage(almacen)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .register_uri_scheme_protocol("almendros", responde_protocolo)
        .setup(crea_ventana)
        .invoke_handler(tauri::generate_handler![
            comandos::nombre_carpeta,
            comandos::existe,
            comandos::lee_texto,
            comandos::escribe_texto,
            comandos::lista_carpeta,
            comandos::borra_archivo,
            comandos::exporta_texto,
            comandos::abre_enlace,
        ])
        .run(tauri::generate_context!());

    match resultado {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error al ejecutar Tauri: {}", e);
            std::process::exit(1);
        }
    }
}
