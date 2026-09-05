#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Prueba de costura: integración de Tauri en index.html con aserciones reales.

Arranca Edge/Chrome en modo headless contra copias temporales de index.html con
un `window.__TAURI__` falso inyectado, deja que las aserciones (también
inyectadas) hagan su trabajo y comprueben el resultado en document.title, y lo
lee con --dump-dom. No hace falta instalar nada ni levantar Tauri de verdad.

    python escritorio/pruebas/costura.py
"""
import html
import json
import os
import re
import shutil
import subprocess
import sys
import io
from datetime import datetime, timedelta, timezone
from pathlib import Path

# Forzar UTF-8 en stdout para que los símbolos ✓/✗ se vean bien en Windows.
if sys.stdout.encoding != 'utf-8':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

# Usar getcwd (no __file__) evita que Path.resolve() intente normalizar una
# ruta UNC de red y se cuelgue; ver memoria del proyecto sobre generar.py.
AQUI = Path(os.getcwd()) / "escritorio" / "pruebas"
REPO = Path(os.getcwd())
TMP = AQUI / ".tmp"

# Chrome primero: en algunos Edge (152.0.4191.62 comprobado) --dump-dom no
# escribe nada en stdout dentro de este entorno (sale con éxito y sin volcar
# el DOM, seguramente por alguna política de empresa), mientras que --screenshot
# en ese mismo Edge sí funciona. capturas/generar.py no lo nota porque usa
# --screenshot; esta prueba sí depende de leer stdout, así que evita ese Edge.
CANDIDATOS = [
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    "google-chrome", "chromium", "chromium-browser",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "microsoft-edge",
]

ANCHO_VENTANA, ALTO_VENTANA = 1280, 860
PRESUPUESTO_TIEMPO_MS = 8000      # tiempo virtual que se le da a la página
TIEMPO_LIMITE_PROCESO_S = 60      # límite real de espera al proceso del navegador

SIN_TITULO = '✗ sin título (¿no arrancó?)'
TIEMPO_AGOTADO = '✗ tiempo agotado'
PREFIJO_OK = '✓'

FECHA_SEMILLA = "2026-01-01T00:00:00.000Z"
LEGADO_RUTA = "_bbdd/negativos.json"
DIAS_COPIAS_ANTIGUAS = 32          # cuántas copias diarias se siembran de más
LIMITE_COPIAS = 30                 # debe coincidir con COPIAS_MAX en index.html

ESCENARIOS = ["pruebas", "navegador", "vacia", "existente", "legado", "copias-muchas"]

# Mismo orden que CAMPOS en index.html; solo importa para sembrar carretes de prueba.
CAMPOS_TEXTO = [
    "codigo", "camara", "pelicula", "iso", "formato", "exposiciones", "estado", "flujo",
    "laboratorio", "pedido", "revelador", "dilucion", "tiempo", "temperatura",
    "carga", "fin", "revelado", "archivo", "escaneos", "notas",
]


def busca_navegador():
    for candidato in CANDIDATOS:
        if Path(candidato).exists():
            return candidato
        hallado = shutil.which(candidato)
        if hallado:
            return hallado
    sys.exit("No he encontrado Edge ni Chrome. Instala uno, o edita CANDIDATOS.")


def escape_js(valor):
    """Codifica un valor Python como literal JavaScript seguro (JSON es un subconjunto de JS)."""
    return json.dumps(valor)


def estructura_vacia():
    """Estructura mínima válida, la misma forma que `normaliza()` da a una carpeta vacía."""
    return {
        "app": "almendros",
        "version": 2,
        # Nunca por encima de VERSION: si no, la aplicación cantaría «pestaña vieja»
        # y el aviso fijo estorbaría a las comprobaciones sobre #nota.
        "porVersion": "1.0",
        "actualizado": FECHA_SEMILLA,
        "ajustes": {"flujo": "lab"},
        "camaras": [],
        "camarasMod": {},
        "carretes": [],
        "papelera": [],
    }


def carrete_semilla(id_carrete, codigo, camara):
    """Un carrete con los campos de texto vacíos, igual que los crea la propia aplicación."""
    carrete = {campo: "" for campo in CAMPOS_TEXTO}
    carrete.update(id=id_carrete, mod=FECHA_SEMILLA, codigo=codigo, camara=camara,
                   estado="Cargado", flujo="lab")
    return carrete


def datos_con_un_carrete(id_carrete, codigo, camara):
    """Estructura de datos sembrada con un único carrete, para arrancar con carpeta no vacía."""
    datos = estructura_vacia()
    datos["carretes"] = [carrete_semilla(id_carrete, codigo, camara)]
    datos["camaras"] = [camara]
    datos["camarasMod"] = {camara: FECHA_SEMILLA}
    return datos


def texto_json(datos):
    return json.dumps(datos, indent=1)


def fecha_utc_hoy():
    """Fecha de hoy en UTC: index.html calcula la suya con `toISOString().slice(0,10)`."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%d")


