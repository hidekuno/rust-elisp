/*
  Rust study program.
  This is prototype program mini scheme subset what porting from go-scheme.

  hidekuno@gmail.com
*/
use crate::draw::DrawLine;
use crate::fractal::Fractal;
use elisp::draw::coord::Coord;
use std::f64::consts::PI;

pub struct Tree {
    left: (Coord<f64>, Coord<f64>),
    right: (Coord<f64>, Coord<f64>),
    draw_line: DrawLine,
}
impl Tree {
    pub fn new(draw_line: DrawLine) -> Self {
        let cs = ((PI * 15.0) / 180.0).cos();
        let sn = ((PI * 45.0) / 180.0).sin();

        Tree {
            left: (Coord::<f64>::new(cs, -sn), Coord::<f64>::new(sn, cs)),
            right: (Coord::<f64>::new(cs, sn), Coord::<f64>::new(-sn, cs)),
            draw_line: draw_line,
        }
    }
    pub fn draw(&self, v0: Coord<f64>, v1: Coord<f64>, c: i32) {
        let alpha = 0.6;

        (self.draw_line)(v0.x, v0.y, v1.x, v1.y);

        let s = (v1 - v0).scale(alpha);
        let va = v1 + Coord::<f64>::new((self.left.0 * s).sum(), (self.left.1 * s).sum());
        let vb = v1 + Coord::<f64>::new((self.right.0 * s).sum(), (self.right.1 * s).sum());

        if 0 >= c {
            (self.draw_line)(v1.x, v1.y, va.x, va.y);
            (self.draw_line)(v1.x, v1.y, vb.x, vb.y);
        } else {
            self.draw(v1, va, c - 1);
            self.draw(v1, vb, c - 1);
        }
    }
}
impl Fractal for Tree {
    fn get_func_name(&self) -> &'static str {
        "draw-tree"
    }
    fn do_demo(&self, c: i32) {
        self.draw(
            Coord::<f64>::new(300.0, 400.0),
            Coord::<f64>::new(300.0, 300.0),
            c,
        );
    }
}
