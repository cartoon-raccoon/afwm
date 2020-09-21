Super simple window manager in Rust. Accronym TBD.

If you're wondering why any of this then look -- `afwm [-y|--why]`

LOC count (according to `loc` tool): `1137`

Floating only (for now).

Set your key binds in `src/config.rs`

# Todos

- tiling mode
  - window gaps in tiling mode

- fix full screen windows breaking EVERYTHING

- fix some windows unable to be moved/resized

- window cycle when highlighting over some ignores them during shuffle
  (because on MOD key press it refocuses then performs focus change again)

- status bar with workspace info

- window borders

- dmenu coloring matching status bar coloring

- properly comment (add where missing, fix old comments)

- ICCCM support

- EWMH support?