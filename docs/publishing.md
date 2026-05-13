# Publishing

The workspace contains two crates:

- `tttui_core`
- `tttui`

`tttui` depends on `tttui_core`, so publish them in this order:

```sh
cargo publish -p tttui_core
cargo publish -p tttui
```

After `tttui` is published, users can install the executable globally with:

```sh
cargo install tttui
```

Before publishing, run:

```sh
cargo fmt --check
cargo check -p tttui
cargo test -p tttui
cargo package -p tttui_core
cargo package -p tttui
```
