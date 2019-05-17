(define gframe (make-frame (make-vect 0.0 0.0) (make-vect 540.0 0.0) (make-vect 0.0 540.0)))
(define roger
  (let ((filename "/home/kunohi/rust-elisp/glisp/samples/sicp/sicp.png"))
    (lambda (f)
      (draw-image filename 
                  (list 
                   (/ (xcor-vect (edge1-frame f)) 180.0)
                   (/ (ycor-vect (edge1-frame f)) 180.0)
                   (/ (xcor-vect (edge2-frame f)) 180.0)
                   (/ (ycor-vect (edge2-frame f)) 180.0)
                   (xcor-vect (origin-frame f))
                   (ycor-vect (origin-frame f)))))))
;;((square-limit roger 1) gframe)
