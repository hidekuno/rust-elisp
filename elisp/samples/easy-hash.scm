;
; this is a sample program
;
; hidekuno@gmail.com

(define tekito-prime 1021)
(define (easy-hash s)
  (let loop ((x (map (lambda (n) (char->integer n))(string->list s)))
             (h 0)
             (u 0))
    (cond ((null? x) (modulo (+ u h) tekito-prime))
          (else
           (loop (cdr x)
                 (modulo (+ (car x) (* h (string-length s))) tekito-prime)
                 (- 8 (modulo (+ (car x) (* u (string-length s))) 8)))))))
