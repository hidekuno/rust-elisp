;
; this is a sample program, and this is drawing sierpinski
;
; hidekuno@gmail.com
;

(define (sierpinski x0 y0 x1 y1 x2 y2 c)
  (if (> c 1) (let ((xx0 (/ (+ x0 x1) 2))
                    (yy0 (/ (+ y0 y1) 2))
                    (xx1 (/ (+ x1 x2) 2))
                    (yy1 (/ (+ y1 y2) 2))
                    (xx2 (/ (+ x2 x0) 2))
                    (yy2 (/ (+ y2 y0) 2)))
                (sierpinski x0 y0 xx0 yy0 xx2 yy2 (- c 1))
                (sierpinski x1 y1 xx0 yy0 xx1 yy1 (- c 1))
                (sierpinski x2 y2 xx2 yy2 xx1 yy1 (- c 1)))
      (let ((hoge 0))
        (draw-line x0 y0 x1 y1)
        (draw-line x1 y1 x2 y2)
        (draw-line x2 y2 x0 y0))))
(draw-clear)
(sierpinski 319.0 40.0 30.0 430.0 609.0 430.0 8)
