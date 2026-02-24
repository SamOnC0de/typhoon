//! # Typhoon Core
//!
//! Lightweight Rust/WASM frontend framework for beginners.
//!
//! Build web UIs in pure Rust with the `tp!` macro — no JavaScript needed.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use typhoon_core::prelude::*;
//!
//! #[typhoon_core::main]
//! fn app() {
//!     let count = use_state(0u32);
//!
//!     let inc = {
//!         let count = count.clone();
//!         move || count.set(count.get() + 1)
//!     };
//!
//!     mount(tp! {
//!         div.class("app") {
//!             h1.text("Counter")
//!             button.onclick(inc) { "+" }
//!             p.text(count.get())
//!         }
//!     });
//! }
//! ```

use std::{cell::RefCell, rc::Rc};

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Text};

// Re-export the tp! macro
pub use typhoon_macro::tp;

// ─── Panic hook ──────────────────────────────────────────────────────────────

/// Call once at startup to get readable panic messages in the browser console.
pub fn init() {
    console_error_panic_hook::set_once();
}

// ─── DOM helpers (used by tp! generated code) ────────────────────────────────

fn document() -> Document {
    web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
}

/// Create a DOM element by tag name.
#[inline]
pub fn create_element(tag: &str) -> Element {
    document()
        .create_element(tag)
        .unwrap_or_else(|_| panic!("failed to create <{}>", tag))
}

/// Set the `textContent` of an element.
#[inline]
pub fn set_text_content(el: &Element, value: &dyn std::fmt::Display) {
    el.set_text_content(Some(&value.to_string()));
}

/// Set a CSS class string on an element.
#[inline]
pub fn set_class(el: &Element, class: &str) {
    el.set_class_name(class);
}

/// Set an inline style string on an element.
#[inline]
pub fn set_style(el: &Element, style: &str) {
    el.set_attribute("style", style)
        .expect("failed to set style");
}

/// Set an arbitrary HTML attribute.
#[inline]
pub fn set_attribute(el: &Element, name: &str, value: &dyn std::fmt::Display) {
    el.set_attribute(name, &value.to_string())
        .unwrap_or_else(|_| panic!("failed to set attribute {}", name));
}

/// Append a child element.
#[inline]
pub fn append_child(parent: &Element, child: &Element) {
    parent
        .append_child(child.as_ref())
        .expect("failed to append child");
}

/// Append a raw text node.
#[inline]
pub fn append_text_node(parent: &Element, text: &str) {
    let doc = document();
    let node: Text = doc.create_text_node(text);
    parent
        .append_child(node.as_ref())
        .expect("failed to append text node");
}

/// Attach a click handler to an element.
pub fn set_onclick<F: FnMut() + 'static>(el: &Element, mut handler: F) {
    let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
        handler();
    });
    el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .expect("failed to add click listener");
    closure.forget(); // keep alive
}

/// Attach an input handler to an element.
pub fn set_oninput<F: FnMut(String) + 'static>(el: &Element, mut handler: F) {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::InputEvent| {
        let target = event.target().expect("no target");
        let input: web_sys::HtmlInputElement = target.unchecked_into();
        handler(input.value());
    });
    el.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
        .expect("failed to add input listener");
    closure.forget();
}

/// Attach a keydown handler to an element.
pub fn set_onkeydown<F: FnMut(String) + 'static>(el: &Element, mut handler: F) {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
        handler(event.key());
    });
    el.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .expect("failed to add keydown listener");
    closure.forget();
}

// ─── Reactive State ───────────────────────────────────────────────────────────

type Subscriber = Box<dyn Fn()>;

struct SignalInner<T> {
    value: T,
    subscribers: Vec<Subscriber>,
}

/// A reactive signal. When you call `.set()`, all registered listeners
/// are notified automatically.
///
/// # Example
/// ```ignore
/// let count = use_state(0u32);
/// let count2 = count.clone();
/// count.subscribe(move || {
///     // re-render or update DOM here
/// });
/// count2.set(count2.get() + 1); // triggers subscriber
/// ```
pub struct Signal<T: Clone + 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

