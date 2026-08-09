<p align="center">
  <img src="capturas/01-general.png" alt="Almendros: a film photography logbook showing rolls grouped by camera, with a colour-coded status track" width="900">
</p>

# 🎞️ Almendros — a film photography logbook in a single HTML file

**Track every roll of film you shoot** — which camera it's loaded in, what stock, push or
pull, whether it went to a lab or into your own tank, when it comes back, and *where the
negative ended up*. Almendros is a film log for analog (analogue) photographers that runs
from one HTML file and saves to a folder you own.

No account. No server. No subscription. No telemetry. Unlimited rolls.

<p>
  <a href="LICENSE"><img alt="MIT licence" src="https://img.shields.io/badge/licence-MIT-000?style=flat-square"></a>
  <img alt="Version 1.0.0" src="https://img.shields.io/badge/version-1.0.0-ff5b14?style=flat-square">
  <img alt="One file, 92 KB, no dependencies" src="https://img.shields.io/badge/one%20file-92%20KB%20·%20no%20dependencies-000?style=flat-square">
  <a href="https://jaimealekos.github.io/almendros/#demo"><img alt="Live demo" src="https://img.shields.io/badge/live%20demo-try%20it-ff5b14?style=flat-square"></a>
</p>

**[▶ Try the demo](https://jaimealekos.github.io/almendros/#demo)** (sample rolls, nothing
saved) · **[Open the app](https://jaimealekos.github.io/almendros/)** ·
**[Léeme en español](LEEME.md)**

---

## Before anything else — three honest limits

Most film logs hide their catch. Here are ours, up front:

1. **Saving to a folder needs a desktop browser**: Chrome, Edge or Brave on Windows, macOS
   or Linux. Firefox and Safari can't write to folders yet.
2. **On a phone it runs, but it can't save to a folder.** Everything stays inside the
   browser and you download a copy when you want one. It is a desk tool, not a pocket one.
3. **It does not log frame by frame.** No aperture and shutter speed for every shot, no GPS.
   [That's deliberate](#why-not-frame-by-frame).

If any of those is a dealbreaker, [there are good apps that do it differently](#other-tools-worth-knowing)
— honestly listed below.

---

## What it's for

You load a roll. You shoot it over three weeks. It sits in a drawer. You send it to the lab.
It comes back. You scan it. Two years later you want that one frame — and the negative is in
one of eleven sleeves with no markings.

Almendros is the thread through all of that. It answers the two questions every film
photographer asks:

- **"What's in this camera, and was I shooting it at box speed?"**
- **"Where the hell did I file that negative?"**

Ten seconds per roll. That's the whole commitment.

## The status track

The row of states along the top is the heart of it: your whole shelf at a glance, with a
count at every stage. Click a stage to see only those rolls.

![The status track with a count for every stage of the process](capturas/02-la-via.png)

| State | Meaning |
|---|---|
| **Loaded** | in the camera, being shot |
| **To develop** | finished — the one thing you actually have to do |
| **Developing** | at the lab, or in your tank |
| **Developed** | ready — go get it, or it's hanging to dry |
| **Picked up** | home, waiting to be scanned |
| **Scanned** | digitised |
| **Filed** | in its sleeve, where you can find it |

Click a roll's status label to push it to the next stage. No dialogs, no menus — and
deletions, status changes and removed cameras can all be undone from the notice that
appears.

## Lab or home development

Each roll takes one of two paths, and the card shows the fields that match. Send it to a lab
and you get the lab's name and your order number. Develop it yourself and you get the
recipe: developer, dilution, time, temperature — kept with the roll forever, so next time
you know what you did.

![A roll's card, showing the home-development recipe](capturas/03-ficha.png)

## Everything else

- **Search** across every field at once — film, place, developer, note, order number.
- **Group by camera or by status** — one view for browsing, one for "what do I have to do".
- **Free-text dates.** `22/06/26`, `july`, `before the trip?` — a logbook should accept the
  way you actually remember things. There's a `today` button when you want to be exact.
- **Where the negative lives** (`Binder 2 · sleeve 14`) and **a link to the scans.**
- **Format and exposure count** — 35mm, 120, 4×5, whatever you shoot.
- **English and Spanish**, light and dark, keyboard shortcuts, and a clean printout for the
  darkroom wall.
- **Export to a spreadsheet** whenever you want your data somewhere else.

![The same rolls grouped by status: what you have to do](capturas/04-por-estado.png)

![The light theme](capturas/05-claro.png)

---

## Install

There is no installer. Pick either:

**Use it on the web** — open **[jaimealekos.github.io/almendros](https://jaimealekos.github.io/almendros/)**.
It still saves into your own folder; nothing is uploaded anywhere.

**Or keep your own copy** — download [`index.html`](https://raw.githubusercontent.com/jaimealekos/almendros/main/index.html)
(right-click → Save link as) and put it wherever you like. Double-click to open. It works
with the wifi off, forever, whatever happens to this repository.

Then: choose the folder where your rolls will live — once per computer — and add your first
roll.

![The welcome screen, asking for a folder](capturas/07-primer-arranque.png)

> **Brave users:** Brave ships with folder saving switched off. The page detects it and
> walks you through the one setting you need to enable, once per computer.

📖 **[The full manual is here](LEEME.md#manual)** — it's short.

---

## Your data

Everything lives in **one plain file** called `almendros.json`, inside the folder you chose,
with a daily backup in `copias/`.

```
your folder/
├── almendros.json      ← every roll you have ever shot
└── copias/             ← daily backups, last 30 kept
```

Open it in Notepad and you can read it. **If this project disappears tomorrow, your log is
still there, still legible, without me.** That's the whole point of the format.

**Several computers, one folder.** Put it on a NAS or a synced drive and open the page
anywhere. Almendros merges roll by roll — each roll knows when it was last touched and each
deletion leaves a marker — so two computers editing different rolls both keep their work.

**Offline is fine.** If the folder is unreachable, changes wait in the browser and are
written the moment it's back.

## Why not frame by frame?

Because you won't keep it up. Everyone who has tried a per-frame log knows how it ends: the
spreadsheet gets opened with good intentions and abandoned by roll three. Nobody stops
mid-street to type an aperture.

A roll takes ten seconds to log, which is why you'll actually do it — and the roll is the
unit that matters anyway. It's what the lab handles, what the sleeve holds, and what you
lose.

## Other tools worth knowing

Almendros doesn't try to be everything. If you want something it deliberately doesn't do,
these are good and you should use them — alongside it or instead of it:

- **[Exif Notes](https://github.com/tommi1hirvonen/ExifNotes)** — open source, Android.
  Frame-by-frame notes and it can write EXIF into your scans.
- **[Massive Dev Chart](https://www.digitaltruth.com/devchart.php)** — the development-time
  database and timer. Almendros records the recipe you used; this tells you what it should be.
- **[Crown + Flint](https://crownandflint.com/)**, **[Pellica](https://pellica.app/)**,
  **[Frames](https://withframes.com/)** — polished mobile apps with light meters and
  in-the-field logging. If you want to log while you walk, get one of these.
- **[Filmbook](https://flathub.org/apps/io.github.nate_xyz.Filmbook)** — a native Linux app
  with a similar roll-level philosophy.

What Almendros gives you that those don't: it's open source, it costs nothing, it has no
account and no limits, it runs on any desktop, and your log is a plain file you own.

## Development

Everything is in `index.html`: CSS, markup and one vanilla script. No build step, no
dependencies, no network access at runtime — it has to work from `file://` with the wifi
off. To work on it, open the file. That's the whole toolchain.

- **`index.html#pruebas`** runs the built-in test suite in the page — merge logic,
  migrations from every historical file format, code numbering, CSV escaping.
- **`index.html#demo`** loads sample rolls in memory and saves nothing.

Bug reports and ideas are welcome, especially from photographers who develop at home.

## Licence

[MIT](LICENSE) — free to use, copy, improve and share.
