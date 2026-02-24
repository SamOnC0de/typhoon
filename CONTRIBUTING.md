# Contributing to Typhoon 🌀

Thanks for your interest! Here's how to get started.

## Setup

```bash
git clone https://github.com/SamOnC0de/typhoon
cd typhoon
cargo check  # verify everything compiles
```

You'll need:

- Rust stable (1.75+)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

## Project Structure

```
typhoon/
├── typhoon-macro/     # The tp! proc-macro (compile-time)
│   └── src/lib.rs
├── typhoon-core/      # Runtime: DOM helpers, Signal, mount()
│   └── src/lib.rs
├── examples/
│   ├── counter/       # Click counter demo
│   └── todo/          # Todo list demo
└── README.md
```

## Running Examples

```bash
cd examples/counter
trunk serve             # → localhost:8080
```

## Workflow

1. Open an issue describing your change
2. Fork + create a branch: `git checkout -b feat/my-feature`
3. Make changes
4. Test: `cargo check` + `trunk build` in at least one example
5. Open a PR referencing the issue

## Code Style

- `cargo fmt` before committing
- `cargo clippy -- -D warnings` must pass
- Keep `typhoon-core/src/lib.rs` under 300 lines for now (MVP simplicity)
- Avoid adding new deps without discussion

## What's Needed

Check the [roadmap in README](README.md#️-roadmap) — `use_local_storage` and basic routing are great first contributions!
