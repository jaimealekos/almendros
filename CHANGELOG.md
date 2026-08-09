# Changelog

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) · [SemVer](https://semver.org/).

## [1.0.1] — 2026-08-10

### Added
- **Language in the address**: `?lang=en` or `?lang=es` opens the app in that language and
  remembers the choice. Each manual links to its own version of the app, so an English
  reader never lands on a Spanish interface (and the other way round).

### Changed
- The manuals are now two complete, mirrored documents — `README.md` in English and
  `LEEME.md` in Spanish — each with **its own set of screenshots** showing the interface in
  that language. A language switch is the first thing on both.

## [1.0.0] — 2026-08-10

First public release. Almendros grew out of a personal film log its author had been using
and rewriting since 2026-07-28, rebuilt to be useful to any film photographer.

### The roll
- Code, camera, film stock, push/pull, format and exposure count.
- Free-text dates on purpose (`22/06/26`, `july`, `before the trip?`), with a `today` button.
- Notes, physical storage location of the negative, and a link to the scans.

### Two developing paths
- **Lab** — lab name and order number.
- **At home** — developer, dilution, time and temperature, kept with the roll.
- The card shows the fields that match, and the lifecycle adapts: a roll developed at home
  never waits to be "picked up".

### Lifecycle
- Seven states — Loaded, To develop, Developing, Developed, Picked up, Scanned, Filed —
  each with its colour, and the two "waiting on the lab" states breathing slowly.
- A status track across the top doubles as legend, filter and per-stage count.
- Click a roll's status label to advance it, without opening anything.
- Undo for deletions, status changes and removed cameras, in place of browser dialogs.

### Your data
- One readable `almendros.json` in a folder you choose, with a daily backup in `copias/`
  (last 30 kept). Written through the File System Access API.
- **Roll-by-roll merging** on every write and resync: each roll carries a modification
  timestamp and each deletion leaves a tombstone, so two computers sharing a folder over a
  NAS don't overwrite each other.
- Stale-tab warning, via the version stamped in the file and a BroadcastChannel between tabs.
- Browser-only mode for browsers that can't write to folders (Firefox, Safari, phones),
  with a reminder to download a copy.
- Download a `.json` copy, open a file (merges, never replaces), drag a `.json` onto the
  window, and export to a spreadsheet (`.csv`, separator by language, BOM for Excel).
- Reads and migrates two earlier file formats. If the chosen folder already contains
  `_bbdd/negativos.json`, it keeps writing there.

### Interface
- English and Spanish, auto-detected and switchable; the data file never changes language.
- Dark and light themes. Print stylesheet for a clean lab sheet.
- Search across every field; group by camera or by status.
- Keyboard shortcuts: `N`, `/`, `Esc`, `Ctrl+Enter`.
- Next-code suggestion per camera and inheritance from the previous roll.
- Accessible labels, keyboard navigation on rows and status labels, and
  `prefers-reduced-motion` support.

### Design
Darkroom vocabulary, deliberately unlike a generic web app: a perforated spine down the left
edge carrying the name vertically, rolls as solid colour frames forming a continuous strip,
oversized condensed camera names on heavy rules, filled rectangular status labels, and
technical lab typography. No rounded corners, no shadows, no blur.

### Under the hood
- One `index.html`, ~92 KB, zero dependencies, no build step, no network at runtime.
- `#pruebas` runs 21 built-in tests in the page (merge logic, migrations from every
  historical format, code numbering, CSV escaping, version comparison).
- `#demo` loads sample rolls in memory and saves nothing.
