Super simple X window manager in Rust. Accronym TBD.

Built around Rust XCB bindings.

No *full* ICCCM/EWMH support planned, just enough to get by. As
for why, see: https://raw.githubusercontent.com/kfish/xsel/master/rant.txt

Floating only (for now).

Set your key binds in `src/config.rs`.

Keepin' it simple. LOC count: `1010`

![screenshot](https://github.com/grufwub/afwm/raw/master/screenshot.png)

# Usage

Window dragging: `MOD` + left click mouse

Window resizing: `MOD` + right click mouse

Example xinitrc:
```sh
#!/bin/sh

(while true; do echo "$(date +%T)"; sleep 1; done) | lemonbar -f ' - 12' &

exec afwm
```
