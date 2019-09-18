;
; this is a demo program for picture language
;
; hidekuno@gmail.com
;
(define (demo)
  (let ((dframe (make-frame (make-vect 0.45  0.55)(make-vect 0.45  0)(make-vect 0  0.45)))
        (frame (make-frame (make-vect 0  0.57)(make-vect 0.45  0)(make-vect 0 0.45)))
        (sframe (make-frame (make-vect 0 0)(make-vect 0.45 0)(make-vect 0 0.5714285714285714)))
        (gframe (make-frame (make-vect 0.472  0) (make-vect 0.375  0) (make-vect 0 0.48214285714285715))))
    ((square-limit wave 4) sframe)
    ((square-limit (paint-image roger) 4) gframe)
    ((square-limit (sierpinski 6) 0) frame)
    ((square-limit (tree 10) 0) dframe)))
