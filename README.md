# jsonlz4

[![CI](https://github.com/stvleung/jsonlz4/actions/workflows/ci.yml/badge.svg)](https://github.com/stvleung/jsonlz4/actions/workflows/ci.yml)
[![License: BSD-2-Clause](https://img.shields.io/badge/license-BSD--2--Clause-blue.svg)](LICENSE)

A small Rust library and CLI for Mozilla Firefox's `.jsonlz4` / `.mozlz4`
container format. Firefox uses it for bookmark backups, session restore data,
and search settings. The format is **not** the standard LZ4 frame format that
the upstream `lz4` CLI produces — it's a fixed 12-byte header followed by a
raw LZ4 block — so the standard tool can't read or write these files.

## File format

```text
+---------------------+--------------------------------+--------------------------+
| 8 bytes: "mozLz40\0" | 4 bytes: u32 LE decomp. size  | N bytes: raw LZ4 block  |
+---------------------+--------------------------------+--------------------------+
```

## Install

```sh
cargo install jsonlz4
```

Or build from source:

```sh
git clone https://github.com/stvleung/jsonlz4
cd jsonlz4
cargo build --release
# binary at target/release/jsonlz4
```

## CLI usage

```text
jsonlz4 [-d|-c] IN_FILE [OUT_FILE]

  -d, --decompress    Decompress IN_FILE (default).
  -c, --compress      Compress IN_FILE.
  -h, --help          Show help.
  -V, --version       Show version.

Use `-` for IN_FILE or OUT_FILE to read/write standard streams.
If OUT_FILE is omitted, output goes to stdout.
```

### Examples

```sh
# Inspect a Firefox bookmark backup as JSON.
jsonlz4 ~/Library/Application\ Support/Firefox/Profiles/*.default-release/bookmarkbackups/*.jsonlz4 \
  | jq .

# Round-trip via stdin/stdout.
cat backup.jsonlz4 | jsonlz4 -d - > backup.json
cat backup.json    | jsonlz4 -c - > backup.jsonlz4
```

> ⚠️  Writing `.jsonlz4` files is supported, but the format is non-standard
> and Mozilla has discussed migrating away from it
> ([bug 1209390](https://bugzilla.mozilla.org/show_bug.cgi?id=1209390)).
> Prefer the standard LZ4 frame format for new use cases.

## Library usage

```rust
let original = b"{\"hello\": \"world\"}";
let encoded  = jsonlz4::encode(original).unwrap();
let decoded  = jsonlz4::decode(&encoded).unwrap();
assert_eq!(decoded, original);
```

The crate is `#![forbid(unsafe_code)]` and depends only on
[`lz4_flex`](https://crates.io/crates/lz4_flex) (pure-Rust LZ4) and
[`thiserror`](https://crates.io/crates/thiserror).

See the [API docs](https://docs.rs/jsonlz4) for details.

## Development

```sh
cargo test         # unit + integration tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The minimum supported Rust version (MSRV) is **1.85**.

## References

- Mozilla Firefox [bug 818587](https://bugzilla.mozilla.org/show_bug.cgi?id=818587)
  — original "compress bookmark backups" change.
- Mozilla Firefox [bug 1209390](https://bugzilla.mozilla.org/show_bug.cgi?id=1209390)
  — proposed migration to the standard LZ4 frame format.
- Original C tool: <https://github.com/avih/dejsonlz4>

## License

BSD 2-Clause. See [LICENSE](LICENSE).
