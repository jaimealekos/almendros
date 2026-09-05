//! M1 · almacen: resolver la carpeta de datos y hacer entrada/salida de texto acotada a ella.
//!
//! Resuelve la carpeta de datos a partir del ejecutable y proporciona métodos para leer,
//! escribir y listar archivos dentro de ella de forma segura.

use std::fs;
use std::path::{Path, PathBuf};

pub struct Almacen {
    raiz: PathBuf,
}

impl Almacen {
    /// Carpeta de datos a partir de la ruta del ejecutable. Si algún antecesor termina en
    /// «.app» (macOS), es la carpeta que contiene ese paquete; si no, la del ejecutable.
    /// Err si la ruta no tiene carpeta padre.
    pub fn desde_ejecutable(ejecutable: &Path) -> Result<Almacen, String> {
        let carpeta = ejecutable.parent().ok_or_else(|| {
            "La ruta del ejecutable no tiene carpeta padre".to_string()
        })?;

        // Sin cfg: la regla .app se prueba incluso en Windows.
        let raiz = carpeta
            .ancestors()
            .find(|a| a.extension().is_some_and(|e| e == "app"))
            .and_then(|app_dir| app_dir.parent())
            .unwrap_or(carpeta);

        Ok(Almacen {
            raiz: raiz.to_path_buf(),
        })
    }

    /// Para pruebas y para construir sobre una carpeta concreta.
    pub fn en(raiz: PathBuf) -> Almacen {
        Almacen { raiz }
    }

