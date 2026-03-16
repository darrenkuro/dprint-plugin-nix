# CLAUDE.md

Configurable Nix code formatter as a dprint WASM plugin. Parses Nix via `rnix`, generates formatted output via `dprint-core` print items.

## Commands

```bash
# Build WASM
cargo build --release --target wasm32-unknown-unknown --features wasm

# Run tests
cargo test

# Run local-only idempotency test (formats ~/.config/home-manager/home.nix)
cargo test -- --ignored

# Clippy
cargo clippy -- -D warnings

# Test the plugin via dprint
echo '{pkgs,...}:{home.packages=with pkgs;[hello];}' | dprint fmt --stdin file.nix
```

## Release

CI auto-builds and publishes on tag push. To release a new version:

1. Bump `version` in `Cargo.toml`
2. Commit: `git commit -am "chore: bump to vX.Y.Z"`
3. Tag and push: `git tag vX.Y.Z && git push && git push origin vX.Y.Z`
4. GitHub Actions builds WASM and creates a release with the artifact
5. Update the plugin URL in `~/.config/home-manager/configs/dprint.json` to the new version

The WASM is published at:
`https://github.com/darrenkuro/dprint-plugin-nix/releases/download/vX.Y.Z/dprint_plugin_nix.wasm`

## Architecture

- `src/plugin.rs` — dprint plugin handler (config schema, formatting entry point)
- `src/format.rs` — top-level formatting: parse with rnix, generate print items
- `src/config.rs` — `NixConfig` (lineWidth, indentWidth)
- `src/generate/` — per-AST-node print item generation (attr_set, lambda, let_in, list, if_else, etc.)
- `tests/format_test.rs` — unit tests
- `tests/diff_test.rs` — local idempotency test against home.nix (#[ignore] in CI)