def fecha_utc_hace(dias):
    return (datetime.now(timezone.utc) - timedelta(days=dias)).strftime("%Y-%m-%d")


def semillas_escenario(escenario):
    """Contenido inicial del sistema de archivos falso, según el escenario."""
    if escenario == "existente":
        return {"almendros.json": texto_json(datos_con_un_carrete("r1", "Hasel01", "Hasel"))}
    if escenario == "legado":
        return {LEGADO_RUTA: texto_json(datos_con_un_carrete("n1", "Negativos01", "Negativos"))}
    if escenario == "copias-muchas":
        semillas = {"almendros.json": texto_json(datos_con_un_carrete("r1", "Test01", "Test"))}
        for dias in range(1, DIAS_COPIAS_ANTIGUAS + 1):
            nombre = f"copias/almendros-{fecha_utc_hace(dias)}.json"
            semillas[nombre] = texto_json(estructura_vacia())
        return semillas
    return {}


def js_fs_semillas(semillas):
    """Genera las llamadas fs.set(...) que precargan el sistema de archivos falso."""
    return "".join(f"fs.set({escape_js(ruta)}, {escape_js(texto)});\n" for ruta, texto in semillas.items())


PLANTILLA_TAURI_FALSO = r"""
<script>
(function(){
  const fs = new Map();
  window.__fs = fs;
  window.__llamadas = [];
  window.__exporta = true;

  FS_INIT_AQUI

  const COMANDOS_CON_RUTA = ['existe', 'lee_texto', 'escribe_texto', 'lista_carpeta', 'borra_archivo'];

  // La misma validación que Almacen::ruta en Rust, en todos los comandos de archivo:
  // si la costura construyera una ruta inválida, aquí debe fallar igual que en el ejecutable.
  function valida(relativa) {
    if(!relativa) throw new Error('relativa vacía');
    if(relativa.startsWith('/')) throw new Error('relativa empieza por /');
    if(relativa.includes('..')) throw new Error('relativa tiene ..');
    if(relativa.includes('\\')) throw new Error('relativa tiene \\');
    if(relativa.includes('//')) throw new Error('relativa tiene //');
    if(relativa.split('/').includes('.')) throw new Error('relativa tiene .');
  }

  async function invoke(cmd, args) {
    window.__llamadas.push({cmd, args});
    if(!cmd) throw new Error('cmd undefined');
    if(COMANDOS_CON_RUTA.includes(cmd)) valida(args.relativa);
    if(cmd === 'nombre_carpeta') return 'CarpetaDatos';
    if(cmd === 'existe') {
      return fs.has(args.relativa) || [...fs.keys()].some(k => k.startsWith(args.relativa + '/'));
    }
    if(cmd === 'lee_texto') {
      if(!fs.has(args.relativa)) throw new Error('No existe ' + args.relativa);
      return fs.get(args.relativa);
    }
    if(cmd === 'escribe_texto') { fs.set(args.relativa, args.texto); return; }
    if(cmd === 'lista_carpeta') {
      const prefijo = args.relativa + '/';
      const nombres = [];
      for(const [ruta] of fs) {
        if(ruta.startsWith(prefijo)) {
          const resto = ruta.slice(prefijo.length);
          if(!resto.includes('/')) nombres.push(resto);
        }
      }
      return nombres.sort();
    }
    if(cmd === 'borra_archivo') {
      if(!fs.has(args.relativa)) throw new Error('No existe');
      fs.delete(args.relativa);
      return;
    }
    if(cmd === 'exporta_texto') { return window.__exporta; }
    if(cmd === 'abre_enlace') { return; }
    throw new Error('Comando desconocido ' + cmd);
  }

  window.__TAURI__ = { core: { invoke } };
})();
</script>
"""