    /// Nombre de la carpeta de datos (último componente), para enseñarlo en la interfaz.
    /// Si la raíz no tiene último componente (raíz de unidad como C:\ o /), devuelve la ruta completa.
    pub fn nombre(&self) -> String {
        self.raiz
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.raiz.display().to_string())
    }

    /// Ruta absoluta de una relativa válida. Err si la relativa está vacía, es absoluta,
    /// contiene `.`, `..` o `\`, o tiene segmentos vacíos (`a//b`).
    /// La validación se hace por segmentos, no por canonicalize, porque este falla en rutas inexistentes.
    pub fn ruta(&self, relativa: &str) -> Result<PathBuf, String> {
        if relativa.is_empty() {
            return Err("La ruta relativa no puede estar vacía".to_string());
        }

        if relativa.starts_with('/') || relativa.starts_with('\\') || relativa.contains(':') {
            return Err(format!("La ruta no puede ser absoluta: «{}»", relativa));
        }

        let segmentos: Vec<&str> = relativa.split('/').collect();

        for segmento in &segmentos {
            if segmento.is_empty() {
                return Err(format!(
                    "La ruta contiene segmentos vacíos: «{}»",
                    relativa
                ));
            }
            if *segmento == "." || *segmento == ".." {
                return Err(format!(
                    "La ruta contiene . o ..: «{}»",
                    relativa
                ));
            }
            if segmento.contains('\\') {
                return Err(format!(
                    "La ruta contiene barra invertida (usar /): «{}»",
                    relativa
                ));
            }
        }

        let mut resultado = self.raiz.clone();
        for segmento in segmentos {
            resultado.push(segmento);
        }

        Ok(resultado)
    }

    pub fn existe(&self, relativa: &str) -> Result<bool, String> {
        let ruta = self.ruta(relativa)?;
        Ok(ruta.exists())
    }

    /// Err si no existe, no se puede leer o no es UTF-8. El mensaje incluye la ruta.
    pub fn lee_texto(&self, relativa: &str) -> Result<String, String> {
        let ruta = self.ruta(relativa)?;
        fs::read_to_string(&ruta).map_err(|err| {
            format!("No se puede leer «{}»: {}", ruta.display(), err)
        })
    }

    /// Crea las carpetas intermedias. Escribe en `<archivo>.tmp` y renombra encima.
    pub fn escribe_texto(&self, relativa: &str, texto: &str) -> Result<(), String> {
        let ruta = self.ruta(relativa)?;

        if let Some(padre) = ruta.parent() {
            fs::create_dir_all(padre).map_err(|err| {
                format!("No se puede crear la carpeta «{}»: {}", padre.display(), err)
            })?;
        }

        // El archivo temporal está en la misma carpeta para que fs::rename sea atómico dentro del volumen.
        let nombre_archivo = ruta.file_name().ok_or_else(|| {
            format!("La ruta «{}» no tiene nombre de archivo", ruta.display())
        })?;
        let mut archivo_tmp = ruta.clone();
        archivo_tmp.set_file_name(format!("{}.tmp", nombre_archivo.to_string_lossy()));

        fs::write(&archivo_tmp, texto).map_err(|err| {
            format!(
                "No se puede escribir en «{}»: {}",
                archivo_tmp.display(),
                err
            )
        })?;

        fs::rename(&archivo_tmp, &ruta).map_err(|err| {
            format!(
                "No se puede renombrar «{}» a «{}»: {}",
                archivo_tmp.display(),
                ruta.display(),
                err
            )
        })?;

        Ok(())
    }

    /// Nombres (no rutas) de las entradas de la carpeta, ordenados. Carpeta inexistente: lista vacía.
    pub fn lista(&self, relativa: &str) -> Result<Vec<String>, String> {
        let ruta = self.ruta(relativa)?;

        if !ruta.exists() {
            return Ok(Vec::new());
        }

        let mut nombres: Vec<String> = fs::read_dir(&ruta)
            .map_err(|err| {
                format!(
                    "No se puede leer la carpeta «{}»: {}",
                    ruta.display(),
                    err
                )
            })?
            .map(|entrada_result| {
                entrada_result
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .map_err(|err| {
                        format!(
                            "No se puede leer entrada en «{}»: {}",
                            ruta.display(),
                            err
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        nombres.sort();
        Ok(nombres)
    }

    /// Err si no existe o no se puede borrar.
    pub fn borra(&self, relativa: &str) -> Result<(), String> {
        let ruta = self.ruta(relativa)?;
        fs::remove_file(&ruta).map_err(|err| {
            format!("No se puede borrar «{}»: {}", ruta.display(), err)
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CarpetaTemporal(PathBuf);

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
            let carpeta = temp.join(format!("almendros-prueba-{}-{}-{}", pid, n, nanos));
            let _ = fs::create_dir_all(&carpeta);
            CarpetaTemporal(carpeta)
        }

        fn ruta(&self) -> &PathBuf {
            &self.0
        }
    }

    impl Drop for CarpetaTemporal {
        fn drop(&mut self) {
            // Es limpieza de prueba: si falla, no hay nada mejor que se pueda hacer.
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn desde_ejecutable_windows() {
        let ruta = Path::new("C:/x/NEGATIVOS/almendros.exe");
        let almacen = Almacen::desde_ejecutable(ruta).unwrap();
        assert_eq!(almacen.nombre(), "NEGATIVOS");
    }

    #[test]
    fn desde_ejecutable_macos_app() {
        let ruta = Path::new("/Users/j/Fotos/Almendros.app/Contents/MacOS/almendros");
        let almacen = Almacen::desde_ejecutable(ruta).unwrap();
        let ruta_str = almacen.raiz.to_string_lossy();
        assert!(ruta_str.ends_with("Fotos"));
    }

    #[test]
    fn ruta_acepta_simple() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let ruta = almacen.ruta("index.html").unwrap();
        assert!(ruta.to_string_lossy().ends_with("index.html"));
    }

    #[test]
    fn ruta_acepta_anidada() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let ruta = almacen.ruta("_bbdd/negativos.json").unwrap();
        let ruta_str = ruta.to_string_lossy();
        assert!(ruta_str.contains("_bbdd"));
        assert!(ruta_str.ends_with("negativos.json"));
    }

    #[test]
    fn ruta_rechaza_vacia() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(almacen.ruta("").is_err());
    }

    #[test]
    fn ruta_rechaza_padre() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(almacen.ruta("../x").is_err());
        assert!(almacen.ruta("a/../b").is_err());
    }

    #[test]
    fn ruta_rechaza_actual() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(almacen.ruta("./a").is_err());
        assert!(almacen.ruta("a/./b").is_err());
    }

    #[test]
    fn ruta_rechaza_absoluta_unix() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(almacen.ruta("/abs").is_err());
    }

    #[test]
    fn ruta_rechaza_absoluta_windows() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(almacen.ruta("C:/abs").is_err());
    }

    #[test]
    fn ruta_rechaza_barra_invertida() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(almacen.ruta("a\\b").is_err());
    }

    #[test]
    fn ruta_rechaza_segmentos_vacios() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(almacen.ruta("a//b").is_err());
    }

    #[test]
    fn escribe_y_lee_texto_acentos_comillas() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let texto_original = "Texto con acentos: café, comillas: «ejemplo»";
        almacen
            .escribe_texto("test.txt", texto_original)
            .unwrap();

        let texto_leído = almacen.lee_texto("test.txt").unwrap();
        assert_eq!(texto_leído, texto_original);
    }

    #[test]
    fn escribe_crea_carpetas_intermedias() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        almacen
            .escribe_texto("copias/archivo.txt", "contenido")
            .unwrap();

        assert!(almacen.existe("copias").unwrap());
        assert!(almacen.existe("copias/archivo.txt").unwrap());
    }

    #[test]
    fn escribe_reemplaza_contenido() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        almacen
            .escribe_texto("test.txt", "primer contenido")
            .unwrap();
        almacen
            .escribe_texto("test.txt", "segundo contenido")
            .unwrap();

        let texto = almacen.lee_texto("test.txt").unwrap();
        assert_eq!(texto, "segundo contenido");
    }

    #[test]
    fn escribe_no_deja_tmp() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        almacen.escribe_texto("test.txt", "contenido").unwrap();

        let archivos = almacen.lista("copias").unwrap_or_default();
        assert!(archivos.is_empty());
    }

    #[test]
    fn existe_archivo_presente() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        almacen.escribe_texto("existe.txt", "contenido").unwrap();

        assert!(almacen.existe("existe.txt").unwrap());
    }

    #[test]
    fn existe_archivo_ausente() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(!almacen.existe("no_existe.txt").unwrap());
    }

    #[test]
    fn existe_carpeta() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        almacen
            .escribe_texto("sub/archivo.txt", "contenido")
            .unwrap();

        assert!(almacen.existe("sub").unwrap());
    }

    #[test]
    fn lista_nombres_ordenados() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        almacen.escribe_texto("copias/z.txt", "").unwrap();
        almacen.escribe_texto("copias/a.txt", "").unwrap();
        almacen.escribe_texto("copias/m.txt", "").unwrap();

        let nombres = almacen.lista("copias").unwrap();
        assert_eq!(nombres, vec!["a.txt", "m.txt", "z.txt"]);
    }

    #[test]
    fn lista_carpeta_inexistente_vacia() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let nombres = almacen.lista("inexistente").unwrap();
        assert_eq!(nombres.len(), 0);
    }

    #[test]
    fn lista_vacia_error() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        assert!(almacen.lista("").is_err());
    }

    #[test]
    fn borra_archivo() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        almacen.escribe_texto("a_borrar.txt", "contenido").unwrap();
        assert!(almacen.existe("a_borrar.txt").unwrap());

        almacen.borra("a_borrar.txt").unwrap();
        assert!(!almacen.existe("a_borrar.txt").unwrap());
    }

    #[test]
    fn borra_inexistente_error() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let resultado = almacen.borra("no_existe.txt");
        assert!(resultado.is_err());
    }

    #[test]
    fn lee_inexistente_error_con_ruta() {
        let carpeta = CarpetaTemporal::nueva();
        let almacen = Almacen::en(carpeta.ruta().clone());

        let resultado = almacen.lee_texto("no_existe.txt");
        assert!(resultado.is_err());
        let error = resultado.unwrap_err();
        assert!(error.contains("no_existe.txt"));
    }

    #[test]
    fn nombre_negativos() {
        let carpeta = CarpetaTemporal::nueva();
        let negativos = carpeta.ruta().join("NEGATIVOS");
        fs::create_dir_all(&negativos).unwrap();

        let almacen = Almacen::en(negativos);
        assert_eq!(almacen.nombre(), "NEGATIVOS");
    }
}
