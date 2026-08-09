#!/usr/bin/env python3
"""Regenera las capturas de pantalla de los manuales, en los dos idiomas.

Toma la aplicación real (../index.html), le inyecta un guion que fija el idioma
y deja la vista preparada, y la fotografía con un navegador Chromium en modo
headless. Deja `capturas/es/` para LEEME.md y `capturas/en/` para README.md.

    python capturas/generar.py

No hace falta instalar nada: usa el Edge o el Chrome que ya tengas.

---

Regenerates the manual screenshots in both languages. Takes the real app,
injects a small script that sets the language and prepares each view, and
photographs it with headless Chromium. Writes `capturas/en/` for README.md and
`capturas/es/` for LEEME.md. No install needed: it uses whatever Edge or Chrome
you already have.
"""
import shutil
import subprocess
import sys
from pathlib import Path

AQUI = Path(__file__).resolve().parent
REPO = AQUI.parent
TMP = AQUI / ".tmp"

CANDIDATOS = [
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "google-chrome", "chromium", "chromium-browser", "microsoft-edge",
]

# En una foto fija, un latido a media respiración solo se vería apagado:
# se congela en su punto alto.
CSS_FIJO = """
<style>
  .etiqueta.late, .parada.late .marca { animation: none !important; opacity: 1 !important; }
  * { caret-color: transparent !important; }
</style>
"""

# El modo demostración avisa de que no guarda nada; en el manual queremos
# enseñar la aplicación tal y como se ve en uso normal, con su carpeta.
# El idioma se fija aquí y se rehacen los carretes de ejemplo, porque su texto
# libre (notas, laboratorio, fechas) también está traducido.
COMO_EN_USO = """
    lang = '@IDIOMA@'; aplicaIdioma(); adopta(datosDemo());
    demo = false; conectado = true;
    carpeta = {name: lang === 'es' ? 'Carretes' : 'Rolls'};
    ultimoIndicador = 'ok';
    document.getElementById('aviso').hidden = true;
    window.scrollTo(0, 0);
"""

TOMAS = [
    dict(nombre="01-general", ancho=1440, alto=1240, ancla="#demo",
         guion=COMO_EN_USO + "vista = 'camara'; pinta();"),
    dict(nombre="02-la-via", ancho=1440, alto=300, ancla="#demo",
         guion=COMO_EN_USO + "vista = 'camara'; pinta();"),
    dict(nombre="03-ficha", ancho=1440, alto=900, ancla="#demo",
         guion=COMO_EN_USO + """
        vista = 'camara'; pinta();
        abreFicha(est.carretes.find(r => r.codigo === 'Leica01').id);
        document.getElementById('m-pelicula').blur();
    """),
    # agrupado por estado la lista mide 1522 px: se corta a 900 para que el
    # recorte caiga en el hueco entre grupos y no a media fila
    dict(nombre="04-por-estado", ancho=1440, alto=900, ancla="#demo",
         guion=COMO_EN_USO + "vista = 'estado'; pinta();"),
    dict(nombre="05-claro", ancho=1440, alto=900, ancla="#demo",
         guion=COMO_EN_USO + "ponTema('claro'); vista = 'camara'; pinta();"),
    # Chromium headless impone una ventana mínima de 492 px: por debajo maqueta
    # a 492 y recorta la foto. Se piden 500 para que no se corte nada.
    dict(nombre="06-movil", ancho=500, alto=1000, ancla="#demo",
         guion=COMO_EN_USO + "vista = 'camara'; pinta();"),
    # La bienvenida se pinta tras consultar el almacén del navegador, que es
    # asíncrono y no llega a tiempo: se pide directamente.
    dict(nombre="07-primer-arranque", ancho=1440, alto=560, ancla="", guion="""
        lang = '@IDIOMA@'; aplicaIdioma();
        tarjeta({
          texto: t('conPrimera'),
          botones: [{txt: t('conElegir'), principal: true, fn: function(){}}],
          mini: t('conMini'),
          saltar: t('conSaltar')
        });
    """),
]

IDIOMAS = ["es", "en"]


def busca_navegador():
    for c in CANDIDATOS:
        if Path(c).exists():
            return c
        hallado = shutil.which(c)
        if hallado:
            return hallado
    sys.exit("No he encontrado Edge ni Chrome. Instala uno, o edita CANDIDATOS.")


def prepara(toma, fuente, idioma):
    guion = toma["guion"].replace("@IDIOMA@", idioma)
    inyeccion = CSS_FIJO
    if guion:
        inyeccion += "<script>\ntry{\n" + guion + "\n}catch(e){console.error(e)}\n</script>\n"
    destino = TMP / f"{idioma}-{toma['nombre']}.html"
    destino.write_text(fuente.replace("</body>", inyeccion + "</body>"), encoding="utf-8")
    return destino


def dispara(navegador, toma, fuente, idioma):
    origen = prepara(toma, fuente, idioma)
    carpeta = AQUI / idioma
    carpeta.mkdir(parents=True, exist_ok=True)
    salida = carpeta / (toma["nombre"] + ".png")
    salida.unlink(missing_ok=True)
    perfil = TMP / ("perfil-" + idioma + "-" + toma["nombre"])
    cmd = [
        navegador, "--headless=new", "--disable-gpu", "--no-first-run", "--hide-scrollbars",
        "--force-device-scale-factor=2", "--disable-lcd-text",
        f"--window-size={toma['ancho']},{toma['alto']}",
        f"--user-data-dir={perfil}",
        "--virtual-time-budget=6000",
        f"--screenshot={salida}", origen.as_uri() + toma["ancla"],
    ]
    subprocess.run(cmd, capture_output=True, text=True, timeout=180)
    if salida.exists():
        print(f"  OK  {idioma}/{toma['nombre']:<20} {toma['ancho']}x{toma['alto']} -> {salida.stat().st_size // 1024} KB")
        return True
    print(f"  MAL {idioma}/{toma['nombre']}")
    return False


def main():
    navegador = busca_navegador()
    fuente = (REPO / "index.html").read_text(encoding="utf-8")
    TMP.mkdir(parents=True, exist_ok=True)
    print(f"Navegador: {navegador}")
    print(f"Generando {len(TOMAS) * len(IDIOMAS)} capturas en {AQUI}")
    fallos = 0
    for idioma in IDIOMAS:
        for toma in TOMAS:
            if not dispara(navegador, toma, fuente, idioma):
                fallos += 1
    shutil.rmtree(TMP, ignore_errors=True)
    total = sum(p.stat().st_size for i in IDIOMAS for p in (AQUI / i).glob("*.png"))
    hechas = len(TOMAS) * len(IDIOMAS) - fallos
    print(f"\n{hechas}/{len(TOMAS) * len(IDIOMAS)} capturas · {total // 1024} KB")
    return 1 if fallos else 0


if __name__ == "__main__":
    sys.exit(main())
