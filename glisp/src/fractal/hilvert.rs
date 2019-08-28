/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use crate::draw::DrawLine;
use crate::fractal::Fractal;

struct Coord {
    oldx: f64,
    oldy: f64,
    x: f64,
    y: f64,
    lgth: f64,
}
impl Coord {
    fn new(c: i32) -> Self {
        let width = 1.0;
        let lgth = ((width / 2.0) as f64).powi(c);
        let y = (width - (lgth * (2.0 as f64).powi(c - 1))) / 3.6;
        let x = width - y;
        let oldx = x;
        let oldy = y;

        Coord {
            oldx: oldx,
            oldy: oldy,
            x: x,
            y: y,
            lgth: lgth,
        }
    }
}
pub struct Hilvert {
    draw_line: DrawLine,
}
impl Hilvert {
    pub fn new(draw_line: DrawLine) -> Self {
        Hilvert {
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
            coord.x -= coord.lgth;
            self.draw(coord);

            self.ldr(c - 1, coord);
            coord.y += coord.lgth;
            self.draw(coord);

            self.ldr(c - 1, coord);
            coord.x += coord.lgth;
            self.draw(coord);

            self.urd(c - 1, coord);
        }
    }
    fn urd(&self, c: i32, coord: &mut Coord) {
        if c == 0 {
            return;
        } else {
            self.rul(c - 1, coord);
            coord.y -= coord.lgth;
            self.draw(coord);

            self.urd(c - 1, coord);
            coord.x += coord.lgth;
            self.draw(coord);

            self.urd(c - 1, coord);
            coord.y += coord.lgth;
            self.draw(coord);

            self.ldr(c - 1, coord);
        }
    }
    fn rul(&self, c: i32, coord: &mut Coord) {
        if c == 0 {
            return;
        } else {
            self.urd(c - 1, coord);
            coord.x += coord.lgth;
            self.draw(coord);

            self.rul(c - 1, coord);
            coord.y -= coord.lgth;
            self.draw(coord);

            self.rul(c - 1, coord);
            coord.x -= coord.lgth;
            self.draw(coord);

            self.dlu(c - 1, coord);
        }
    }
    fn dlu(&self, c: i32, coord: &mut Coord) {
        if c == 0 {
            return;
        } else {
            self.ldr(c - 1, coord);
            coord.y += coord.lgth;
            self.draw(coord);

            self.dlu(c - 1, coord);
            coord.x -= coord.lgth;
            self.draw(coord);

            self.dlu(c - 1, coord);
            coord.y -= coord.lgth;
            self.draw(coord);

            self.rul(c - 1, coord);
        }
    }
}
impl Fractal for Hilvert {
    fn get_func_name(&self) -> &'static str {
        "draw-hilvert"
    }
    fn do_demo(&self, c: i32) {
        let mut coord = Coord::new(c);
        self.ldr(c, &mut coord);
    }
}
