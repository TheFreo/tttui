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

## GitHub Actions Release

The release workflow in `.github/workflows/release.yml` publishes both crates automatically when a tag matching `v*` is pushed, for example:

```sh
git tag v0.1.0
git push origin v0.1.0
```

Before using the workflow, create a crates.io API token and add it to the GitHub repository as an Actions secret named:

```text
CARGO_REGISTRY_TOKEN
```

The workflow publishes `tttui_core` first, waits for it to become visible on crates.io, then publishes `tttui`.

## Manual Preflight

Before publishing, run:

```sh
cargo fmt --check
cargo check -p tttui
cargo test -p tttui
cargo package -p tttui_core
cargo package -p tttui
```

## Local Workflow Testing

GitHub Actions run on GitHub; nothing must be installed locally for them to work after you push.

If you want a local approximation, you can install `act` and run workflows against Docker containers. Use it for quick feedback only; the GitHub-hosted run is still the real verification environment.
