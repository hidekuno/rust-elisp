/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use elisp::draw::DrawLine;
use elisp::draw::Fractal;
use elisp::lisp::Error;

pub struct Dragon {
    draw_line: DrawLine,
    max: i32,
}
impl Dragon {
    pub fn new(draw_line: DrawLine) -> Self {
        Dragon { draw_line, max: 20 }
    }
    pub fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, c: i32) -> Result<(), Error> {
        let xx = x1 - x0;
        let yy = (y1 - y0) * -1.0;
        let xc = x0 + (xx + yy) / 2.0;
        let yc = y1 + (xx + yy) / 2.0;

        if 0 >= c {
            (self.draw_line)(x0, y0, xc, yc)?;
            (self.draw_line)(x1, y1, xc, yc)?;
        } else {
            self.draw(x0, y0, xc, yc, c - 1)?;
            self.draw(x1, y1, xc, yc, c - 1)?;
        }
        Ok(())
    }
}
impl Fractal for Dragon {
    fn get_func_name(&self) -> &'static str {
        "draw-dragon"
    }
    fn get_max(&self) -> i32 {
        self.max
    }
    fn do_demo(&self, c: i32) -> Result<(), Error> {
        self.draw(0.2777777777777778, 0.25, 0.5972222222222222, 0.625, c)?;
        Ok(())
    }
}
