;
; this is a sample program, and this is drawing tree curve
;
; hidekuno@gmail.com
;
(define (tree x0 y0 x1 y1 c)
  (let ((tcos (cs 15))
        (tsin (sn 45))
        (alpha 0.6))
    (let ((xa (+ x1  (*    tcos (- x1 x0) alpha) (* -1 tsin (- y1 y0) alpha)))
          (ya (+ y1  (*    tsin (- x1 x0) alpha) (*    tcos (- y1 y0) alpha)))
          (xb (+ x1  (*    tcos (- x1 x0) alpha) (*    tsin (- y1 y0) alpha)))
          (yb (+ y1  (* -1 tsin (- x1 x0) alpha) (*    tcos (- y1 y0) alpha))))
      (draw-line x0 y0 x1 y1)
      (if (>= 0 c)
          ((lambda () (draw-line x1 y1 xa ya) (draw-line x1 y1 xb yb)))
          ((lambda () (tree x1 y1 xa ya (- c 1))(tree x1  y1  xb  yb (- c 1))))))))

(draw-clear)
(tree 0.4166666666666667 0.7142857142857143 0.4166666666666667 0.5357142857142857 14)
