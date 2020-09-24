Super simple X window manager in Rust. Accronym TBD.

Built around Rust XCB bindings.

No *full* ICCCM/EWMH support planned, possibly just enought to get by. As
for why, see: https://raw.githubusercontent.com/kfish/xsel/master/rant.txt

Floating only (for now).

Set your key binds in `src/config.rs`.

Keepin' it simple. LOC count: `944`

![screenshot](https://github.com/grufwub/afwm/raw/master/screenshot.png)

# Issues

- MOD+Tab window cycling ignores currently focused window

- child windows of children (e.g. popup windows, not menus) not rendering
  until they're grabbed with MOD+LeftClick and focused. I think this may be
  us not propagating CirculateRequest events

- full-screen windows (e.g. games) not working. could be related to above

- opening new Firefox windows doesn't map them to the screen until the current
  window is moved to another workspace. again, could be related to above

# Todos

- add randr support

- add tiling mode + window gaps in tiling mode

- either add status bar with workspace info, or support _some_ EWMH

- fix old code comments (referring to previous versions)
