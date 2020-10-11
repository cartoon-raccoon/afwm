Super simple X window manager in Rust. Accronym TBD.

Built around Rust XCB bindings.

No *full* ICCCM/EWMH support planned, just enough to get by. As
for why, see: https://raw.githubusercontent.com/kfish/xsel/master/rant.txt

Floating only (for now).

Set your key binds in `src/config.rs`.

Keepin' it simple. LOC count: `1040`

Initially inpsired by Lanta (https://github.com/mjkillough/lanta) but very
quickly became its own beast.

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

# Building

`./build` will build the release version for your current default target.

Building the debug version will include _a lot_ of extra debug printing should
you need it.