impl<T: Clone + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Clone + 'static> Signal<T> {
    fn new(value: T) -> Self {
        Signal {
            inner: Rc::new(RefCell::new(SignalInner {
                value,
                subscribers: Vec::new(),
            })),
        }
    }

    /// Get the current value (cloned).
    pub fn get(&self) -> T {
        self.inner.borrow().value.clone()
    }

    /// Set a new value and notify all subscribers.
    pub fn set(&self, value: T) {
        self.inner.borrow_mut().value = value;

        // Collect subscriber count first, then call each one.
        // We re-borrow each time so that a subscriber calling .set() again
        // (re-entrancy) works without panicking — new subscribers added during
        // notification are skipped for this round, which is fine for MVP.
        let len = self.inner.borrow().subscribers.len();
        for i in 0..len {
            // Clone the Rc so we don't hold the RefCell borrow while calling the fn.
            let rc = Rc::clone(&self.inner);
            // SAFETY (logical): we index within bounds checked above, and Subscriber
            // is a Box<dyn Fn()> stored in a Vec that grows but never shrinks.
            // The Box address is stable once inserted, so casting to a raw pointer
            // and calling after releasing the borrow is sound for our MVP use case.
            let fn_ptr: *const dyn Fn() = {
                let guard = rc.borrow();
                &*guard.subscribers[i] as *const dyn Fn()
            };
            // SAFETY: the Box is alive as long as `self.inner` (Rc) is alive,
            // and we're still holding an Rc clone (`rc`) throughout this call.
            unsafe { (*fn_ptr)() };
            drop(rc);
        }
    }

    /// Register a callback that runs whenever the value changes.
    pub fn subscribe<F: Fn() + 'static>(&self, f: F) {
        self.inner.borrow_mut().subscribers.push(Box::new(f));
    }
}

impl<T: Clone + std::fmt::Display + 'static> std::fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// Create a reactive state value.
///
/// # Example
/// ```ignore
/// let count = use_state(0u32);
/// let c = count.clone();
/// button.onclick(move || c.set(c.get() + 1));
/// ```
pub fn use_state<T: Clone + 'static>(initial: T) -> Signal<T> {
    Signal::new(initial)
}

// ─── Mount ────────────────────────────────────────────────────────────────────

/// Mount an element to `document.body`.
pub fn mount(el: Element) {
    let body = document().body().expect("document has no body");
    body.append_child(el.as_ref()).expect("failed to mount");
}

/// Mount an element to a specific DOM id.
pub fn mount_to(id: &str, el: Element) {
    let target = document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("no element with id #{}", id));
    target.append_child(el.as_ref()).expect("failed to mount");
}

// ─── LocalStorage hook ───────────────────────────────────────────────────────

/// Create a reactive signal backed by `localStorage`.
///
/// The value is loaded from `localStorage` on first call (falling back to
/// `default` if absent or malformed), and automatically persisted as JSON
/// every time `.set()` is called.
///
/// # Example
/// ```ignore
/// let todos: Signal<Vec<String>> = use_local_storage("todos", vec![]);
/// todos.set(vec!["Buy milk".into()]); // persisted immediately
/// ```
pub fn use_local_storage<T>(key: &'static str, default: T) -> Signal<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    let initial = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or(default);

    let signal = Signal::new(initial);

    let signal_for_sub = signal.clone();
    signal.subscribe(move || {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            if let Ok(json) = serde_json::to_string(&signal_for_sub.get()) {
                storage.set_item(key, &json).ok();
            }
        }
    });

    signal
}

// ─── Hash Router ─────────────────────────────────────────────────────────────

/// A lightweight hash-based router.
///
/// Routes are matched against `window.location.hash` (e.g. `"#/"`, `"#/about"`).
/// When none match, the first route is rendered as default.
///
/// Returns a container `Element` that you can `mount()` or embed anywhere.
///
/// # Example
/// ```ignore
/// let app = use_router(vec![
///     ("#/",       Box::new(|| tp! { h1.text("Home") })),
///     ("#/about",  Box::new(|| tp! { h1.text("About") })),
/// ]);
/// mount(app);
/// ```
pub fn use_router(routes: Vec<(&'static str, Box<dyn Fn() -> Element + 'static>)>) -> Element {
    let container = create_element("div");
    let routes: Rc<Vec<(&'static str, Box<dyn Fn() -> Element>)>> = Rc::new(routes);

    let container_render = container.clone();
    let routes_render = Rc::clone(&routes);

    let render: Rc<dyn Fn()> = Rc::new(move || {
        let hash = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .unwrap_or_default();
        let hash = if hash.is_empty() {
            String::from("#/")
        } else {
            hash
        };

        while let Some(child) = container_render.first_child() {
            container_render.remove_child(&child).ok();
        }

        let mut matched = false;
        for (path, handler) in routes_render.iter() {
            if hash == *path {
                let el = handler();
                container_render.append_child(el.as_ref()).ok();
                matched = true;
                break;
            }
        }

        if !matched {
            if let Some((_, handler)) = routes_render.first() {
                let el = handler();
                container_render.append_child(el.as_ref()).ok();
            }
        }
    });

    // Initial render
    render();

    // Listen for hashchange events
    let render_for_event = Rc::clone(&render);
    let closure =
        Closure::<dyn FnMut(_)>::new(move |_event: web_sys::HashChangeEvent| {
            render_for_event();
        });
    web_sys::window()
        .expect("no window")
        .add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref())
        .expect("failed to add hashchange listener");
    closure.forget();

    container
}

// ─── Prelude ──────────────────────────────────────────────────────────────────

pub mod prelude {
    pub use super::{init, mount, mount_to, tp, use_local_storage, use_router, use_state, Signal};
}
