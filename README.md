Super simple X window manager in Rust. Accronym TBD.

Built around Rust XCB bindings.

No *full* ICCCM/EWMH support planned, possibly just enought to get by. As
for why, see: https://raw.githubusercontent.com/kfish/xsel/master/rant.txt

Floating only (for now).

Set your key binds in `src/config.rs`.

Keepin' it simple. LOC count: `944`

![screenshot](https://github.com/grufwub/afwm/raw/master/screenshot.png)

# Todos

- add randr support

- add tiling mode + window gaps in tiling mode

- add full-screen window support (e.g. games break everything right now)

- improve MOD+Tab window cycling

- either add status bar with workspace info, or support _some_ EWMH

- fix old code comments (referring to previous versions)
