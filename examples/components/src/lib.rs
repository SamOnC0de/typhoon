use typhoon_core::prelude::*;
use wasm_bindgen::prelude::*;

// Stateless components — plain functions returning Element

fn badge(label: &str) -> Element {
    tp! {
        span.text(label)
            .style("font-size:.7rem;padding:2px 8px;border-radius:999px;\
                    background:#313244;color:#cba6f7;font-weight:600")
    }
}

fn card(title: &str, body: &str, tag: &str) -> Element {
    let el = tp! {
        div.style("background:#1e1e2e;border:1px solid #313244;border-radius:10px;\
                   padding:1.2rem;display:flex;flex-direction:column;gap:.6rem")
    };
    let h = tp! { h3.text(title).style("margin:0;font-size:.95rem;color:#cdd6f4") };
    let p = tp! { p.text(body).style("margin:0;color:#6c7086;font-size:.85rem;flex:1") };
    el.append_child(h.as_ref()).unwrap();
    el.append_child(p.as_ref()).unwrap();
    el.append_child(badge(tag).as_ref()).unwrap();
    el
}

// Stateful component — each call gets its own Signal

fn mini_counter(label: &str) -> Element {
    let count = use_state(0i32);

    let display = tp! {
        span.style("font-family:monospace;min-width:80px;display:inline-block;text-align:center")
    };
    let disp_ref = display.clone();
    let count_sub = count.clone();
    let lbl = label.to_string();
    display.set_text_content(Some(&format!("{}: 0", lbl)));
    count.subscribe(move || {
        disp_ref.set_text_content(Some(&format!("{}: {}", lbl, count_sub.get())));
    });

    let count_inc = count.clone();
    let count_dec = count.clone();
    let btn_style = "padding:2px 10px;cursor:pointer;background:#313244;\
                     color:#cdd6f4;border:none;border-radius:4px;font-size:1rem";

    let dec = tp! {
        button.onclick(move || count_dec.set(count_dec.get() - 1))
              .style(btn_style) { "−" }
    };
    let inc = tp! {
        button.onclick(move || count_inc.set(count_inc.get() + 1))
              .style(btn_style) { "+" }
    };

    let row = tp! {
        div.style("display:flex;align-items:center;gap:.6rem;padding:.5rem 0;\
                   border-bottom:1px solid #313244")
    };
    row.append_child(dec.as_ref()).unwrap();
    row.append_child(display.as_ref()).unwrap();
    row.append_child(inc.as_ref()).unwrap();
    row
}

#[wasm_bindgen(start)]
pub fn main() {
    init();

    let app = tp! {
        div.class("app")
           .style("font-family:sans-serif;padding:2rem;max-width:600px;\
                   margin:0 auto;color:#cdd6f4") {

            h1.text("🧩 Typhoon Components")
               .style("color:#cba6f7;margin-bottom:.4rem")
            p.text("Components are plain Rust functions that return an Element.")
             .style("color:#6c7086;font-size:.9rem;margin-bottom:1.5rem")

            h2.text("Stateless components")
               .style("font-size:1rem;color:#a6e3a1;margin-bottom:.6rem")
        }
    };

    let grid = tp! {
        div.style("display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));\
                   gap:.8rem;margin-bottom:1.5rem")
    };
    grid.append_child(card("Counter",    "Reactive signals",        "use_state").as_ref()).unwrap();
    grid.append_child(card("Todo",       "Persistent list",         "use_local_storage").as_ref()).unwrap();
    grid.append_child(card("Clock",      "Ticking timer",           "use_interval").as_ref()).unwrap();
    grid.append_child(card("Router",     "Hash-based navigation",   "use_router").as_ref()).unwrap();

    app.append_child(grid.as_ref()).unwrap();

    let section = tp! {
        div {
            h2.text("Stateful components — each counter is independent")
               .style("font-size:1rem;color:#a6e3a1;margin-bottom:.4rem")
            p.text("Each call to mini_counter() creates its own Signal — no shared state.")
             .style("color:#6c7086;font-size:.85rem;margin-bottom:.6rem")
        }
    };

    let counters = tp! {
        div {
            (mini_counter("Alpha"))
            (mini_counter("Beta"))
            (mini_counter("Gamma"))
        }
    };

    section.append_child(counters.as_ref()).unwrap();
    app.append_child(section.as_ref()).unwrap();

    mount(app);
}
