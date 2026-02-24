use typhoon_core::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn main() {
    init();

    let todos: Signal<Vec<String>> = use_local_storage("todos", vec![]);
    let input_val: Signal<String> = use_state(String::new());

    let list = tp! { ul.style("list-style:none;padding:0;margin:1rem 0;max-width:400px") };
    let list_ref = list.clone();

    let todos_for_sub = todos.clone();
    todos.subscribe(move || {
        while let Some(child) = list_ref.first_child() {
            list_ref.remove_child(&child).ok();
        }
        for (i, item) in todos_for_sub.get().iter().enumerate() {
            let li = tp! {
                li.style("display:flex;align-items:center;gap:.5rem;padding:.4rem 0;border-bottom:1px solid #313244")
            };
            let span = tp! { span.text(item) };
            let todos_del = todos_for_sub.clone();
            let idx = i;
            let del_btn = tp! { button.onclick(move || {
                let mut v = todos_del.get();
                v.remove(idx);
                todos_del.set(v);
            }).style("margin-left:auto;cursor:pointer;background:#313244;color:#f38ba8;border:none;border-radius:4px;padding:2px 8px") };
            del_btn.set_text_content(Some("✕"));
            li.append_child(span.as_ref()).unwrap();
            li.append_child(del_btn.as_ref()).unwrap();
            list_ref.append_child(li.as_ref()).unwrap();
        }
    });

    let add_todo = {
        let todos = todos.clone();
        let input_val = input_val.clone();
        move || {
            let val = input_val.get();
            let trimmed = val.trim().to_string();
            if !trimmed.is_empty() {
                let mut v = todos.get();
                v.push(trimmed);
                todos.set(v);
                input_val.set(String::new());
            }
        }
    };

    let input_val_for_input = input_val.clone();
    let add_for_input = add_todo.clone();

    let inp = tp! {
        input
            .placeholder(&"Add a task…")
            .style("flex:1;padding:.5rem .8rem;background:#1e1e2e;color:#cdd6f4;border:1px solid #45475a;border-radius:6px;font-size:1rem")
            .oninput(move |v: String| input_val_for_input.set(v))
            .onkeydown(move |key: String| { if key == "Enter" { add_for_input(); } })
    };

    // Sync the DOM value property so the field clears after adding
    let inp_ref = inp.clone();
    let input_val_sub = input_val.clone();
    input_val.subscribe(move || {
        inp_ref.set_attribute("value", &input_val_sub.get()).ok();
        let html_input: web_sys::HtmlInputElement = inp_ref.clone().dyn_into().unwrap();
        html_input.set_value(&input_val_sub.get());
    });

    let app = tp! {
        div.class("app").style("font-family:sans-serif;padding:2rem;max-width:480px;margin:0 auto") {
            h1.text("🌀 Typhoon Todo").style("margin-bottom:1rem")
        }
    };

    let row = tp! { div.style("display:flex;gap:.5rem") };

    let add_btn = tp! {
        button
            .onclick(add_todo)
            .style("padding:.5rem 1rem;cursor:pointer;background:#cba6f7;color:#1e1e2e;border:none;border-radius:6px;font-weight:bold")
    };
    add_btn.set_text_content(Some("Add"));

    row.append_child(inp.as_ref()).unwrap();
    row.append_child(add_btn.as_ref()).unwrap();
    app.append_child(row.as_ref()).unwrap();
    app.append_child(list.as_ref()).unwrap();

    mount(app);
}
