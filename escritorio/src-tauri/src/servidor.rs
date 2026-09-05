//! M2 · servidor: decidir la respuesta a cada petición del protocolo almendros.
//!
//! Función pura sobre tipos propios, sin tauri ni http, para poder probarla sin arrancar nada.

use crate::almacen::Almacen;

pub struct Respuesta {
    pub estado: u16,            // 200 o 404
    pub tipo: &'static str,     // valor de Content-Type
    pub cuerpo: Vec<u8>,
}

const ESTADO_OK: u16 = 200;
const ESTADO_NO_ENCONTRADO: u16 = 404;
const TIPO_HTML: &str = "text/html; charset=utf-8";
const TIPO_TEXTO: &str = "text/plain; charset=utf-8";

const COLOR_FONDO: &str = "#090a0b";
const COLOR_TEXTO: &str = "#eef1f2";
const COLOR_ACENTO: &str = "#ff5b14";

/// Escapa caracteres especiales de HTML. El nombre de la carpeta lo elige el usuario
/// y puede contener cualquier carácter que deba visualizarse correctamente en HTML.
/// Reemplaza `&` primero para evitar reescapar.
fn escapa_html(texto: &str) -> String {
    texto
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
}

/// `ruta` es la parte de camino de la URL, sin consulta ni fragmento (`/index.html`).
/// `/` y `/index.html` sirven el index.html de la carpeta de datos con
/// `text/html; charset=utf-8`. Si falta, 404 con una página HTML (español e inglés) que
/// dice que hay que copiar `index.html` junto al programa e indica la carpeta. Cualquier
/// otra ruta: 404 `text/plain; charset=utf-8` con el texto «No existe».
pub fn responde(almacen: &Almacen, ruta: &str) -> Respuesta {
    if ruta == "/" || ruta == "/index.html" {
        return sirve_index(almacen);
    }
    Respuesta {
        estado: ESTADO_NO_ENCONTRADO,
        tipo: TIPO_TEXTO,
        cuerpo: "No existe".as_bytes().to_vec(),
    }
}

fn sirve_index(almacen: &Almacen) -> Respuesta {
    match almacen.lee_texto("index.html") {
        Ok(contenido) => Respuesta {
            estado: ESTADO_OK,
            tipo: TIPO_HTML,
            cuerpo: contenido.into_bytes(),
        },
        Err(_) => Respuesta {
            estado: ESTADO_NO_ENCONTRADO,
            tipo: TIPO_HTML,
            cuerpo: genera_error_404(&almacen.nombre()).into_bytes(),
        },
    }
}

