;
; this is a fractal program for picture language
; (But, Consumes Huge memory)
;
; hidekuno@gmail.com
;

;;========================================================================
;; sierpinski(paint版)
;;========================================================================
(define (sierpinski-danger-iter x0 y0 x1 y1 x2 y2 c)
  (if (> c 1) (let ((xx0 (/ (+ x0 x1) 2))
                    (yy0 (/ (+ y0 y1) 2))
                    (xx1 (/ (+ x1 x2) 2))
                    (yy1 (/ (+ y1 y2) 2))
                    (xx2 (/ (+ x2 x0) 2))
                    (yy2 (/ (+ y2 y0) 2)))
                (append
                 (sierpinski-danger-iter x0 y0 xx0 yy0 xx2 yy2 (- c 1))
                 (sierpinski-danger-iter x1 y1 xx0 yy0 xx1 yy1 (- c 1))
                 (sierpinski-danger-iter x2 y2 xx2 yy2 xx1 yy1 (- c 1))))
      (let ((hoge 0))
        (append
         (list (make-segment (make-vect x0 y0) (make-vect x1 y1)))
         (list (make-segment (make-vect x1 y1) (make-vect x2 y2)))
         (list (make-segment (make-vect x2 y2) (make-vect x0 y0)))))))

(define sierpinski-danger (segments->painter (sierpinski-danger-iter 0.44428969359331477 0.07168458781362007 0.04178272980501393 0.7706093189964157 0.8481894150417827 0.7706093189964157 8)))
;;((square-limit sierpinski-danger 0) frame)

;;========================================================================
;; ツリーカーブ(paint版)
;;========================================================================
(define (tree-danger-iter x0 y0 x1 y1 c)
  (let ((tcos (cs 15))
        (tsin (sn 45))
        (alpha 0.6))
    (let ((xa (+ x1  (*    tcos (- x1 x0) alpha) (* -1 tsin (- y1 y0) alpha)))
          (ya (+ y1  (*    tsin (- x1 x0) alpha) (*    tcos (- y1 y0) alpha)))
          (xb (+ x1  (*    tcos (- x1 x0) alpha) (*    tsin (- y1 y0) alpha)))
          (yb (+ y1  (* -1 tsin (- x1 x0) alpha) (*    tcos (- y1 y0) alpha))))
      (append (list (make-segment (make-vect x0 y0) (make-vect x1 y1)))
              (if (>= 0 c)
                  (append
                   (list (make-segment (make-vect x1 y1) (make-vect xa ya)))
                   (list (make-segment (make-vect x1 y1) (make-vect xb yb))))
                  (append (tree-danger-iter x1 y1 xa ya (- c 1))(tree-danger-iter x1  y1  xb  yb (- c 1))))))))

(define tree-danger (segments->painter (tree-danger-iter 0.4166666666666667 0.7142857142857143 0.4166666666666667 0.5357142857142857 14)))
;;((square-limit tree-danger 0) frame)
