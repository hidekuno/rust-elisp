/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use crate::draw::DrawLine;

pub struct Dragon {
    scale: i64,
    draw_line: DrawLine,
}
impl Dragon {
    pub fn new(c: i64, draw_line: DrawLine) -> Dragon {
        Dragon {
            scale: c,
            draw_line: draw_line,
        }
    }
    pub fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, c: i64) {
        let xx = x1 - x0;
        let yy = (y1 - y0) * -1.0;
        let xc = x0 + (xx + yy) / 2.0;
        let yc = y1 + (xx + yy) / 2.0;

        if 0 >= c {
            (self.draw_line)(x0, y0, xc, yc);
            (self.draw_line)(x1, y1, xc, yc);
        } else {
            self.draw(x0, y0, xc, yc, c - 1);
            self.draw(x1, y1, xc, yc, c - 1);
        }
    }
    pub fn do_demo(&self) {
        self.draw(
            0.2777777777777778,
            0.25,
            0.5972222222222222,
            0.625,
            self.scale,
        );
    }
}
