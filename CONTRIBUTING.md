# Contributing

Thanks for your interest in `jsonlz4`. This is a small project, so the bar is
straightforward: keep changes focused, keep the test suite green, and prefer
small PRs over large ones.

## Development setup

```sh
git clone https://github.com/stvleung/jsonlz4
cd jsonlz4
cargo build
cargo test
```

The minimum supported Rust version is listed in `Cargo.toml` (`rust-version`).
CI verifies that the crate still builds on it; please don't bump it casually.

## Before opening a PR

Run the same checks CI runs:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

If you're changing the file-format handling itself, please add or update tests
in `src/lib.rs` (unit) or `tests/format_compatibility.rs` (round-trip /
layout) so the on-disk format stays compatible with Firefox.

## Reporting issues

When filing a bug, please include:

- Rust version (`rustc --version`).
- A minimal reproducible example, ideally a small `.jsonlz4` fixture.
- The expected vs. actual behavior.

## Code style

- `rustfmt` defaults plus `max_width = 100` (see `rustfmt.toml`).
- `#![forbid(unsafe_code)]` is intentional — please don't introduce `unsafe`.
- Prefer adding tests over adding comments that explain "what" the code does.

## License

By contributing you agree that your contributions will be licensed under the
project's BSD 2-Clause license.
