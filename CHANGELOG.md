# Changelog

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) · [SemVer](https://semver.org/).

## [1.0] — 2026-09-05

First public release. Almendros grew out of a personal film log its author had been using
and rewriting since 2026-07-28, rebuilt to be useful to any film photographer.

### The log
- A roll carries its code, camera, film stock, push/pull, format and exposure count; how it
  was developed — lab and order number, or developer, dilution, time and temperature; notes,
  where the negative is filed, and a link to the scans.
- Seven stages, each with its colour, from Loaded to Filed. Clicking a roll's label advances
  it; rolls developed at home skip "Picked up". Undo for status changes, deleted rolls and
  removed cameras.
- Dates are free text on purpose (`22/06/26`, `july`, `before the trip?`).
- Search across every field, grouping by camera or by status, and a next-code suggestion per
  camera.

### Your data
- One readable `almendros.json` in a folder you choose, with a daily backup in `copias/`
  (last 30 kept), written through the File System Access API.
- **Roll-by-roll merging** on every write and resync, so a folder shared over a NAS survives
  two computers editing at once. Browser-only fallback where folders can't be written.
- Download a `.json` copy, open or drag one in (it merges, never replaces), export to `.csv`,
  and print a clean sheet for the lab. Reads and migrates two earlier file formats.

### On the desktop
- **Almendros for the desktop** (`escritorio/`): a small Tauri app for Windows and macOS that
  opens the same `index.html` in its own window and writes the folder directly — no File
  System Access API and no browser permission prompt at every start. The Windows binary is
  attached to this release, unsigned; the macOS one you build yourself with Rust and the
  Tauri CLI.

### Interface and design
- English and Spanish, auto-detected and switchable — `?lang=en` or `?lang=es` — without ever
  changing the data file. Dark and light themes, keyboard shortcuts, accessible labels and
  `prefers-reduced-motion` support.
- Darkroom vocabulary rather than generic web app: a perforated spine down the left edge,
  rolls as solid colour frames forming a continuous strip, technical lab typography. No
  rounded corners, no shadows, no blur.

### Under the hood
One `index.html`, ~97 KB, zero dependencies, no build step, no network at runtime. 21 tests
run in the page at `#pruebas`; `escritorio/pruebas/costura.py` covers the desktop seam and
`cargo test` the Rust side.
