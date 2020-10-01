Super simple X window manager in Rust. Accronym TBD.

Built around Rust XCB bindings.

No *full* ICCCM/EWMH support planned, just enough to get by. As
for why, see: https://raw.githubusercontent.com/kfish/xsel/master/rant.txt

Floating only (for now).

Set your key binds in `src/config.rs`.

Keepin' it simple. LOC count: `1009`

![screenshot](https://github.com/grufwub/afwm/raw/master/screenshot.png)

# Usage

Window dragging: `MOD` + left click mouse

Window resizing: `MOD` + right click mouse

Example xinitrc:
```sh
#!/bin/sh

(while true; do echo "$(date +%T)"; sleep 1; done) | lemonbar -f ' - 12' &

startx afwm
```

# Issues

- MOD+Tab window cycling ignores currently focused window

- full-screen windows (e.g. games) not working

- opening new Firefox windows doesn't map them to the screen until the current
  window is moved to another workspace

# Todos

- add randr support

- add tiling mode + window gaps in tiling mode

- output window manager state to file (for lemonbar, etc)