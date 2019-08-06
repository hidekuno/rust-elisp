/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use crate::draw::DrawLine;

pub struct Sierpinski {
    scale: i64,
    draw_line: DrawLine,
}
impl Sierpinski {
    pub fn new(c: i64, draw_line: DrawLine) -> Sierpinski {
        Sierpinski {
            scale: c,
            draw_line: Box::new(draw_line),
        }
    }
    fn draw(&self, x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64, c: i64) {
        if c > 1 {
            let xx0 = (x0 + x1) / 2.0;
            let yy0 = (y0 + y1) / 2.0;
            let xx1 = (x1 + x2) / 2.0;
            let yy1 = (y1 + y2) / 2.0;
            let xx2 = (x2 + x0) / 2.0;
            let yy2 = (y2 + y0) / 2.0;

            self.draw(x0, y0, xx0, yy0, xx2, yy2, c - 1);
            self.draw(x1, y1, xx0, yy0, xx1, yy1, c - 1);
            self.draw(x2, y2, xx2, yy2, xx1, yy1, c - 1);
        } else {
            (self.draw_line)(x0, y0, x1, y1);
            (self.draw_line)(x1, y1, x2, y2);
            (self.draw_line)(x2, y2, x0, y0);
        }
    }
    pub fn do_demo(&self) {
        self.draw(
            0.44428969359331477,
            0.07168458781362007,
            0.04178272980501393,
            0.7706093189964157,
            0.8481894150417827,
            0.7706093189964157,
            self.scale,
        );
    }
}
