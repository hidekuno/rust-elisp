(define frame (make-frame (make-vect 0 0) (make-vect 1 0) (make-vect 0 1)))
(define pi (*(atan 1)4))
(define (cs angle)(cos (/(* pi angle)180)))
(define (sn angle)(sin (/(* pi angle)180)))

;;========================================================================
;; 線を描画する
;;========================================================================
(define (draw-line-segment s f)
  (draw-line-vect
   ((frame-coord-map f) (start-segment s))
   ((frame-coord-map f) (end-segment s))))

;;========================================================================
;; ツリーカーブ(paint版)
;;========================================================================
(define (tree n)
  (lambda (frame)
    (define (tree-iter x0 y0 x1 y1 c)
      (let ((tcos (cs 15))
            (tsin (sn 45))
            (alpha 0.6))
        (let ((ya (+ y1  (*    tsin (- x1 x0) alpha) (*    tcos (- y1 y0) alpha)))
              (xa (+ x1  (*    tcos (- x1 x0) alpha) (* -1 tsin (- y1 y0) alpha)))
              (yb (+ y1  (* -1 tsin (- x1 x0) alpha) (*    tcos (- y1 y0) alpha)))
              (xb (+ x1  (*    tcos (- x1 x0) alpha) (*    tsin (- y1 y0) alpha))))
          (draw-line-segment (make-segment (make-vect x0 y0) (make-vect x1 y1)) frame)
          (if (>= 0 c)
              (begin
                (draw-line-segment (make-segment (make-vect x1 y1) (make-vect xa ya)) frame)
                (draw-line-segment (make-segment (make-vect x1 y1) (make-vect xb yb)) frame))
              (begin
                (tree-iter x1 y1 xa ya (- c 1))
                (tree-iter x1 y1 xb yb (- c 1)))))))
    (tree-iter 0.4166666666666667 0.7142857142857143 0.4166666666666667 0.5357142857142857 n)))
;;((square-limit (tree 14) 0) frame)

;;========================================================================
;; sierpinski(paint版)
;;========================================================================
(define (sierpinski n)
  (lambda (frame)
    (define (sierpinski-iter x0 y0 x1 y1 x2 y2 c)
      (if (> c 1) (let ((xx0 (/ (+ x0 x1) 2))
                        (yy0 (/ (+ y0 y1) 2))
                        (xx1 (/ (+ x1 x2) 2))
                        (yy1 (/ (+ y1 y2) 2))
                        (xx2 (/ (+ x2 x0) 2))
                        (yy2 (/ (+ y2 y0) 2)))
                     (sierpinski-iter x0 y0 xx0 yy0 xx2 yy2 (- c 1))
                     (sierpinski-iter x1 y1 xx0 yy0 xx1 yy1 (- c 1))
                     (sierpinski-iter x2 y2 xx2 yy2 xx1 yy1 (- c 1)))
          (begin
            (draw-line-segment (make-segment (make-vect x0 y0) (make-vect x1 y1)) frame)
            (draw-line-segment (make-segment (make-vect x1 y1) (make-vect x2 y2)) frame)
            (draw-line-segment (make-segment (make-vect x2 y2) (make-vect x0 y0)) frame))))
    (sierpinski-iter 0.44428969359331477 0.07168458781362007 0.04178272980501393 0.7706093189964157 0.8481894150417827 0.7706093189964157 n)))
;;((square-limit (sierpinski 8) 0) frame)
