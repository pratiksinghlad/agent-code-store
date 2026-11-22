# Agent KB

Search this file before writing a reusable script. One row is one script. `File` is the path. `Description` is what it does. Run that command with `--help`. Do not open the file unless you are fixing or extending it.

Create a folder named after the script and put the one script inside it:

- Python: `python/<name>/run.py`
- TypeScript: `ts/<name>/run.ts`
- Rust: `rust/src/bin/<name>/main.rs`
- Data, later: `data/<name>/`

Add the same row to that folder's `INDEX.md`.

| Name | File | Description | Run |
| :--- | :--- | :--- | :--- |
| `json` | `python/json/run.py` | Read JSON or write it atomically | `uv run python/json/run.py --path <file>` |
| `json` | `ts/json/run.ts` | Read JSON or write it atomically | `bun ts/json/run.ts --path <file>` |
| `json` | `rust/src/bin/json/main.rs` | Read JSON or write it atomically | `cargo run --manifest-path rust/Cargo.toml --bin json -- --path <file>` |
