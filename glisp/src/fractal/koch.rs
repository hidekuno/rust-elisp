/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use elisp::draw::DrawLine;
use elisp::draw::Fractal;
use elisp::lisp::Error;

pub struct Koch {
    sin60: f64,
    cos60: f64,
    draw_line: DrawLine,
    max: i32,
}
impl Koch {
    pub fn new(draw_line: DrawLine) -> Self {
        Koch {
            sin60: ((std::f64::consts::PI * 60.0) / 180.0).sin(),
            cos60: ((std::f64::consts::PI * 60.0) / 180.0).cos(),
            draw_line,
            max: 12,
        }
    }
    pub fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, c: i32) -> Result<(), Error> {
        if c > 1 {
            let xa = (x0 * 2.0 + x1) / 3.0;
            let ya = (y0 * 2.0 + y1) / 3.0;
            let xb = (x1 * 2.0 + x0) / 3.0;
            let yb = (y1 * 2.0 + y0) / 3.0;

            let yc = ya + (xb - xa) * self.sin60 + (yb - ya) * self.cos60;
            let xc = xa + (xb - xa) * self.cos60 - (yb - ya) * self.sin60;

            self.draw(x0, y0, xa, ya, c - 1)?;
            self.draw(xa, ya, xc, yc, c - 1)?;
            self.draw(xc, yc, xb, yb, c - 1)?;
            self.draw(xb, yb, x1, y1, c - 1)?;
        } else {
            (self.draw_line)(x0, y0, x1, y1)?;
        }
        Ok(())
    }
}
impl Fractal for Koch {
    fn get_func_name(&self) -> &'static str {
        "draw-koch"
    }
    fn get_max(&self) -> i32 {
        self.max
    }
    fn do_demo(&self, c: i32) -> Result<(), Error> {
        self.draw(
            0.3597222222222222,
            0.0,
            0.04722222222222222,
            0.6964285714285714,
            c,
        )?;
        self.draw(
            0.04722222222222222,
            0.6964285714285714,
            0.6708333333333333,
            0.6964285714285714,
            c,
        )?;
        self.draw(
            0.6708333333333333,
            0.6964285714285714,
            0.3597222222222222,
            0.0,
            c,
        )?;
        Ok(())
    }
}
