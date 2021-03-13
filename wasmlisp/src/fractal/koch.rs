/*
  Rust study program.
  This is prototype program mini scheme subset what porting from go-scheme.

  hidekuno@gmail.com
*/
use crate::draw::DrawLine;
use crate::fractal::Fractal;
use elisp::draw::coord::Coord;
use std::f64::consts::PI;

pub struct Koch {
    matrix: (Coord<f64>, Coord<f64>),
    draw_line: DrawLine,
}
impl Koch {
    pub fn new(draw_line: DrawLine) -> Self {
        let sn = ((PI * 60.0) / 180.0).sin();
        let cs = ((PI * 60.0) / 180.0).cos();

        Koch {
            matrix: (Coord::<f64>::new(cs, -sn), Coord::<f64>::new(sn, cs)),
            draw_line: draw_line,
        }
    }
    pub fn draw(&self, v0: Coord<f64>, v1: Coord<f64>, c: i32) {
        if c > 1 {
            let va = (v0.scale(2 as f64) + v1).scale(1.0 / 3.0);
            let vb = (v1.scale(2 as f64) + v0).scale(1.0 / 3.0);
            let vc = va
                + Coord::<f64>::new(
                    (self.matrix.0 * (vb - va)).sum(),
                    (self.matrix.1 * (vb - va)).sum(),
                );

            self.draw(v0, va, c - 1);
            self.draw(va, vc, c - 1);
            self.draw(vc, vb, c - 1);
            self.draw(vb, v1, c - 1);
        } else {
            (self.draw_line)(v0.x, v0.y, v1.x, v1.y);
        }
    }
}
impl Fractal for Koch {
    fn get_func_name(&self) -> &'static str {
        "draw-koch"
    }
    fn do_demo(&self, c: i32) {
        self.draw(
            Coord::<f64>::new(260.0, 0.0),
            Coord::<f64>::new(34.0, 390.0),
            c,
        );
        self.draw(
            Coord::<f64>::new(34.0, 390.0),
            Coord::<f64>::new(483.0, 390.0),
            c,
        );
        self.draw(
            Coord::<f64>::new(483.0, 390.0),
            Coord::<f64>::new(260.0, 0.0),
            c,
        );
    }
}
