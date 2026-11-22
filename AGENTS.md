# Agent knowledge base

This repository is the local store of standalone scripts. Search `INDEX.md` before writing a reusable script, run a hit as a command, and update the index rows when you add, change, or remove a script.

## Layout

```text
INDEX.md                         every script: file path and description
python/<name>/run.py             one Python script in its own folder
python/INDEX.md                  those Python rows
ts/<name>/run.ts                 one TypeScript script in its own folder
ts/INDEX.md                      those TypeScript rows
rust/src/bin/<name>/main.rs      one Rust binary in its own folder
rust/INDEX.md                    those Rust rows
rust/Cargo.toml                  builds the binaries; copy a release build to bin/
data/<name>/                     schemas, templates, and lookup data when added
data/INDEX.md                    those data rows
```

The folder name is the script name. One script lives in that folder. Do not add a second script beside it.

## Search

1. Read `INDEX.md`. Match on the file path or the description.
2. Or run `rg -i "<keyword>" INDEX.md */INDEX.md`.
3. Run that row's command with `--help`. Do not open the source unless you are fixing or extending it.

## Add

Add a script only when another repository can run it unchanged. Skip project-specific fixes, secrets, and one-off data.

1. Create a folder named after the script and put the one file inside it.
   - Python: `python/<name>/run.py`, PEP 723 header, standard library only, `uv run`.
   - TypeScript: `ts/<name>/run.ts`, no build step, `bun`.
   - Rust: `rust/src/bin/<name>/main.rs`. Add a `[[bin]]` in `rust/Cargo.toml`. Build with `cargo build --release --manifest-path rust/Cargo.toml` and copy the binary to `bin/<name>` (`bin/<name>.exe` on Windows).
   - Data: `data/<name>/`.
2. The script accepts `--help`, takes arguments or stdin, writes result data to stdout, and has no secrets in source.
3. Add one row to `INDEX.md` and the same row to that folder's `INDEX.md`: name, file, description, run command.
4. Check `--help` and one successful run before you finish.

Prefer a new flag on an existing script over a second script that does the same job. When the file, description, or run command changes, update both rows in the same edit.

## Remove

Delete the script folder, delete `bin/<name>` when it exists, and delete both index rows in the same change.

## Harness pointer

Point Claude Code, Codex, and Antigravity at this clone:

```markdown
## Agent knowledge base
- Before writing a reusable script, read `INDEX.md` (file path and description) in this clone, or `rg -i "<keyword>" INDEX.md */INDEX.md`.
- Reuse a tool with `--help`. Do not read its source unless you are fixing or extending it.
- Add a script by creating `<lang>/<name>/` and putting the one file inside it. Update `INDEX.md` and that language `INDEX.md` on add, edit, and delete.
```
