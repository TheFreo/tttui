# tttui

`tttui` is a minimal terminal typing test written in Rust. It keeps the interface compact: choose a mode from one selector row, press `Enter`, and type.

## Features

- Time, words, punctuation, numbers, and quote modes
- Compact selector-first home screen
- Configurable key sequences
- User-editable TOML themes with color and presentation overrides
- Bundled and user-provided languages / quotes
- Personal best tracking
- Result stats and WPM graph
- Layout that remains usable at `80x24`

## Run

```sh
cargo run --bin tttui
```

## Controls

Default bindings:

- `Tab`: move focus on the home screen
- `Left` / `Right`: change the focused selector
- `Enter`: start or retry
- `q`: quit
- `Tab Enter`: restart during a test
- `Tab m`: return to the menu during a test

All bindings are configurable in `~/.config/tttui/config.toml`.

## Configuration

The app creates its config directory on first launch:

```text
~/.config/tttui/
├── config.toml
├── languages/
├── quotes/
└── themes/
```

Example `config.toml`:

```toml
[defaults]
mode = "time"
duration = 30
word_count = 25
language = "english"
theme = "default"

[options]
durations = [15, 30, 60, 120]
word_counts = [10, 25, 50, 100]

[keybindings]
quit = ["q"]
start = ["enter"]
focus_next = ["tab"]
focus_previous = ["shift+tab"]
cycle_next = ["right", "l"]
cycle_previous = ["left", "h"]
restart = ["tab enter"]
menu = ["tab m"]
backspace = ["backspace"]
```

Keybindings support multi-key sequences such as `"tab enter"` and modified keys such as `"ctrl+r"`.

Supported modes are `time`, `words`, `punctuation`, `numbers`, and `quote`. The word-count selector is reused by `words`, `punctuation`, and `numbers`.

## Themes

Place custom themes in `~/.config/tttui/themes/<name>.toml`. Built-in themes are `default`, `nord`, and `catppuccin-mocha`.

Example theme:

```toml
[colors]
text = "#cdd6f4"
muted = "#6c7086"
correct = "#a6e3a1"
incorrect = "#f38ba8"
untyped = "#7f849c"
caret = "#f9e2af"
accent = "#89b4fa"
background = "default"
selection = "#45475a"

[presentation]
show_borders = false
border_style = "plain"
selector_separator = " / "
caret_symbol = "_"
```

Supported colors:

- Named terminal colors such as `"green"` or `"lightcyan"`
- 256-color indexes such as `"151"`
- Hex RGB values such as `"#a6e3a1"`
- `"default"` for the terminal default background

Theme presentation fields currently allow color choices, optional borders, selector separators, and caret symbols to vary without recompiling.

## Custom content

Add word lists and quote files here:

```text
~/.config/tttui/languages/<language>.txt
~/.config/tttui/quotes/<language>.txt
```

Each file uses one word or quote per line. User files override bundled files with the same name.

## Workspace

```text
crates/
├── tttui_core/
│   └── shared kernel types and errors
└── tttui_app/
    ├── config/
    ├── features/preferences/
    └── features/typing_test/
```

The code follows feature boundaries where they carry real responsibility, without adding empty layers only to satisfy a directory pattern.
