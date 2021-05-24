/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
extern crate elisp;
extern crate wasm_bindgen;
extern crate web_sys;

use elisp::create_error;
use elisp::draw::DrawArc;
use elisp::draw::DrawImage;
use elisp::draw::DrawLine;
use elisp::lisp::ErrCode;
use elisp::lisp::Error;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::Document;
use web_sys::HtmlImageElement;

//--------------------------------------------------------
// Graphics (ex. background color)
//--------------------------------------------------------
pub struct Graphics {
    pub bg: Option<JsValue>,
}
impl Graphics {
    pub fn new() -> Graphics {
        Graphics { bg: None }
    }
}
impl Default for Graphics {
    fn default() -> Self {
        Self::new()
    }
}
// ----------------------------------------------------------------
// draw string
// ----------------------------------------------------------------
pub fn create_draw_string(
    context: &CanvasRenderingContext2d,
) -> Box<dyn Fn(String, f64, f64, String) + 'static> {
    let ctx = context.clone();
    let draw_string = move |s: String, x, y, f: String| {
        ctx.set_font(&f);
        ctx.fill_text(&s, x, y).unwrap();
    };
    Box::new(draw_string)
}
// ----------------------------------------------------------------
// draw line
// ----------------------------------------------------------------
pub fn create_draw_line(context: &CanvasRenderingContext2d) -> DrawLine {
    let ctx = context.clone();

    let draw_line = move |x1, y1, x2, y2| {
        ctx.begin_path();
        ctx.move_to(x1, y1);
        ctx.line_to(x2, y2);
        ctx.close_path();
        ctx.stroke();
    };
    Box::new(draw_line)
}
// ----------------------------------------------------------------
// draw arc
// ----------------------------------------------------------------
pub fn create_draw_arc(context: &CanvasRenderingContext2d) -> DrawArc {
    let ctx = context.clone();

    let draw_arc = move |x, y, r, angle| {
        ctx.begin_path();
        ctx.arc(x, y, r, angle, std::f64::consts::PI * 2.0).unwrap();
        ctx.stroke();
    };
    Box::new(draw_arc)
}
// ----------------------------------------------------------------
// draw arc
// ----------------------------------------------------------------
pub fn create_draw_image(context: &CanvasRenderingContext2d, document: &Document) -> DrawImage {
    let ctx = context.clone();
    let doc = document.clone();

    let draw_image = move |x0, y0, x1, y1, xorg, yorg, symbol: &String| {
        let img = match doc.get_element_by_id(symbol) {
            Some(e) => e.dyn_into::<HtmlImageElement>().unwrap(),
            None => return Err(create_error!(ErrCode::E1008)),
        };
        let w = img.width() as f64;
        let h = img.height() as f64;

        if ctx
            .set_transform(x0 / w, y0 / h, x1 / w, y1 / h, xorg, yorg)
            .is_err()
        {
            return Err(create_error!(ErrCode::E9999));
        }
        // https://rustwasm.github.io/wasm-bindgen/api/web_sys/
        if ctx
            .draw_image_with_html_image_element(&img, 0.0, 0.0)
            .is_err()
        {
            return Err(create_error!(ErrCode::E9999));
        }
        if ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).is_err() {
            return Err(create_error!(ErrCode::E9999));
        }
        Ok(())
    };
    Box::new(draw_image)
}
