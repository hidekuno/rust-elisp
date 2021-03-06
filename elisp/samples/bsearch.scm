;
; this is a sample program
;
; hidekuno@gmail.com

(define (bsearch buf target)
  (let loop ((mid (quotient (- (length buf) 1) 2))
             (first 0)
             (end (- (length buf) 1)))
    (cond ((< end first) #f)
          ((= (list-ref buf mid) target) mid)
          ((< (list-ref buf mid) target) 
           (loop (quotient (+ (+ mid 1) end) 2) (+ mid 1) end))
          ((> (list-ref buf mid) target)
           (loop (quotient (+ (- mid 1) end) 2) first (- mid 1))))))
