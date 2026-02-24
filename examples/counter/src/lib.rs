use typhoon_core::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    init();

    let count = use_state(0i32);

    let display = tp! { p.text(count.get()).style("font-size:2rem;margin:1rem 0") };
    let display_ref = display.clone();
    let count_sub = count.clone();
    count.subscribe(move || {
        display_ref.set_text_content(Some(&count_sub.get().to_string()));
    });

    let count_inc = count.clone();
    let on_inc = move || count_inc.set(count_inc.get() + 1);

    let count_dec = count.clone();
    let on_dec = move || count_dec.set(count_dec.get() - 1);

    let count_rst = count.clone();
    let on_rst = move || count_rst.set(0);

    let app = tp! {
        div.class("app").style("font-family:sans-serif;text-align:center;padding:2rem") {
            h1.text("🌀 Typhoon Counter")
            div.style("display:flex;gap:1rem;justify-content:center;align-items:center") {
                button.onclick(on_dec).style("font-size:1.5rem;padding:.5rem 1.2rem;cursor:pointer") { "−" }
                button.onclick(on_rst).style("font-size:1rem;padding:.5rem 1rem;cursor:pointer") { "Reset" }
                button.onclick(on_inc).style("font-size:1.5rem;padding:.5rem 1.2rem;cursor:pointer") { "+" }
            }
        }
    };

    app.append_child(display.as_ref()).unwrap();
    mount(app);
}
