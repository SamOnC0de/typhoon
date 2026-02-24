//! Lightweight Rust/WASM frontend framework.

use std::{cell::RefCell, rc::Rc};

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Text};

pub use typhoon_macro::tp;

/// Call once at startup to get readable panic messages in the browser console.
pub fn init() {
    console_error_panic_hook::set_once();
}

fn document() -> Document {
    web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
}

#[inline]
pub fn create_element(tag: &str) -> Element {
    document()
        .create_element(tag)
        .unwrap_or_else(|_| panic!("failed to create <{}>", tag))
}

#[inline]
pub fn set_text_content(el: &Element, value: &dyn std::fmt::Display) {
    el.set_text_content(Some(&value.to_string()));
}

#[inline]
pub fn set_class(el: &Element, class: &str) {
    el.set_class_name(class);
}

#[inline]
pub fn set_style(el: &Element, style: &str) {
    el.set_attribute("style", style)
        .expect("failed to set style");
}

#[inline]
pub fn set_attribute(el: &Element, name: &str, value: &dyn std::fmt::Display) {
    el.set_attribute(name, &value.to_string())
        .unwrap_or_else(|_| panic!("failed to set attribute {}", name));
}

#[inline]
pub fn append_child(parent: &Element, child: &Element) {
    parent
        .append_child(child.as_ref())
        .expect("failed to append child");
}

#[inline]
pub fn append_text_node(parent: &Element, text: &str) {
    let doc = document();
    let node: Text = doc.create_text_node(text);
    parent
        .append_child(node.as_ref())
        .expect("failed to append text node");
}

pub fn set_onclick<F: FnMut() + 'static>(el: &Element, mut handler: F) {
    let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::MouseEvent| {
        handler();
    });
    el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .expect("failed to add click listener");
    closure.forget();
}

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

pub fn set_onkeydown<F: FnMut(String) + 'static>(el: &Element, mut handler: F) {
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
        handler(event.key());
    });
    el.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .expect("failed to add keydown listener");
    closure.forget();
}

// ── Signal ────────────────────────────────────────────────────────────────────

type Subscriber = Box<dyn Fn()>;

struct SignalInner<T> {
    value: T,
    subscribers: Vec<Subscriber>,
}

/// Reactive value. Cloning shares the same underlying state.
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

    /// Returns the current value (cloned).
    pub fn get(&self) -> T {
        self.inner.borrow().value.clone()
    }

    /// Updates the value and notifies all subscribers.
    pub fn set(&self, value: T) {
        self.inner.borrow_mut().value = value;

        // Index-based loop + raw pointer so a subscriber calling .set() again
        // (re-entrant) doesn't panic on the RefCell borrow.
        // SAFETY: Box<dyn Fn()> address is stable in a Vec that only grows;
        // the Rc clone keeps it alive for the duration of the call.
        let len = self.inner.borrow().subscribers.len();
        for i in 0..len {
            let rc = Rc::clone(&self.inner);
            let fn_ptr: *const dyn Fn() = {
                let guard = rc.borrow();
                &*guard.subscribers[i] as *const dyn Fn()
            };
            unsafe { (*fn_ptr)() };
            drop(rc);
        }
    }

    /// Registers a callback that runs on every value change.
    pub fn subscribe<F: Fn() + 'static>(&self, f: F) {
        self.inner.borrow_mut().subscribers.push(Box::new(f));
    }
}

impl<T: Clone + std::fmt::Display + 'static> std::fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// Creates a reactive state value.
pub fn use_state<T: Clone + 'static>(initial: T) -> Signal<T> {
    Signal::new(initial)
}

// ── Mount ─────────────────────────────────────────────────────────────────────

/// Mounts an element to `document.body`.
pub fn mount(el: Element) {
    let body = document().body().expect("document has no body");
    body.append_child(el.as_ref()).expect("failed to mount");
}

/// Mounts an element to a specific DOM id.
pub fn mount_to(id: &str, el: Element) {
    let target = document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("no element with id #{}", id));
    target.append_child(el.as_ref()).expect("failed to mount");
}

// ── Effects ───────────────────────────────────────────────────────────────────