def crea_tauri_falso(semillas):
    """Objeto `window.__TAURI__` falso: sistema de archivos en memoria y registro de llamadas."""
    return PLANTILLA_TAURI_FALSO.replace("FS_INIT_AQUI", js_fs_semillas(semillas))


# El escenario "navegador" no debe fingir Tauri, pero conectar() consulta la
# IndexedDB real para recordar la carpeta (kv('get','carpeta')) y esa
# petición nunca resuelve bajo --virtual-time-budget: no es un temporizador
# de la página, sino una ida y vuelta al proceso navegador que el reloj
# virtual no impulsa, así que la página se queda esperando para siempre y
# nunca llega a pideConexion(). Se sustituye por una IndexedDB mínima en
# memoria, solo para desatascar esa espera; no toca __TAURI__ ni escritorio.
INDEXEDDB_FALSA = r"""
<script>
(function(){
  // window.indexedDB solo tiene getter en el navegador real: una asignación
  // normal no hace nada (se queda con el objeto nativo). Hace falta
  // reemplazar la propiedad entera con defineProperty.
  Object.defineProperty(window, 'indexedDB', {configurable: true, writable: true, value: {
    open: function(){
      const peticion = {};
      Promise.resolve().then(() => {
        peticion.result = { transaction: () => ({ objectStore: () => ({ get: () => {
          const resultado = {};
          Promise.resolve().then(() => { resultado.result = undefined; if(resultado.onsuccess) resultado.onsuccess(); });
          return resultado;
        }})})};
        if(peticion.onsuccess) peticion.onsuccess();
      });
      return peticion;
    }
  }});
})();
</script>
"""


# ── aserciones ──────────────────────────────────────────────────────────
# `ARNES_JS` se repite en todos los escenarios: temporizador de espera,
# contador de comprobaciones y los ayudantes que pide la especificación
# (carretePrueba y el alta de la cámara "Prueba").
ARNES_JS = r"""
// persiste() tarda 250 ms en escribir de verdad; estos márgenes dan tiempo
// de sobra a que arrancar, guardar o exportar terminen antes de comprobar.
const ESPERA_ARRANQUE_MS = 500;
const ESPERA_ESCRITURA_MS = 600;
const ESPERA_AVISO_MS = 100;
const espera = ms => new Promise(r => setTimeout(r, ms));
const fallos = [];
let total = 0;
const comprueba = (cond, mensaje) => { total++; if(!cond) fallos.push(mensaje); };
function carretePrueba(id, codigo){
  const r = {id, mod: ahora()};
  for(const c of CAMPOS) r[c] = '';
  r.codigo = codigo;
  r.camara = 'Prueba';
  r.estado = 'Cargado';
  r.flujo = 'lab';
  return r;
}
function anadeCarretePrueba(id, codigo){
  est.carretes.push(carretePrueba(id, codigo));
  if(!est.camaras.includes('Prueba')){
    est.camaras.push('Prueba');
    est.camarasMod['Prueba'] = ahora();
  }
}
"""

PLANTILLA_ASERCIONES = """
<script>
ARNES_AQUI
(async () => {
  try {
CUERPO_AQUI
  } catch(e) {
    fallos.push('excepción: ' + e.message);
  }
  document.title = fallos.length ? '✗ ' + fallos.join(' | ') : '✓ ' + total + '/' + total;
})();
</script>
"""


def envoltorio_aserciones(cuerpo_js):
    """Boilerplate común: ejecuta el cuerpo del escenario y deja el resultado en <title>."""
    return PLANTILLA_ASERCIONES.replace("ARNES_AQUI", ARNES_JS).replace("CUERPO_AQUI", cuerpo_js)


def con_constantes(cuerpo_js, valores):
    """Antepone declaraciones `const NOMBRE = valor;` (codificadas en JSON) al cuerpo dado."""
    declaraciones = "".join(f"const {nombre} = {escape_js(valor)};\n" for nombre, valor in valores.items())
    return declaraciones + cuerpo_js


