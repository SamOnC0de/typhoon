use js_sys::Date;
use typhoon_core::prelude::*;
use wasm_bindgen::prelude::*;

fn current_time() -> String {
    let d = Date::new_0();
    format!(
        "{:02}:{:02}:{:02}",
        d.get_hours(),
        d.get_minutes(),
        d.get_seconds()
    )
}

#[wasm_bindgen(start)]
pub fn main() {
    init();

    let time = use_state(current_time());

    let display = tp! {
        p.text(time.get())
         .style("font-size:4rem;font-family:monospace;letter-spacing:.1em;margin:1rem 0")
    };
    let disp_ref = display.clone();
    let time_sub = time.clone();
    time.subscribe(move || {
        disp_ref.set_text_content(Some(&time_sub.get()));
    });

    let time_tick = time.clone();
    use_interval(
        move || {
            time_tick.set(current_time());
        },
        1000,
    )
    .forget();

    let app = tp! {
        div.class("app")
           .style("font-family:sans-serif;text-align:center;padding:3rem;max-width:480px;margin:0 auto") {
            h1.text("🕐 Typhoon Clock").style("margin-bottom:.5rem")
            p.text("Built with use_interval — no JavaScript")
             .style("color:#6c7086;font-size:.9rem;margin-bottom:1.5rem")
        }
    };
    app.append_child(display.as_ref()).unwrap();
    mount(app);
}
