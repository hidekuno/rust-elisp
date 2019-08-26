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
(define (koch-demo)
  (begin
    (draw-clear)
    (let ((c 8))
      (koch 0.3597222222222222 0.0 0.04722222222222222 0.6964285714285714 c)
      (koch 0.04722222222222222 0.6964285714285714 0.6708333333333333 0.6964285714285714 c)
      (koch 0.6708333333333333 0.6964285714285714 0.3597222222222222 0.0 c))))