CUERPO_NAVEGADOR = r"""
await espera(ESPERA_ARRANQUE_MS);
comprueba(escritorio === null, 'escritorio !== null');
comprueba(typeof window.__TAURI__ === 'undefined', 'TAURI definido');
comprueba(!!$('conexion') && !$('conexion').hidden, 'conexion no visible');
"""

# Conectar con una carpeta vacía dispara un persiste() automático que crea
# almendros.json (con 0 carretes) 250 ms después de arrancar. Si se esperase
# más que eso antes de añadir el carrete de prueba, esa escritura ya estaría
# en el disco falso y la siguiente crearía una copia diaria de la nada. Por
# eso aquí se espera lo mínimo (un macrotask) en vez de ESPERA_ARRANQUE_MS:
# basta para que se resuelvan las promesas de arranque, que no usan temporizador.
CUERPO_VACIA = r"""
await espera(0);
comprueba(conectado === true, 'conectado !== true');
comprueba($('conexion').hidden === true, 'conexion.hidden !== true');
comprueba($('pie-carpeta').textContent === 'CarpetaDatos/', 'pie-carpeta incorrecto');

anadeCarretePrueba('p9', 'Prueba09');
persiste();
await espera(ESPERA_ESCRITURA_MS);

comprueba(window.__fs.has('almendros.json'), 'almendros.json no existe');
try {
  const datos = JSON.parse(window.__fs.get('almendros.json'));
  comprueba(datos.carretes.length === 1, 'carretes.length !== 1');
  normaliza(datos);
} catch(e) {
  fallos.push('JSON parse/normaliza: ' + e.message);
}

const copias = [...window.__fs.keys()].filter(k => k.startsWith('copias/'));
comprueba(copias.length === 0, 'existen copias sin estado previo');
const clavesLegado = [...window.__fs.keys()].filter(k => k.startsWith('_bbdd/'));
comprueba(clavesLegado.length === 0, 'existen claves _bbdd/');

pintaMenu();
const itemsMenu = [...document.querySelectorAll('#menu .item')].map(i => i.textContent);
comprueba(!itemsMenu.includes(t('mCambiarCarpeta')), 'mCambiarCarpeta en menú');
comprueba(!itemsMenu.includes(t('mConectar')), 'mConectar en menú');

window.__exporta = false;
await descargaCopia();
await espera(ESPERA_AVISO_MS);
let ultima = window.__llamadas[window.__llamadas.length - 1];
comprueba(ultima.cmd === 'exporta_texto', 'exporta sin permiso no llamó a exporta_texto');
comprueba(/^almendros-\d{4}-\d{2}-\d{2}\.json$/.test(ultima.args.nombre),
  'nombre exporta incorrecto: ' + ultima.args.nombre);
comprueba(!$('nota').classList.contains('visible'), 'nota visible cuando no debe');

window.__exporta = true;
await descargaCopia();
await espera(ESPERA_AVISO_MS);
comprueba($('nota').classList.contains('visible'), 'nota no visible cuando debe');
comprueba($('nota').querySelector('span.txt').textContent === t('copiaHecha'), 'nota.txt incorrecto');

await descargaCsv();
await espera(ESPERA_AVISO_MS);
ultima = window.__llamadas[window.__llamadas.length - 1];
comprueba(ultima.cmd === 'exporta_texto', 'CSV no llamó a exporta_texto');
comprueba(ultima.args.nombre.endsWith('.csv'), 'nombre CSV no acaba en .csv');

const enlace = document.createElement('a');
enlace.target = '_blank';
enlace.href = 'https://ejemplo.test/x';
document.body.appendChild(enlace);
const clicEnlace = new MouseEvent('click', {bubbles: true, cancelable: true});
enlace.dispatchEvent(clicEnlace);
comprueba(clicEnlace.defaultPrevented === true, 'defaultPrevented !== true');
const llamadaAbre = window.__llamadas.find(l => l.cmd === 'abre_enlace');
comprueba(!!llamadaAbre, 'no hay llamada a abre_enlace');
comprueba(!!llamadaAbre && llamadaAbre.args.url === 'https://ejemplo.test/x', 'url de abre_enlace incorrecta');
"""

