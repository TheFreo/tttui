# Contributing

## Prerequisites

- Rust toolchain
- Cargo

## Commits

- Use conventional commits.
- Keep commits scoped to one concern where practical.

## Development

Run the standard checks before opening a change:

```sh
cargo fmt
cargo check -p tttui
cargo test -p tttui
```

Run the app locally with:

```sh
cargo run --bin tttui
```

## Project Structure

```text
crates/
├── tttui_core/
│   └── shared kernel types and errors
└── tttui_app/
    ├── config/
    ├── features/preferences/
    └── features/typing_test/
```

Keep feature boundaries meaningful. Do not add empty layers or generic abstractions unless they remove real complexity.
