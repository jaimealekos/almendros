# Arquitectura · Architecture

*Español primero; English below.*

Todo el programa vive en **`index.html`**: `<style>`, marcado y un único `<script>` vanilla.
Sin dependencias externas, sin compilación, sin peticiones de red. Tiene que funcionar
abierto desde `file://` con el wifi apagado — eso condiciona cada decisión que sigue.

## Modelo de datos

```json
{
  "app":"almendros", "version":2, "porVersion":"1.0.0",
  "actualizado":"2026-08-10T04:00:00.000Z",
  "ajustes":{"flujo":"lab"},
  "camaras":["Hasel","Yashica"],
  "camarasMod":{"Hasel":"2026-08-01T..."},
  "carretes":[ … ],
  "papelera":[{"k":"r19","mod":"…"}, {"k":"cam:Nikon","mod":"…"}]
}
```

`version` es la del **formato de archivo**; `porVersion`, la de la **aplicación**. No se
mueven juntas: el formato solo sube si cambia el esquema.

Un carrete tiene `id`, `mod` (fecha ISO de su última modificación) y los 20 campos de
`CAMPOS`. **Todos son cadenas de texto, siempre.**

- **Las fechas son texto libre a propósito** («22/06/26», «julio», «¿antes del viaje?»).
  No convertirlas nunca a selectores de fecha: es una decisión de producto.
- **Las claves internas van en español**, que es el idioma fundacional del proyecto:
  `estado` guarda `Revelando`, no `Developing`. La interfaz traduce al pintar y el archivo
  no cambia con el idioma, así que un registro escrito en inglés se abre idéntico en
  español.
- `flujo` vale `lab` o `casa`, y decide qué campos se enseñan y por qué estados pasa el
  carrete. En `casa` no existe `Recogido`.
- `camarasMod` guarda cuándo se dio de alta cada cámara: sin esa fecha no se puede saber si
  un alta es posterior a un borrado, y la fusión fallaría.

## Las cuatro piezas del motor

**`normaliza(d)`** — única puerta de entrada. Acepta los tres formatos que han existido
(lista suelta, `{camaras, carretes}` y el actual), rellena lo que falte, sella `mod` y funde
las cámaras de los carretes con la lista. Todo lo que entra —archivo, caché, importación—
pasa por aquí.

**`fusiona(a, b)`** — combina dos copias del archivo sin perder trabajo. Gana el carrete con
`mod` mayor; si hay marca en la papelera posterior, desaparece; si se editó *después* del
borrado, resucita (correcto: alguien lo recuperó). Las cámaras se unen salvo que su marca
`cam:Nombre` sea posterior a su alta, y una cámara con carretes vivos nunca se pierde. Las
fechas ISO en UTC se comparan bien como texto: de ahí el formato.

**`escribeArchivo()`** — **lee, fusiona y escribe**, en ese orden. Nunca vuelca el estado en
memoria a pelo. Después adopta lo fusionado, salvo con la ficha abierta para no cambiarle
los campos al usuario a media edición; la siguiente escritura lo recupera.

**`resincroniza()`** — al recuperar el foco y al recibir aviso por `BroadcastChannel`.
**Nunca corre con una escritura pendiente**: los diálogos del sistema devuelven el foco justo
antes de que el cambio llegue al disco, y releer en ese instante resucitaría lo recién
borrado. Fue un fallo real y hoy hay una prueba que lo cubre.

## Invariantes — no romper

- Ni dependencias externas ni peticiones de red: `index.html` debe bastar.
- No cambiar sin migración las claves de `localStorage`, la base `almendros-kv` de IndexedDB
  ni los nombres de archivo y de la carpeta de copias.
- Todo dato que entra pasa por `normaliza()`; todo dato que sale pasa por
  `escribeArchivo()`.
- Las claves de `ESTADOS` y `CAMPOS` son datos, no texto de interfaz: no se traducen.
- Al tocar `index.html`, subir `VERSION`: el aviso de pestaña vieja depende de ello.
- Toda clave usada con `data-t` o `data-tp` debe existir en los **dos** diccionarios.
- Las animaciones son lentas y se apagan con `prefers-reduced-motion`.

