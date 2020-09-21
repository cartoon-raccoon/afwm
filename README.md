Super simple window manager in Rust. Accronym TBD.

If you're wondering why any of this then look -- `afwm [-y|--why]`

LOC count (according to `loc` tool): `1049`

Floating only (for now).

Set your key binds in `src/config.rs`.

![screenshot](https://github.com/grufwub/afwm/raw/master/screenshot.png)

This is my learning project for both X and Rust, so I may not accept PRs for now unless
they're smaller things like bug fixes.

No ICCCM support, see: https://raw.githubusercontent.com/kfish/xsel/master/rant.txt

No EWMH support.

Keepin' it simple.

# Todos

- tiling mode
  - window gaps in tiling mode

- fix full screen windows breaking EVERYTHING

- window cycle when highlighting over some ignores them during shuffle
  (because on MOD key press it refocuses then performs focus change again)

- status bar with workspace info

- window borders

- dmenu coloring matching status bar coloring

- properly comment (add where missing, fix old comments)

