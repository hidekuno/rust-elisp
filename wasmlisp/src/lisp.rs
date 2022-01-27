/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use crate::alert;
use crate::console_log;
use crate::log;

use crate::buildin;
use crate::fractal;
use buildin::build_lisp_function;
use fractal::build_demo_function;

use elisp::lisp;
use lisp::do_core_logic;
use lisp::Environment;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;

use js_sys::Promise;
use web_sys::{Element, Event, HtmlTextAreaElement};

use crate::add_loading;
use crate::set_ace_text;
use crate::set_textarea_from_ace;

const SOURCE_BUTTONS: [(&str, &str); 4] = [
    ("sicp", "(load-url \"wasm-sicp.scm\")"),
    (
        "demo",
        "(draw-clear)(define (draw-line-vect s e)(draw-line s e))(demo)",
    ),
    (
        "anime",
        "(draw-clear)(define (draw-line-vect s e)(add-timeout (draw-line s e) 10))(demo)",
    ),
    ("album",
     "(draw-clear)((square-limit (below(beside rv ps)(beside sd am)) 0)(make-image-frame-rectangle \"am\" 1.74 1.74))",
    ),
];
//--------------------------------------------------------
// entry point
//--------------------------------------------------------
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let button = document
        .get_element_by_id("eval")
        .unwrap()
        .dyn_into::<Element>()
        .unwrap();

    let text = document
        .get_element_by_id("codearea")
        .unwrap()
        .dyn_into::<HtmlTextAreaElement>()
        .unwrap();

    let env = Environment::new();
    build_lisp_function(&env, &document);
    build_demo_function(&env, &document);

    // evalButton.onmousedown = () => {...}
    let closure = Closure::wrap(Box::new(move |_event: Event| {
        add_loading();

        set_textarea_from_ace();
    }) as Box<dyn FnMut(_)>);
    button.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
    closure.forget();

    // evalButton.onclick = () => {...}
    let closure = Closure::wrap(Box::new(move |_event: Event| {
        let c = Closure::wrap(Box::new(move |v: JsValue| {
            alert(&v.as_string().unwrap());
        }) as Box<dyn FnMut(_)>);

        // It's experimental code for study.
        let _ = future_to_promise(execute_lisp(text.value(), env.clone())).then(&c);
        c.forget();

        if let Some(element) = document.get_element_by_id("loading") {
            let loading = element.dyn_into::<Element>().unwrap();
            loading.remove();
        }
        console_log!("eval done.");
    }) as Box<dyn FnMut(_)>);
    button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;

    // The instance of `Closure` that we created will invalidate its
    // corresponding JS callback whenever it is dropped, so if we were to
    // normally return from `setup_clock` then our registered closure will
    // raise an exception when invoked.
    //
    // Normally we'd store the handle to later get dropped at an appropriate
    // time but for now we want it to be a global handler so we use the
    // `forget` method to drop it without invalidating the closure. Note that
    // this is leaking memory in Rust, so this should be done judiciously!
    closure.forget();

    make_source_button_callback();

    console_log!("Hello World from Rust");
    Ok(())
}
//--------------------------------------------------------
// It's experimental code for study.
//--------------------------------------------------------
async fn execute_lisp(code: String, env: Environment) -> Result<JsValue, JsValue> {
    fn call_elisp(code: String, env: Environment) -> JsValue {
        let v = match do_core_logic(&code, &env) {
            Ok(r) => r.to_string(),
            Err(e) => e.get_msg(),
        };
        JsValue::from(v)
    }
    let text = JsFuture::from(Promise::resolve(&call_elisp(code, env))).await?;
    Ok(text)
}
//--------------------------------------------------------
// make callback
//--------------------------------------------------------
fn make_source_button_callback() {
    let document = web_sys::window().unwrap().document().unwrap();

    for btn in &SOURCE_BUTTONS {
        let button = document
            .get_element_by_id(btn.0)
            .unwrap()
            .dyn_into::<Element>()
            .unwrap();

        // evalButton.onmousedown = () => {...}
        let closure = Closure::wrap(Box::new(move |_event: Event| {
            set_ace_text(btn.1);
        }) as Box<dyn FnMut(_)>);
        button
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}
