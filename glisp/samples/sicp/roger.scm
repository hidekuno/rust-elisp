;
; this is a SICP program
; (https://sicp.iijlab.net/fulltext/x224.html)
;
; hidekuno@gmail.com
;

(define gframe (make-frame (make-vect 0.0 0.0)(make-vect 0.75 0.0)(make-vect 0.0 0.9642857142857143)))
(create-image-from-png "roger" "/home/kunohi/rust-elisp/glisp/samples/sicp/sicp.png")

(define roger
  (let ((image-name "roger"))
    (lambda (f)
      (draw-image image-name
                  (list 
                   (/ (xcor-vect (edge1-frame f)) 180.0)
                   (/ (ycor-vect (edge1-frame f)) 180.0)
                   (/ (xcor-vect (edge2-frame f)) 180.0)
                   (/ (ycor-vect (edge2-frame f)) 180.0)
                   (xcor-vect (origin-frame f))
                   (ycor-vect (origin-frame f)))))))
;;((square-limit roger 4) gframe)
