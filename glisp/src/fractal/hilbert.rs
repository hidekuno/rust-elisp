/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use crate::draw::DrawLine;
use crate::fractal::FractalMut;

struct Coord {
    oldx: f64,
    oldy: f64,
    x: f64,
    y: f64,
}
impl Coord {
    fn new(oldx: f64, oldy: f64, x: f64, y: f64) -> Self {
        Coord {
            oldx: oldx,
            oldy: oldy,
            x: x,
            y: y,
        }
    }
}
pub struct Hilbert {
    lgth: f64,
    draw_line: DrawLine,
}
impl Hilbert {
    pub fn new(draw_line: DrawLine) -> Self {
        Hilbert {
            lgth: 0.0,
            draw_line: draw_line,
        }
    }
    fn draw(&self, coord: &mut Coord) {
        (self.draw_line)(coord.oldx, coord.oldy, coord.x, coord.y);
        coord.oldx = coord.x;
        coord.oldy = coord.y;
    }
    fn ldr(&self, c: i32, coord: &mut Coord) {
        if c == 0 {
            return;
        } else {
            self.dlu(c - 1, coord);
            coord.x -= self.lgth;
            self.draw(coord);

            self.ldr(c - 1, coord);
            coord.y += self.lgth;
            self.draw(coord);

            self.ldr(c - 1, coord);
            coord.x += self.lgth;
            self.draw(coord);

            self.urd(c - 1, coord);
        }
    }
    fn urd(&self, c: i32, coord: &mut Coord) {
        if c == 0 {
            return;
        } else {
            self.rul(c - 1, coord);
            coord.y -= self.lgth;
            self.draw(coord);

            self.urd(c - 1, coord);
            coord.x += self.lgth;
            self.draw(coord);

            self.urd(c - 1, coord);
            coord.y += self.lgth;
            self.draw(coord);

            self.ldr(c - 1, coord);
        }
    }
    fn rul(&self, c: i32, coord: &mut Coord) {
        if c == 0 {
            return;
        } else {
            self.urd(c - 1, coord);
            coord.x += self.lgth;
            self.draw(coord);

            self.rul(c - 1, coord);
            coord.y -= self.lgth;
            self.draw(coord);

            self.rul(c - 1, coord);
            coord.x -= self.lgth;
            self.draw(coord);

            self.dlu(c - 1, coord);
        }
    }
    fn dlu(&self, c: i32, coord: &mut Coord) {
        if c == 0 {
            return;
        } else {
            self.ldr(c - 1, coord);
            coord.y += self.lgth;
            self.draw(coord);

            self.dlu(c - 1, coord);
            coord.x -= self.lgth;
            self.draw(coord);

            self.dlu(c - 1, coord);
            coord.y -= self.lgth;
            self.draw(coord);

            self.rul(c - 1, coord);
        }
    }
}
impl FractalMut for Hilbert {
    fn get_func_name(&self) -> &'static str {
        "draw-hilbert"
    }
    fn do_demo(&mut self, c: i32) {
        let width = 1.0;
        self.lgth = ((width / 2.0) as f64).powi(c);
        let y = (width - (self.lgth * (2.0 as f64).powi(c - 1))) / 3.6;
        let x = width - y;
        let oldx = x;
        let oldy = y;

        let mut coord = Coord::new(oldx, oldy, x, y);
        self.ldr(c, &mut coord);
    }
}
