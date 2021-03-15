/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use elisp::draw::coord::Coord;
use elisp::draw::DrawLine;
use elisp::draw::Fractal;

pub struct Sierpinski {
    draw_line: DrawLine,
}
impl Sierpinski {
    pub fn new(draw_line: DrawLine) -> Self {
        Sierpinski {
            draw_line: draw_line,
        }
    }
    pub fn draw(&self, v0: Coord<f64>, v1: Coord<f64>, v2: Coord<f64>, c: i32) {
        if c > 1 {
            let vv0 = (v0 + v1) / 2.0;
            let vv1 = (v1 + v2) / 2.0;
            let vv2 = (v2 + v0) / 2.0;

            self.draw(v0, vv0, vv2, c - 1);
            self.draw(v1, vv0, vv1, c - 1);
            self.draw(v2, vv2, vv1, c - 1);
        } else {
            (self.draw_line)(v0.x, v0.y, v1.x, v1.y);
            (self.draw_line)(v1.x, v1.y, v2.x, v2.y);
            (self.draw_line)(v2.x, v2.y, v0.x, v0.y);
        }
    }
}
impl Fractal for Sierpinski {
    fn get_func_name(&self) -> &'static str {
        "draw-sierpinski"
    }
    fn do_demo(&self, c: i32) {
        self.draw(
            Coord::<f64>::new(310.0, 67.0),
            Coord::<f64>::new(60.0, 500.0),
            Coord::<f64>::new(560.0, 500.0),
            c,
        );
    }
}
