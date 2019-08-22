;
; this is a SICP program
; (https://sicp.iijlab.net/fulltext/x224.html)
;
; hidekuno@gmail.com
;
(define frame (make-frame (make-vect 0 0) (make-vect 1 0) (make-vect 0 1)))

(define outline
  (let ((v0 (make-vect 0.0 0.0))
        (v1 (make-vect 1.0 0.0))
        (v2 (make-vect 0.0 1.0))
        (v3 (make-vect 1.0 1.0)))
    (segments->painter (list (make-segment v0 v1)
                             (make-segment v1 v3)
                             (make-segment v3 v2)
                             (make-segment v2 v0)))))
(define x11
  (let ((v1 (make-vect 0.0 0.0))
        (v2 (make-vect 1.0 0.0))
        (v3 (make-vect 0.0 1.0))
        (v4 (make-vect 1.0 1.0)))
    (segments->painter (list (make-segment v1 v4)
                             (make-segment v2 v3)))))

(define diamond
  (let ((v1 (make-vect 0.5 0.0))
        (v2 (make-vect 0.0 0.5))
        (v3 (make-vect 1.0 0.5))
        (v4 (make-vect 0.5 1.0)))
    (segments->painter (list (make-segment v1 v3)
                             (make-segment v3 v4)
                             (make-segment v4 v2)
                             (make-segment v2 v1)))))
(define wave
  (let ((segments (list
                   (make-segment(make-vect 0.35 0.15) (make-vect 0.4 0))
                   (make-segment(make-vect 0.65 0.15) (make-vect 0.6 0))
                   (make-segment(make-vect 0.35 0.15) (make-vect 0.4 0.35))
                   (make-segment(make-vect 0.65 0.15) (make-vect 0.6 0.35))
                   (make-segment(make-vect 0.6 0.35)  (make-vect 0.75 0.35))
                   (make-segment(make-vect 0.4 0.35)  (make-vect 0.3 0.35))
                   (make-segment(make-vect 0.75 0.35) (make-vect 1 0.65))
                   (make-segment(make-vect 0.6 0.55)  (make-vect 1 0.85))
                   (make-segment(make-vect 0.6 0.55)  (make-vect 0.75 1))
                   (make-segment(make-vect 0.5 0.7)   (make-vect 0.6 1))
                   (make-segment(make-vect 0.3 0.35)  (make-vect 0.15 0.4))
                   (make-segment(make-vect 0.3 0.4)   (make-vect 0.15 0.6))
                   (make-segment(make-vect 0.15 0.4)  (make-vect 0 0.15))
                   (make-segment(make-vect 0.15 0.6)  (make-vect 0 0.35))
                   (make-segment(make-vect 0.3 0.4)   (make-vect 0.35 0.5))
                   (make-segment(make-vect 0.35 0.5)  (make-vect 0.25 1))
                   (make-segment(make-vect 0.5 0.7)   (make-vect 0.4 1))
                   )))
    (segments->painter segments)))
;;((square-limit wave 4) frame)