CUERPO_EXISTENTE = r"""
await espera(ESPERA_ARRANQUE_MS);
comprueba(est.carretes.length === 1, 'carretes.length !== 1 al arrancar');

anadeCarretePrueba('p10', 'Prueba10');
persiste();
await espera(ESPERA_ESCRITURA_MS);

try {
  const datos = JSON.parse(window.__fs.get('almendros.json'));
  comprueba(datos.carretes.length === 2, 'carretes después de guardar !== 2');
} catch(e) {
  fallos.push('JSON read: ' + e.message);
}

const claveCopia = 'copias/almendros-' + HOY + '.json';
comprueba(window.__fs.has(claveCopia), 'copia diaria no existe');
comprueba(window.__fs.get(claveCopia) === TEXTO_SEMBRADO, 'la copia diaria no es igual al texto sembrado');
"""

CUERPO_LEGADO = r"""
await espera(ESPERA_ARRANQUE_MS);
comprueba(est.carretes.length === 1, 'carretes.length !== 1 al arrancar');

anadeCarretePrueba('p1', 'Prueba01');
persiste();
await espera(ESPERA_ESCRITURA_MS);

try {
  const datos = JSON.parse(window.__fs.get('_bbdd/negativos.json'));
  comprueba(datos.carretes.length === 2, 'negativos.json carretes !== 2');
} catch(e) {
  fallos.push('negativos read: ' + e.message);
}

const claveBackup = '_backups/negativos-' + HOY + '.json';
comprueba(window.__fs.has(claveBackup), 'backup diario no existe');
comprueba(!window.__fs.has('almendros.json'), 'almendros.json existe en legado');
"""

CUERPO_COPIAS_MUCHAS = r"""
await espera(ESPERA_ARRANQUE_MS);

anadeCarretePrueba('p1', 'Prueba01');
persiste();
await espera(ESPERA_ESCRITURA_MS);

const patronCopia = /^almendros-\d{4}-\d{2}-\d{2}\.json$/;
const copiasValidas = [...window.__fs.keys()]
  .filter(k => k.startsWith('copias/') && patronCopia.test(k.slice('copias/'.length)))
  .sort();
comprueba(copiasValidas.length === LIMITE_COPIAS,
  'copias válidas !== ' + LIMITE_COPIAS + ', tiene ' + copiasValidas.length);

const fechaHaceDias = dias => {
  const d = new Date();
  d.setUTCDate(d.getUTCDate() - dias);
  return d.toISOString().slice(0, 10);
};
const nombresPosibles = [];
for(let dias = 0; dias <= DIAS_COPIAS_ANTIGUAS; dias++){
  nombresPosibles.push('copias/almendros-' + fechaHaceDias(dias) + '.json');
}
nombresPosibles.sort();
const ultimasValidas = nombresPosibles.slice(-LIMITE_COPIAS);
comprueba(JSON.stringify(copiasValidas) === JSON.stringify(ultimasValidas),
  'las copias no son las ' + LIMITE_COPIAS + ' más recientes');
"""


def crea_aserciones(escenario, semillas):
    """Construye el <script> de aserciones del escenario, o "" si no le hace falta."""
    if escenario == "pruebas":
        return ""
    if escenario == "navegador":
        return envoltorio_aserciones(CUERPO_NAVEGADOR)
    if escenario == "vacia":
        return envoltorio_aserciones(CUERPO_VACIA)
    if escenario == "existente":
        cuerpo = con_constantes(CUERPO_EXISTENTE,
                                 {"HOY": fecha_utc_hoy(), "TEXTO_SEMBRADO": semillas["almendros.json"]})
        return envoltorio_aserciones(cuerpo)
    if escenario == "legado":
        cuerpo = con_constantes(CUERPO_LEGADO, {"HOY": fecha_utc_hoy()})
        return envoltorio_aserciones(cuerpo)
    if escenario == "copias-muchas":
        cuerpo = con_constantes(CUERPO_COPIAS_MUCHAS,
                                 {"DIAS_COPIAS_ANTIGUAS": DIAS_COPIAS_ANTIGUAS, "LIMITE_COPIAS": LIMITE_COPIAS})
        return envoltorio_aserciones(cuerpo)
    raise ValueError(f"escenario desconocido: {escenario}")


