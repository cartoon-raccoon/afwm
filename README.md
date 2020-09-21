Super simple window manager in Rust.

Accronym TBD.

If you're wondering why any of this then look -- `afwm [-y|--why]`

LOC count (according to `loc` tool): `1138`

I am Rust noob. I have never touched X before this. Pls go easy on my
fragile existence.

# Todos

- switch to using x11rb library?

- figure out `EVENT_MASK_ENTER_WINDOW` for mapped windows (fucks keyboard input
  if we enable this)

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

- OPTIMISE