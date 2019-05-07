;
; this is a sample program, and this is drawing koch
;
; hidekuno@gmail.com
;
(define koch 
  (lambda (x0 y0 x1 y1 c)
    (let ((kcos (cs 60))
          (ksin (sn 60)))
      (if (> c 1)
          (let (
                (xa (/ (+ (* x0 2) x1) 3))
                (ya (/ (+ (* y0 2) y1) 3))
                (xb (/ (+ (* x1 2) x0) 3))
                (yb (/ (+ (* y1 2) y0) 3)))
            (let ((yc (+ ya (+ (* (- xb xa) ksin) (* (- yb ya) kcos))))
                  (xc (+ xa (- (* (- xb xa) kcos) (* (- yb ya) ksin)))))
              (koch x0 y0 xa  ya (- c 1))
              (koch xa ya xc  yc (- c 1))
              (koch xc yc xb  yb (- c 1))
              (koch xb yb x1  y1 (- c 1))))
          (draw-line x0 y0 x1 y1)))))
(draw-clear)
(koch 259.0 0.0 34.0 390.0 4)
(koch 34.0 390.0 483.0 390.0 4)
(koch 483.0 390.0 259.0 0.0 4)
