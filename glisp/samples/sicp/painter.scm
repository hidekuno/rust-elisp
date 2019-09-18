;
; this is a SICP program(https://sicp.iijlab.net/fulltext/x224.html)
; (https://sicp.iijlab.net/fulltext/x224.html)
;
; hidekuno@gmail.com
;

;;======================================================================
;; ペインタは規定した平行四辺形の フレームの中に合うように,
;; ずらしたり大きさを変えたりした画像を描く
;;======================================================================
(define (segments->painter segment-list)
  (lambda (frame)
    (for-each
     (lambda (segment)
       (draw-line-vect
        ((frame-coord-map frame) (start-segment segment))
        ((frame-coord-map frame) (end-segment segment))))
     segment-list)))

;;======================================================================
;; ペインタとフレームの変換法の情報をとり, 新しいペインタを作る
;;======================================================================
(define (transform-painter painter origin corner1 corner2)
  (lambda (frame)
    (let ((m (frame-coord-map frame)))
      (let ((new-origin (m origin)))
        (painter
         (make-frame new-origin
                     (sub-vect (m corner1) new-origin)
                     (sub-vect (m corner2) new-origin)))))))
;;======================================================================
;; 以下その応用
;;======================================================================
(define (flip-vert painter)
  (transform-painter painter
                     (make-vect 0.0 1.0)
                     (make-vect 1.0 1.0)
                     (make-vect 0.0 0.0)))

(define (shrink-to-upper-right painter)
  (transform-painter painter
                     (make-vect 0.5 0.5)
                     (make-vect 1.0 0.5)
                     (make-vect 0.5 0.0)))

(define (rotate90 painter)
  (transform-painter painter
                     (make-vect 1.0 0.0)
                     (make-vect 1.0 1.0)
                     (make-vect 0.0 0.0)))

(define (squash-inwards painter)
  (transform-painter painter
                     (make-vect 0.0 0.0)
                     (make-vect 0.65 0.35)
                     (make-vect 0.35 0.65)))


(define (beside painter1 painter2)
  (let ((split-point (make-vect 0.5 0.0)))
    (let ((paint-left
           (transform-painter painter1
                              (make-vect 0.0 0.0)
                              split-point
                              (make-vect 0.0 1.0)))
          (paint-right
           (transform-painter painter2
                              split-point
                              (make-vect 1.0 0.0)
                              (make-vect 0.5 1.0))))
  (lambda (frame)
    (paint-left frame)
    (paint-right frame)))))

(define (corner-split painter n)
  (if (= n 0)
      painter
      (let ((up (up-split painter(- n 1)))
            (right (right-split painter (- n 1))))
        (let ((top-left (beside up up))
              (bottom-right (below right right))
              (corner (corner-split painter (- n 1))))
          (beside (below painter top-left)
                  (below bottom-right corner))))))

(define (square-limit painter n)
  (let ((quarter (corner-split painter n)))
        (let ((half (beside (flip-horiz quarter) quarter)))
          (below (flip-vert half) half))))

(define (up-split painter n)
  (if (= n 0)
      painter
      (let ((smaller (up-split painter (- n 1))))
        (below painter (beside smaller smaller)))))

(define (rotate270 painter)
  (transform-painter painter
                     (make-vect 0.0 1.0)
                     (make-vect 0.0 0.0)
                     (make-vect 1.0 1.0)))


(define (below painter1 painter2)
  (rotate90 (beside (rotate270 painter1) (rotate270 painter2))))

(define (flip-horiz painter)
  (transform-painter painter
                     (make-vect 1.0 0.0)
                     (make-vect 0.0 0.0)
                     (make-vect 1.0 1.0)))

(define (right-split painter n)
  (if (= n 0)
      painter
      (let ((smaller (right-split painter (- n 1))))
        (beside painter (below smaller smaller)))))

;;======================================================================
;; 汎用部品
;;======================================================================
(define pi (*(atan 1)4))
(define (cs angle)(cos (/(* pi angle)180)))
(define (sn angle)(sin (/(* pi angle)180)))

(define draw-line-vect (lambda (p1 p2) (draw-line (xcor-vect p1)(ycor-vect p1)(xcor-vect p2)(ycor-vect p2))))

(define (paint-image image-name)
    (lambda (f)
      (draw-image image-name
                  (xcor-vect (origin-frame f))
                  (ycor-vect (origin-frame f))
                  (xcor-vect (edge1-frame f))
                  (ycor-vect (edge1-frame f))
                  (xcor-vect (edge2-frame f))
                  (ycor-vect (edge2-frame f)))))

(define (make-image-frame img scale)
  (make-frame (make-vect 0.0 0.0)
              (make-vect (/ (* scale (image-width img)) 720.0) 0.0)
              (make-vect 0.0 (/ (* scale (image-height img)) 560.0))))
