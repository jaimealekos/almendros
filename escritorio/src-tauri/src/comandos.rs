//! M4 · comandos: ocho funciones que JS invoca para interactuar con el sistema de archivos y enlaces.

use crate::almacen::Almacen;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

/// Devuelve el nombre de la carpeta de datos.
#[tauri::command]
pub fn nombre_carpeta(almacen: tauri::State<'_, Almacen>) -> String {
    almacen.nombre()
}

/// Comprueba si existe un archivo en la carpeta de datos.
#[tauri::command]
pub fn existe(almacen: tauri::State<'_, Almacen>, relativa: String) -> Result<bool, String> {
    almacen.existe(&relativa)
}

/// Lee el contenido de texto de un archivo en la carpeta de datos.
#[tauri::command]
pub fn lee_texto(almacen: tauri::State<'_, Almacen>, relativa: String) -> Result<String, String> {
    almacen.lee_texto(&relativa)
}

/// Escribe texto en un archivo en la carpeta de datos.
#[tauri::command]
pub fn escribe_texto(
    almacen: tauri::State<'_, Almacen>,
    relativa: String,
    texto: String,
) -> Result<(), String> {
    almacen.escribe_texto(&relativa, &texto)
}

/// Lista las entradas de una carpeta dentro de la carpeta de datos.
#[tauri::command]
pub fn lista_carpeta(almacen: tauri::State<'_, Almacen>, relativa: String) -> Result<Vec<String>, String> {
    almacen.lista(&relativa)
}

/// Borra un archivo en la carpeta de datos.
#[tauri::command]
pub fn borra_archivo(almacen: tauri::State<'_, Almacen>, relativa: String) -> Result<(), String> {
    almacen.borra(&relativa)
}

/// Abre un diálogo nativo de guardar y escribe el texto en la ruta elegida.
/// Solo funciona con rutas locales (no URL de red). Si el usuario cancela, devuelve Ok(false).
#[tauri::command]
pub async fn exporta_texto(
    app: tauri::AppHandle,
    nombre: String,
    texto: String,
) -> Result<bool, String> {
    let (descripcion, extension) = extraer_extension(&nombre);

    let mut archivo = app
        .dialog()
        .file()
        .set_file_name(&nombre);

    if let (Some(desc), Some(ext)) = (descripcion, extension) {
        archivo = archivo.add_filter(desc, &[ext]);
    }

    match archivo.blocking_save_file() {
        Some(ruta_file_path) => {
            let ruta_path = ruta_file_path
                .into_path()
                .map_err(|e| format!("Error al convertir ruta: {}", e))?;
            std::fs::write(&ruta_path, &texto)
                .map_err(|e| format!("No se puede escribir en «{}»: {}", ruta_path.display(), e))?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Extrae extensión del nombre de archivo y devuelve (descripción, extensión).
/// La descripción es 'static porque se retorna directamente para el filtro.
fn extraer_extension(nombre: &str) -> (Option<&'static str>, Option<&str>) {
    if let Some(punto) = nombre.rfind('.') {
        let ext = &nombre[punto + 1..];
        let ext_lower = ext.to_lowercase();
        let descripcion = match ext_lower.as_str() {
            "json" => Some("JSON"),
            "csv" => Some("CSV"),
            _ => None,
        };
        (descripcion, Some(ext))
    } else {
        (None, None)
    }
}

/// Abre una URL en el navegador por defecto. Solo acepta http:// y https://.
#[tauri::command]
pub fn abre_enlace(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let url_lower = url.to_lowercase();
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        return Err("Solo se abren enlaces http o https".to_string());
    }

    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| e.to_string())
}
