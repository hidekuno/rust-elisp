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
use crate::helper;
use buildin::build_lisp_function;
use fractal::build_demo_function;

use elisp::lisp;
use helper::History;
use helper::HISTORY_SIZE;
use lisp::do_core_logic;
use lisp::Environment;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;

use js_sys::Promise;
use web_sys::{
    Document, Element, Event, HtmlDivElement, HtmlOptionElement, HtmlSelectElement,
    HtmlTextAreaElement,
};

use crate::get_ace_text;
use crate::init_ace;
use crate::set_ace_text;

macro_rules! make_code {
    ($s1: expr, $s2: expr ) => {
        concat!("(draw-clear)", "\n", $s1, "\n", $s2)
    };
}

const SOURCE_BUTTONS: [(&str, &str); 4] = [
    ("sicp", "(load-url \"wasm-sicp.scm\")"),
    (
        "demo",
        make_code!("(define (draw-line-vect s e)(draw-line s e))", "(demo)"),
    ),
    (
        "anime",
        make_code!(
            "(define (draw-line-vect s e)(add-timeout (draw-line s e) 10))",
            "(demo)"
        ),
    ),
    (
        "album",
        make_code!(
            "((square-limit (below(beside rv ps)(beside sd am)) 0)",
            " (make-image-frame-rectangle \"am\" 1.74 1.74))"
        ),
    ),
];

const WAIT_DIALOG_ID: &str = "loading";
const WEB_FONT: &str = "<i class='fa fa-spinner fa-spin fa-5x fa-fw'></i><br><br>";
const WAIT_MESSAGE: &str = "Please wait until the alert dialog is displayed.";
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

    init_ace();

    let history = History::new(HISTORY_SIZE);
    make_select_options(&document, &history);

    let commands = history.clone();
    // selectElement.addEventListener('change',
    {
        let select = document
            .get_element_by_id("history-code")
            .unwrap()
            .dyn_into::<HtmlSelectElement>()
            .unwrap();

        let closure = Closure::wrap(Box::new(move |event: Event| {
            if let Some(target) = event.target() {
                let select = target.dyn_into::<HtmlSelectElement>().unwrap();
                if let Ok(n) = select.value().parse::<usize>() {
                    if let Some(s) = commands.get_value(n) {
                        console_log!("{:?}", &s);
                        set_ace_text(&s);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        select.set_onchange(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
    // evalButton.onmousedown = () => {...}
    {
        let mut document = document.clone();
        let text = text.clone();
        let closure = Closure::wrap(Box::new(move |_event: Event| {
            if document.get_element_by_id(WAIT_DIALOG_ID).is_none() {
                add_loading(&mut document);
            }

            let s = get_ace_text();
            text.set_value(&s);
        }) as Box<dyn FnMut(_)>);
        button.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    // evalButton.onclick = () => {...}
    let closure = Closure::wrap(Box::new(move |_event: Event| {
        let doc = document.clone();

        let c = Closure::wrap(Box::new(move |v: JsValue| {
            if let Some(element) = doc.get_element_by_id(WAIT_DIALOG_ID) {
                let loading = element.dyn_into::<Element>().unwrap();
                loading.remove();
            }
            alert(&v.as_string().unwrap());
        }) as Box<dyn FnMut(_)>);

        history.push(&text.value());
        make_select_options(&document, &history);

        // It's experimental code for study.
        let _ = future_to_promise(execute_lisp(text.value(), env.clone())).then(&c);
        c.forget();

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
// add loading message
//--------------------------------------------------------
fn add_loading(document: &mut Document) {
    let div = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlDivElement>()
        .unwrap();

    div.set_id(WAIT_DIALOG_ID);

    let ua = web_sys::window().unwrap().navigator().user_agent().unwrap();
    let ua = ua.to_lowercase();

    let msg = if !ua.contains("firefox") {
        format!("<div class='layout icon'>{}</div>", WAIT_MESSAGE)
    } else {
        format!("<div class='layout'>{}{}</div>", WEB_FONT, WAIT_MESSAGE)
    };
    div.set_inner_html(&msg);
    document.body().unwrap().append_child(&div).unwrap();
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

        // evalButton.onclick = () => {...}
        let closure = Closure::wrap(Box::new(move |_event: Event| {
            set_ace_text(btn.1);
        }) as Box<dyn FnMut(_)>);
        button
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}
//--------------------------------------------------------
// make select options
//--------------------------------------------------------
fn make_select_options(document: &Document, history: &History) {
    let select = document
        .get_element_by_id("history-code")
        .unwrap()
        .dyn_into::<HtmlSelectElement>()
        .unwrap();

    select.set_inner_html("");
    let option = document
        .create_element("option")
        .unwrap()
        .dyn_into::<HtmlOptionElement>()
        .unwrap();
    option.set_text("Code History");
    option.set_value("-1");
    select.append_child(&option).unwrap();

    history.walk_inner(|i, e| {
        let option = document
            .create_element("option")
            .unwrap()
            .dyn_into::<HtmlOptionElement>()
            .unwrap();

        option.set_text(e);
        option.set_value(&i.to_string());
        select.append_child(&option).unwrap();
    });
}
