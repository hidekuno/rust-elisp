/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use crate::draw::DrawLine;

pub struct Tree {
    cs: f64,
    sn: f64,
    scale: i64,
    draw_line: DrawLine,
}
impl Tree {
    pub fn new(c: i64, draw_line: DrawLine) -> Tree {
        Tree {
            cs: ((std::f64::consts::PI * 15.0) / 180.0).cos(),
            sn: ((std::f64::consts::PI * 45.0) / 180.0).sin(),
            scale: c,
            draw_line: Box::new(draw_line),
        }
    }
    pub fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, c: i64) {
        let alpha = 0.6;

        (self.draw_line)(x0, y0, x1, y1);

        let ya = y1 + self.sn * (x1 - x0) * alpha + self.cs * (y1 - y0) * alpha;
        let xa = x1 + self.cs * (x1 - x0) * alpha - self.sn * (y1 - y0) * alpha;

        let yb = y1 + (-self.sn * (x1 - x0)) * alpha + self.cs * (y1 - y0) * alpha;
        let xb = x1 + self.cs * (x1 - x0) * alpha + self.sn * (y1 - y0) * alpha;

        if 0 >= c {
            (self.draw_line)(x1, y1, xa, ya);
            (self.draw_line)(x1, y1, xb, yb);
        } else {
            self.draw(x1, y1, xa, ya, c - 1);
            self.draw(x1, y1, xb, yb, c - 1);
        }
    }
    pub fn do_demo(&self) {
        self.draw(
            0.4166666666666667,
            0.7142857142857143,
            0.4166666666666667,
            0.5357142857142857,
            self.scale,
        );
    }
}