# ── preparación del HTML y arranque del navegador ──────────────────────
def inserta_antes_del_script(html_fuente, fragmento):
    """Coloca `fragmento` justo antes de la única etiqueta <script> de la app."""
    indice = html_fuente.index("<script>")
    return html_fuente[:indice] + fragmento + html_fuente[indice:]


def prepara_html(html_fuente, escenario, semillas):
    """Devuelve el HTML de partida con el Tauri falso y las aserciones ya inyectados."""
    if escenario == "pruebas":
        return html_fuente
    html_mod = html_fuente
    if escenario == "navegador":                     # aquí NO se finge Tauri, ver INDEXEDDB_FALSA
        html_mod = inserta_antes_del_script(html_mod, INDEXEDDB_FALSA)
    else:
        html_mod = inserta_antes_del_script(html_mod, crea_tauri_falso(semillas))
    aserciones = crea_aserciones(escenario, semillas)
    return html_mod.replace("</body>", aserciones + "</body>", 1)


def ruta_html_temporal(escenario):
    """Ruta del HTML ya preparado para el escenario (lo escribe `prepara_escenarios`)."""
    return TMP / f"costura-{escenario}.html"


def prepara_escenarios(html_fuente, escenarios):
    """Escribe en disco el HTML de cada escenario que necesita inyección."""
    TMP.mkdir(parents=True, exist_ok=True)
    for escenario in escenarios:
        if escenario == "pruebas":                   # este carga index.html tal cual
            continue
        semillas = semillas_escenario(escenario)
        contenido = prepara_html(html_fuente, escenario, semillas)
        ruta_html_temporal(escenario).write_text(contenido, encoding="utf-8")


def uri_escenario(escenario):
    """URI que debe cargar el navegador para este escenario."""
    if escenario == "pruebas":
        return (REPO / "index.html").as_uri() + "#pruebas"
    return ruta_html_temporal(escenario).as_uri()


def extrae_titulo(salida_html):
    """Saca el texto de <title> del DOM volcado por --dump-dom, o None si no aparece."""
    coincidencia = re.search(r"<title>(.*?)</title>", salida_html, re.S)
    return html.unescape(coincidencia.group(1)) if coincidencia else None


def dispara(escenario):
    """Arranca el navegador contra el HTML del escenario y devuelve el <title> final.

    Con --dump-dom la salida es el DOM ya resuelto: como las aserciones dejan
    su resultado en document.title al terminar, basta con leerlo de ahí.
    """
    navegador = busca_navegador()
    orden = [
        navegador,
        "--headless=new",
        "--disable-gpu",
        "--no-first-run",
        "--hide-scrollbars",
        f"--window-size={ANCHO_VENTANA},{ALTO_VENTANA}",
        f"--virtual-time-budget={PRESUPUESTO_TIEMPO_MS}",
        "--dump-dom",
        uri_escenario(escenario),
    ]
    try:
        resultado = subprocess.run(orden, capture_output=True, encoding="utf-8",
                                    errors="replace", timeout=TIEMPO_LIMITE_PROCESO_S)
    except subprocess.TimeoutExpired:
        return TIEMPO_AGOTADO
    titulo = extrae_titulo(resultado.stdout)
    return titulo if titulo is not None else SIN_TITULO


def main():
    html_fuente = (REPO / "index.html").read_text(encoding="utf-8")
    try:
        prepara_escenarios(html_fuente, ESCENARIOS)
        resultados = [(escenario, dispara(escenario)) for escenario in ESCENARIOS]
    finally:
        shutil.rmtree(TMP, ignore_errors=True)

    for escenario, titulo in resultados:
        marca = PREFIJO_OK if titulo.startswith(PREFIJO_OK) else '✗'
        print(f"{marca} {escenario} — {titulo}")

    todo_bien = all(titulo.startswith(PREFIJO_OK) for _, titulo in resultados)
    return 0 if todo_bien else 1


if __name__ == "__main__":
    sys.exit(main())
