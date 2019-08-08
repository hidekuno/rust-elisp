/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use crate::draw::DrawLine;

pub struct Koch {
    sin60: f64,
    cos60: f64,
    scale: i64,
    draw_line: DrawLine,
}
impl Koch {
    pub fn new(c: i64, draw_line: DrawLine) -> Koch {
        Koch {
            sin60: ((std::f64::consts::PI * 60.0) / 180.0).sin(),
            cos60: ((std::f64::consts::PI * 60.0) / 180.0).cos(),
            scale: c,
            draw_line: draw_line,
        }
    }
    pub fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, c: i64) {
        if c > 1 {
            let xa = (x0 * 2.0 + x1) / 3.0;
            let ya = (y0 * 2.0 + y1) / 3.0;
            let xb = (x1 * 2.0 + x0) / 3.0;
            let yb = (y1 * 2.0 + y0) / 3.0;

            let yc = ya + (xb - xa) * self.sin60 + (yb - ya) * self.cos60;
            let xc = xa + (xb - xa) * self.cos60 - (yb - ya) * self.sin60;

            self.draw(x0, y0, xa, ya, c - 1);
            self.draw(xa, ya, xc, yc, c - 1);
            self.draw(xc, yc, xb, yb, c - 1);
            self.draw(xb, yb, x1, y1, c - 1);
        } else {
            (self.draw_line)(x0, y0, x1, y1);
        }
    }
    pub fn do_demo(&self) {
        self.draw(
            0.3597222222222222,
            0.0,
            0.04722222222222222,
            0.6964285714285714,
            self.scale,
        );
        self.draw(
            0.04722222222222222,
            0.6964285714285714,
            0.6708333333333333,
            0.6964285714285714,
            self.scale,
        );
        self.draw(
            0.6708333333333333,
            0.6964285714285714,
            0.3597222222222222,
            0.0,
            self.scale,
        );
    }
}