/// Runs a one-shot side-effect after the current render (next event-loop tick).
pub fn use_effect<F: FnOnce() + 'static>(f: F) {
    let f = Rc::new(RefCell::new(Some(f)));
    let closure = Closure::<dyn FnMut()>::new(move || {
        if let Some(f) = f.borrow_mut().take() {
            f();
        }
    });
    web_sys::window()
        .expect("no window")
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            0,
        )
        .expect("failed to schedule effect");
    closure.forget();
}

/// Handle to a running interval. Cleared on drop; call `.forget()` to keep it alive.
pub struct IntervalHandle(i32);

impl IntervalHandle {
    /// Prevents the interval from being cancelled when the handle is dropped.
    pub fn forget(self) {
        std::mem::forget(self);
    }
}

impl Drop for IntervalHandle {
    fn drop(&mut self) {
        if let Some(w) = web_sys::window() {
            w.clear_interval_with_handle(self.0);
        }
    }
}

/// Runs a callback every `ms` milliseconds. Returns an [`IntervalHandle`].
pub fn use_interval<F: FnMut() + 'static>(callback: F, ms: i32) -> IntervalHandle {
    let closure = Closure::<dyn FnMut()>::new(callback);
    let id = web_sys::window()
        .expect("no window")
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            ms,
        )
        .expect("failed to set interval");
    closure.forget();
    IntervalHandle(id)
}

/// Spawns an async block on the WASM executor.
pub use wasm_bindgen_futures::spawn_local;

// ── Local storage ─────────────────────────────────────────────────────────────

/// Reactive signal backed by `localStorage`. Persists as JSON on every `.set()`.
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

// ── Hash router ───────────────────────────────────────────────────────────────

/// Hash-based router. Renders the matching route into a container element.
///
/// Routes are matched against `window.location.hash` (e.g. `"#/"`, `"#/about"`).
/// Falls back to the first route when no match is found.
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

    render();

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

// ── Memo ──────────────────────────────────────────────────────────────────────

/// Implemented for `Signal<T>` and tuples of up to three signals.
pub trait Deps {
    fn on_change<F: Fn() + 'static>(&self, f: F);
}

impl<T: Clone + 'static> Deps for Signal<T> {
    fn on_change<F: Fn() + 'static>(&self, f: F) {
        self.subscribe(f);
    }
}

impl<T1, T2> Deps for (Signal<T1>, Signal<T2>)
where
    T1: Clone + 'static,
    T2: Clone + 'static,
{
    fn on_change<F: Fn() + 'static>(&self, f: F) {
        let f = Rc::new(f);
        let f1 = Rc::clone(&f);
        self.0.subscribe(move || f1());
        self.1.subscribe(move || f());
    }
}

impl<T1, T2, T3> Deps for (Signal<T1>, Signal<T2>, Signal<T3>)
where
    T1: Clone + 'static,
    T2: Clone + 'static,
    T3: Clone + 'static,
{
    fn on_change<F: Fn() + 'static>(&self, f: F) {
        let f = Rc::new(f);
        let f1 = Rc::clone(&f);
        let f2 = Rc::clone(&f);
        self.0.subscribe(move || f1());
        self.1.subscribe(move || f2());
        self.2.subscribe(move || f());
    }
}

/// Computed signal that re-evaluates whenever a dependency changes.
pub fn use_memo<T, D, F>(deps: D, compute: F) -> Signal<T>
where
    T: Clone + 'static,
    D: Deps,
    F: Fn() -> T + 'static,
{
    let compute = Rc::new(compute);
    let result = Signal::new(compute());
    let result_clone = result.clone();
    let compute_clone = Rc::clone(&compute);
    deps.on_change(move || {
        result_clone.set(compute_clone());
    });
    result
}

// ── Components ────────────────────────────────────────────────────────────────

/// Struct-based component. For most cases, plain functions returning `Element` are simpler.
pub trait Component {
    fn render(self) -> Element;
}

// ── Prelude ───────────────────────────────────────────────────────────────────

pub mod prelude {
    pub use super::{
        init, mount, mount_to, spawn_local, tp,
        use_effect, use_interval, use_local_storage, use_router, use_state,
        Component, Deps, IntervalHandle, Signal,
        use_memo,
    };
}
