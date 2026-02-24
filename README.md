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

> Typhoon is not yet published on crates.io. Use the git dependency for now.

Add to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
typhoon-core = { git = "https://github.com/SamOnC0de/typhoon" }
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
typhoon-core = { git = "https://github.com/SamOnC0de/typhoon" }
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

Write your DOM tree using a JSX-like syntax. Each node follows this pattern:

```
tag[.method(arg)]* [{ children }]
```

#### Supported elements

`div`, `span`, `p`, `h1`–`h6`, `button`, `input`, `ul`, `li`, `a`, `section`, `header`, `footer`, `main`, `nav`, `form`, `label`, `img`

#### Supported props

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

#### Nesting

Children go in `{ }` blocks. You can mix nodes and string literals:

```rust
tp! {
    div.class("card") {
        h2.text("Title")
        p { "Some static text inside a paragraph." }
        button.onclick(handler) { "Click me" }
    }
}
```

The macro returns a `web_sys::Element` — you can use all standard web-sys methods on it.

---

### `use_state<T>(initial: T) -> Signal<T>`

Creates a reactive value. `Signal<T>` is cheap to clone (`Rc` under the hood).

```rust
let count = use_state(0u32);

// Read
let current = count.get(); // → 0u32

// Write (triggers all subscribers)
count.set(current + 1);

// React to changes
let count2 = count.clone();
count.subscribe(move || {
    // runs every time count.set() is called
    web_sys::console::log_1(&count2.get().into());
});
```

**Note:** `Signal<T>` implements `Display` when `T: Display`, so you can use `.text(my_signal.get())` directly.

---

### `use_local_storage<T>(key: &'static str, default: T) -> Signal<T>`

Creates a reactive signal backed by `localStorage`. The value is loaded on
startup and automatically saved as JSON every time `.set()` is called.

`T` must implement `serde::Serialize + serde::DeserializeOwned`. All standard
types (`String`, `Vec<T>`, numbers, booleans, structs with `#[derive(Serialize, Deserialize)]`) work out of the box.

```rust
// Persists across page refreshes — no extra code needed
let todos: Signal<Vec<String>> = use_local_storage("todos", vec![]);

todos.set(vec!["Buy milk".into()]); // written to localStorage immediately
```

The hook behaves identically to `use_state` — you can `.subscribe()`, `.get()`,
and `.set()` on the returned `Signal<T>`.

---

### `use_router(routes: Vec<(&'static str, Box<dyn Fn() -> Element>)>) -> Element`

A lightweight hash-based router. Routes are matched against
`window.location.hash` (e.g. `"#/"`, `"#/about"`). Re-renders the active route
on every `hashchange` event. Falls back to the first route when no hash matches.

Returns a container `Element` that you pass to `mount()`.

```rust
let app = use_router(vec![
    ("#/",      Box::new(|| tp! { h1.text("Home") })),
    ("#/about", Box::new(|| tp! { h1.text("About") })),
    ("#/todo",  Box::new(|| {
        // each route is re-created on navigation
        tp! { p.text("Todo page") }
    })),
]);
mount(app);
```

Navigate between routes with plain anchor links:

```html
<a href="#/">Home</a>
<a href="#/about">About</a>
```

---

### `mount(el: Element)`

Appends an element to `document.body`.

```rust
mount(tp! { h1.text("Hello!") });
```

### `mount_to(id: &str, el: Element)`

Appends to a specific element by id.

```rust
// index.html has <div id="root"></div>
mount_to("root", tp! { h1.text("Hello!") });
```

### `init()`

Sets up readable panic messages in the browser console. Always call this first.

```rust
#[wasm_bindgen(start)]
pub fn main() {
    init(); // ← call first
    // ...
}
```

---

## 🧩 Examples

### Counter

```rust
use typhoon_core::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    init();

    let count = use_state(0i32);
    let display = tp! { p.text(count.get()) };

    let display_ref = display.clone();
    let count_sub = count.clone();
    count.subscribe(move || {
        display_ref.set_text_content(Some(&count_sub.get().to_string()));
    });

    let count_inc = count.clone();
    let count_dec = count.clone();

    let app = tp! {
        div.class("app") {
            h1.text("Counter")
            button.onclick(move || count_dec.set(count_dec.get() - 1)) { "−" }
            button.onclick(move || count_inc.set(count_inc.get() + 1)) { "+" }
        }
    };

    app.append_child(display.as_ref()).unwrap();
    mount(app);
}
```

Run it:
```bash
cd examples/counter && trunk serve
```

### Todo List (with persistence)

See [`examples/todo/src/lib.rs`](examples/todo/src/lib.rs) — a full todo app with:
- `use_local_storage` — todos survive page refresh
- Input field with `oninput` + `onkeydown` (Enter to add)
- Delete buttons per item

```bash
cd examples/todo && trunk serve
```

---

## 📚 Glossary

| Term | What it means |
|------|--------------|
| **Macro** | A magic shortcut in Rust. `tp!` transforms short code into full DOM elements. |
| **Hot-reload** | When you change code, the page updates automatically in ~0.1s. |
| **Bundle** | The file your browser downloads. Smaller = faster load. |
| **Reactivity** | When a value changes, the display updates automatically. |
| **Signal** | A reactive box. When you `.set()` it, subscribers are notified. |
| **Proc-macro** | An advanced Rust macro that runs at compile time to generate code. |
| **Tree-shake** | Removing unused code to make the bundle smaller. |
| **WASM** | WebAssembly — runs Rust code at near-native speed in the browser. |
| **Trunk** | The build tool that compiles your Rust → WASM + serves with hot-reload. |

---

## 🗺️ Roadmap

| Feature | Description | Status |
|---------|-------------|--------|
| `tp!` macro | HTML-like syntax for DOM trees | ✅ v0.1 |
| `use_state` | Reactive signals | ✅ v0.1 |
| Event handlers | `onclick`, `oninput`, `onkeydown` | ✅ v0.1 |
| `use_local_storage` | Persistent key/value hook | ✅ v0.2 |
| `use_router` | Hash-based navigation | ✅ v0.2 |
| `use_effect` | Side-effects hook (fetch, timers) | 🔲 v0.3 |
| Components | Function-based component system | 🔲 v0.3 |
| `use_memo` | Computed/derived signals | 🔲 v0.3 |

---

## 🤝 Contributing

Contributions are welcome! Please open an issue first to discuss what you'd like to change.

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/router`)
3. Make your changes + add tests
4. Open a PR

---

## 📄 License

MIT — see [LICENSE](LICENSE)