fn genera_error_404(nombre_carpeta: &str) -> String {
    let nombre_escapado = escapa_html(nombre_carpeta);
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Almendros - archivo no encontrado</title>
    <style>
        body {{
            background-color: {fondo};
            color: {texto};
            font-family: monospace;
            margin: 0;
            padding: 40px;
        }}
        h1 {{
            color: {acento};
            margin-top: 0;
        }}
        .folder {{
            color: {acento};
        }}
        .section {{
            margin: 20px 0;
        }}
    </style>
</head>
<body>
    <h1>Almendros</h1>

    <div class="section">
        <p><strong>Español:</strong></p>
        <p>No se encontró <code>index.html</code>.</p>
        <p>Copia el archivo <code>index.html</code> junto a este programa en la carpeta:</p>
        <p><span class="folder">{}</span></p>
    </div>

    <div class="section">
        <p><strong>English:</strong></p>
        <p><code>index.html</code> not found.</p>
        <p>Copy the <code>index.html</code> file next to this program in the folder:</p>
        <p><span class="folder">{}</span></p>
    </div>
</body>
</html>"#,
        nombre_escapado,
        nombre_escapado,
        fondo = COLOR_FONDO,
        texto = COLOR_TEXTO,
        acento = COLOR_ACENTO,
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::fs;

    struct CarpetaTemporal(std::path::PathBuf);

    impl CarpetaTemporal {
        fn nueva() -> Self {
            static CONTADOR: AtomicUsize = AtomicUsize::new(0);
            let pid = std::process::id();
            let n = CONTADOR.fetch_add(1, Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos();
            let temp = std::env::temp_dir();
            let carpeta = temp.join(format!("almendros-servidor-{}-{}-{}", pid, n, nanos));
            let _ = fs::create_dir_all(&carpeta);
            CarpetaTemporal(carpeta)
        }

        fn ruta(&self) -> &std::path::PathBuf {
            &self.0
        }
    }

    impl Drop for CarpetaTemporal {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn responde_index_html_presente() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let contenido_esperado = "<!DOCTYPE html>\n<html>\n<body>Test</body>\n</html>";
        almacen
            .escribe_texto("index.html", contenido_esperado)
            .unwrap();

        let respuesta = responde(&almacen, "/index.html");

        assert_eq!(respuesta.estado, 200);
        assert_eq!(respuesta.tipo, "text/html; charset=utf-8");
        assert_eq!(respuesta.cuerpo, contenido_esperado.as_bytes());
    }

    #[test]
    fn responde_raiz_sirve_index() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let contenido_esperado = "<!DOCTYPE html>\n<html>\n<body>Test</body>\n</html>";
        almacen
            .escribe_texto("index.html", contenido_esperado)
            .unwrap();

        let respuesta = responde(&almacen, "/");

        assert_eq!(respuesta.estado, 200);
        assert_eq!(respuesta.tipo, "text/html; charset=utf-8");
        assert_eq!(respuesta.cuerpo, contenido_esperado.as_bytes());
    }

    #[test]
    fn responde_sin_index_html() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let respuesta = responde(&almacen, "/index.html");

        assert_eq!(respuesta.estado, 404);
        assert_eq!(respuesta.tipo, "text/html; charset=utf-8");
        let cuerpo_str = String::from_utf8_lossy(&respuesta.cuerpo);
        assert!(cuerpo_str.contains("index.html"));
        assert!(cuerpo_str.contains(almacen.nombre().as_str()));
    }

    #[test]
    fn responde_sin_index_html_raiz() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let respuesta = responde(&almacen, "/");

        assert_eq!(respuesta.estado, 404);
        assert_eq!(respuesta.tipo, "text/html; charset=utf-8");
        let cuerpo_str = String::from_utf8_lossy(&respuesta.cuerpo);
        assert!(cuerpo_str.contains("index.html"));
        assert!(cuerpo_str.contains(almacen.nombre().as_str()));
    }

    #[test]
    fn responde_otra_ruta_404_texto() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let respuesta = responde(&almacen, "/otra.html");

        assert_eq!(respuesta.estado, 404);
        assert_eq!(respuesta.tipo, "text/plain; charset=utf-8");
        assert_eq!(respuesta.cuerpo, "No existe".as_bytes());
    }

    #[test]
    fn responde_json_datos_nunca_se_sirve() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let json_contenido = r#"{"carretes": []}"#;
        almacen
            .escribe_texto("_bbdd/negativos.json", json_contenido)
            .unwrap();

        let respuesta = responde(&almacen, "/_bbdd/negativos.json");

        assert_eq!(respuesta.estado, 404);
        assert_eq!(respuesta.tipo, "text/plain; charset=utf-8");
        assert_eq!(respuesta.cuerpo, "No existe".as_bytes());
        let cuerpo_str = String::from_utf8_lossy(&respuesta.cuerpo);
        assert!(!cuerpo_str.contains("carretes"));
    }

    #[test]
    fn responde_path_traversal_404() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let contenido = "<!DOCTYPE html><html></html>";
        almacen
            .escribe_texto("index.html", contenido)
            .unwrap();

        let respuesta = responde(&almacen, "/../index.html");

        assert_eq!(respuesta.estado, 404);
        assert_eq!(respuesta.tipo, "text/plain; charset=utf-8");
        assert_eq!(respuesta.cuerpo, "No existe".as_bytes());
    }

    #[test]
    fn responde_acentos_en_index() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let contenido = "<!DOCTYPE html><html><body>Café con leche, comillas: «ejemplo»</body></html>";
        almacen
            .escribe_texto("index.html", contenido)
            .unwrap();

        let respuesta = responde(&almacen, "/index.html");

        assert_eq!(respuesta.estado, 200);
        let cuerpo_str = String::from_utf8_lossy(&respuesta.cuerpo);
        assert!(cuerpo_str.contains("Café"));
        assert!(cuerpo_str.contains("«ejemplo»"));
    }

    #[test]
    fn responde_error_404_contiene_español_e_ingles() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let respuesta = responde(&almacen, "/index.html");

        let cuerpo_str = String::from_utf8_lossy(&respuesta.cuerpo);
        assert!(cuerpo_str.contains("Español"));
        assert!(cuerpo_str.contains("English"));
        assert!(cuerpo_str.contains("No se encontró"));
        assert!(cuerpo_str.contains("not found"));
    }

    #[test]
    fn escapa_html_reemplaza_caracteres_especiales() {
        let resultado = escapa_html("a<b>&\"c\"");
        assert_eq!(resultado, "a&lt;b&gt;&amp;&quot;c&quot;");
    }

    #[test]
    fn responde_error_404_escapa_el_nombre_de_la_carpeta() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let respuesta = responde(&almacen, "/index.html");

        let cuerpo_str = String::from_utf8_lossy(&respuesta.cuerpo);
        let nombre = almacen.nombre();

        // El nombre debe estar presente (escapado si es necesario)
        let nombre_escapado = escapa_html(&nombre);
        assert!(cuerpo_str.contains(&nombre_escapado));

        // Si el nombre contiene &, debe estar escapado como &amp; dentro del span
        if nombre.contains("&") {
            assert!(cuerpo_str.contains("&amp;"));
            // Verificar que no hay & sin escapar dentro de un span class="folder"
            if let Some(start) = cuerpo_str.find(r#"<span class="folder">"#) {
                if let Some(end) = cuerpo_str[start..].find("</span>") {
                    let span_content = &cuerpo_str[start..start + end + 7];
                    // Dentro del span no debe haber & sin escapar
                    let after_tag = &span_content[r#"<span class="folder">"#.len()..];
                    let before_close = &after_tag[..after_tag.len() - "</span>".len()];
                    assert!(!before_close.contains("&"), "Encontrado & sin escapar en: {}", before_close);
                }
            }
        }
    }
}
