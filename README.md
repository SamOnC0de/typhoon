# 🌀 Typhoon

> **Build web apps in pure Rust — no JavaScript, no complexity.**
> A lightweight WASM frontend framework designed for beginners and fast prototypes.

```rust
use typhoon_core::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    init();
    let count = use_state(0u32);
    let display = tp! { p.text(count.get()) };

    let display_ref = display.clone();
    let count_sub = count.clone();
    count.subscribe(move || {
        display_ref.set_text_content(Some(&count_sub.get().to_string()));
    });

    let count_inc = count.clone();
    let app = tp! {
        div.class("app") {
            h1.text("Typhoon Counter")
            button.onclick(move || count_inc.set(count_inc.get() + 1)) { "+" }
        }
    };
    app.append_child(display.as_ref()).unwrap();
    mount(app);
}
```

---

## ✨ Features

- **`tp!` macro** — write HTML-like trees directly in Rust
- **Reactive signals** — `use_state()` auto-updates the DOM on change
- **LocalStorage hook** — `use_local_storage()` persists state across page reloads
- **Hash router** — `use_router()` maps `#/`, `#/about` etc. to render functions
- **Zero external JS** — pure Rust + WASM + web-sys
- **Tiny bundles** — targets <100KB with `wasm-opt`
- **Fast hot-reload** — via [Trunk](https://trunkrs.dev/) (~0.1s)
- **Beginner-friendly** — readable errors, simple API, no magic

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
typhoon-core = "0.1"
wasm-bindgen = "0.2"
```

Install [Trunk](https://trunkrs.dev/) for the dev server + build:

```bash
cargo install trunk
```

---

## 🚀 Quick Start

### 1. Create your project

```bash
cargo new --lib my-app
cd my-app
```

Set up `Cargo.toml`:

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
typhoon-core = "0.1"
wasm-bindgen = "0.2"
```

Create `index.html`:

```html
<!DOCTYPE html>
<html>
  <head><meta charset="UTF-8" /><title>My App</title></head>
  <body></body>
</html>
```

### 2. Write your app in `src/lib.rs`

```rust
use typhoon_core::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    init();
    mount(tp! {
        h1.text("Hello, Typhoon! 🌀")
    });
}
```

### 3. Run

```bash
trunk serve
# → http://localhost:8080 with live reload
```

### 4. Build for production

```bash
trunk build --release
# → dist/ folder ready to deploy
```

---

## 📖 API Reference

### `tp! { ... }` — The UI macro

```
tag[.method(arg)]* [{ children }]
```

| Method | Description | Example |
|--------|-------------|---------|
| `.text(val)` | Set text content | `.text("hello")` or `.text(my_var)` |
| `.class(str)` | Set CSS class | `.class("card active")` |
| `.style(str)` | Inline CSS | `.style("color:red;font-size:2rem")` |
| `.id(str)` | Set element id | `.id("my-button")` |
| `.onclick(fn)` | Click handler | `.onclick(my_fn)` |
| `.oninput(fn)` | Input handler (gets `String`) | `.oninput(move \|v\| ...)` |
| `.onkeydown(fn)` | Keydown handler (gets key `String`) | `.onkeydown(move \|k\| ...)` |
| `.placeholder(str)` | Input placeholder | `.placeholder(&"Search…")` |
| `.value(val)` | Element value attribute | `.value(current_val)` |

The macro returns a `web_sys::Element`.

---

### `use_state<T>(initial: T) -> Signal<T>`

Creates a reactive value. `Signal<T>` is cheap to clone (`Rc` under the hood).

```rust
let count = use_state(0u32);
count.get();           // read
count.set(1);          // write — triggers subscribers
let c = count.clone();
count.subscribe(move || { /* runs on every .set() */ });
```

---

### `use_local_storage<T>(key: &'static str, default: T) -> Signal<T>`

Reactive signal backed by `localStorage`. Loads on startup, saves as JSON on every `.set()`.

```rust
let todos: Signal<Vec<String>> = use_local_storage("todos", vec![]);
todos.set(vec!["Buy milk".into()]); // persisted immediately
```

---

### `use_memo<T, D, F>(deps: D, compute: F) -> Signal<T>`

Computed signal that re-evaluates whenever a dependency changes.

```rust
let count = use_state(0i32);
let c = count.clone();
let doubled = use_memo(count.clone(), move || c.get() * 2);
// doubled updates automatically when count changes
```

---

### Components

Plain functions returning `Element`. Embed with `(expr)` in `tp!`.

```rust
fn badge(label: &str) -> Element {
    tp! { span.text(label).style("padding:2px 8px;border-radius:999px") }
}

let app = tp! {
    div {
        h1.text("Gallery")
        (badge("new"))
        (badge("hot"))
    }
};
```

---

### `use_effect(f: impl FnOnce() + 'static)`

Runs once after the current render (next event-loop tick). Use for data fetches or DOM work that must happen after `mount()`.

```rust
use_effect(move || {
    spawn_local(async move {
        let resp = fetch_text("https://api.example.com/data").await;
        data.set(resp);
    });
});
```

---

### `use_interval(callback: impl FnMut() + 'static, ms: i32) -> IntervalHandle`

Repeating callback every `ms` milliseconds. Cancelled on drop; call `.forget()` for a permanent interval.

```rust
use_interval(move || time.set(current_time()), 1000).forget();
```

---

### `spawn_local(future: impl Future<Output = ()> + 'static)`

Runs an `async` block on the WASM executor. Re-exported from `wasm-bindgen-futures`.

---

### `use_router(routes: Vec<(&'static str, Box<dyn Fn() -> Element>)>) -> Element`

Hash-based router. Matches `window.location.hash`, falls back to first route.

```rust
let app = use_router(vec![
    ("#/",      Box::new(|| tp! { h1.text("Home") })),
    ("#/about", Box::new(|| tp! { h1.text("About") })),
]);
mount(app);
```

---

### `mount(el)` / `mount_to(id, el)` / `init()`

```rust
init();               // call first — enables readable panics in console
mount(el);            // append to document.body
mount_to("root", el); // append to #root
```

---

## 🧩 Examples

| Example | Description | Run |
|---------|-------------|-----|
| [counter](examples/counter) | Reactive counter | `cd examples/counter && trunk serve` |
| [todo](examples/todo) | Todo list with localStorage | `cd examples/todo && trunk serve` |
| [clock](examples/clock) | Live clock with `use_interval` | `cd examples/clock && trunk serve` |
| [components](examples/components) | Stateless & stateful components | `cd examples/components && trunk serve` |

---

## 🗺️ Roadmap

| Feature | Status |
|---------|--------|
| `tp!` macro | ✅ |
| `use_state` — reactive signals | ✅ |
| `use_local_storage` | ✅ |
| `use_router` — hash navigation | ✅ |
| `use_effect` + `use_interval` | ✅ |
| Components + `(expr)` embedding | ✅ |
| `use_memo` — derived signals | ✅ |
| Published on crates.io | ✅ |
| DOM diffing | 🔲 |
| `#[typhoon::main]` attribute | 🔲 |

---

## 🤝 Contributing

Open an issue first to discuss the change, then:

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Open a PR

---

## 📄 License

MIT — see [LICENSE](LICENSE)
