(define gframe (make-frame (make-vect 0 0) (make-vect 540 0) (make-vect 0 540)))
(define roger
  (let ((filename "/home/kunohi/rust-elisp/glisp/samples/sicp/sicp.png"))
    (lambda (f)
      (draw-image filename 
                  (list 
                   (/ (xcor-vect (edge1-frame frame)) 180)
                   (/ (ycor-vect (edge1-frame frame)) 180)
                   (/ (xcor-vect (edge2-frame frame)) 180)
                   (/ (ycor-vect (edge2-frame frame)) 180)
                   (xcor-vect (origin-frame frame))
                   (ycor-vect (origin-frame frame)))))))
;;((square-limit roger 1) gframe)