## Lenguaje visual — «la tira»

Vocabulario de tira de película, deliberadamente distinto al de una aplicación web al uso:

- **El lomo**: canto perforado fijo a la izquierda, a toda altura, con el nombre en vertical.
  Es la identidad y el ancla de la maquetación; no hay cabecera horizontal con logotipo.
- **La vía**: los estados en línea, en orden de proceso, con el recuento por fase. Hace de
  leyenda, de filtro y de resumen a la vez — por eso no hay una frase de estadísticas.
- **La tira**: un bloque de color macizo por carrete; seguidos forman una cinta continua de
  fotogramas separados por el filete de cada fila.
- **Nada redondeado, nada con sombra, nada desenfocado.** La textura viene de la estructura:
  filetes, perforaciones y tipografía.
- Negro frío y **naranja de máscara de negativo**, que significa siempre *atención*: marca,
  acción primaria y estado «Revelar». No usarlo para nada decorativo.
- Rótulos en minúscula monoespaciada, como un impreso técnico; los apartados de la ficha van
  numerados. La ficha es un cajón lateral, no una ventana centrada.

## Cómo probar

- **`index.html#pruebas`** — 21 comprobaciones sobre las funciones puras: fusión,
  migraciones, numeración de códigos, escapado del CSV y comparación de versiones. Si tocas
  esas partes, añade la prueba ahí mismo.
- **`index.html#demo`** — carretes de ejemplo en memoria; no guarda nada.
- El selector de carpeta **no se puede automatizar**: exige un gesto real y el diálogo del
  sistema. Para probar el guardado, sustituye `leeArchivo`/`escribeArchivo` por versiones
  contra una variable en memoria; así se verifican la fusión, los borrados y las carreras
  de foco.
- **`python capturas/generar.py`** rehace las capturas del README con Chromium headless.
  Dos avisos: el headless de Brave se cuelga (usa Edge o Chrome) e impone una ventana
  mínima de 492 px de ancho — por debajo maqueta a 492 y recorta la foto.

## Descubribilidad

El `<title>` del archivo es largo y bilingüe a propósito, porque lo leen los buscadores;
`aplicaIdioma()` lo sustituye al cargar por uno corto y localizado. Las etiquetas Open Graph
y la canónica apuntan a la demo alojada: si el repositorio cambia de nombre o de dueño, hay
que actualizarlas.

---

# Architecture (English)

The whole program lives in **`index.html`**: styles, markup and one vanilla script. No
dependencies, no build step, no network at runtime — it has to work from `file://` with the
wifi off, and that constrains everything below.

**Data.** One JSON file, `{version, ajustes, camaras, camarasMod, carretes, papelera}`.
Every roll carries an `id`, a `mod` timestamp and 20 string fields. Dates are free text by
design. **Internal keys are in Spanish**, the project's founding language — the interface
translates them on screen, so the file never changes with the interface language.

**Four moving parts.** `normaliza()` is the only entry point for data and accepts all three
historical formats. `fusiona()` merges two copies of the file roll by roll: newest `mod`
wins, tombstones remove, and an edit later than a deletion resurrects. `escribeArchivo()`
always **reads, merges and writes** — it never dumps memory over the file. `resincroniza()`
re-reads on focus, but **never while a write is pending**: native dialogs return focus just
before the change reaches disk, and re-reading there would resurrect what was just deleted.

**Invariants.** No dependencies, no network. Storage keys, database names and file names
don't change without a migration. `ESTADOS` and `CAMPOS` keys are data, not UI text. Bump
`VERSION` whenever you touch the file — the stale-tab warning depends on it. Every `data-t`
key must exist in **both** dictionaries.

**Testing.** `index.html#pruebas` runs 21 in-page tests over the pure functions; add yours
there. `index.html#demo` loads sample rolls in memory. The folder picker can't be automated,
so test saving by swapping `leeArchivo`/`escribeArchivo` for in-memory versions.
`python capturas/generar.py` regenerates the README screenshots (use Edge or Chrome —
headless Brave hangs — and note the 492 px minimum window width).
