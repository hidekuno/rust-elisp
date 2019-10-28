;
; this is a sample program
;
; hidekuno@gmail.com

;; Zeller's congruence
(define (get-day-of-week year month day)
  (define (make-ym year month)
    (if (or (= month 1)(= month 2))
        (cons (- year 1) (+ month 1))
        (cons year month)))
  (define (get-year ym)(car ym))
  (define (get-month ym)(cdr ym))
  (let ((ym (make-ym year month)))
    (modulo 
     (+ year
        (quotient (get-year ym) 4)
        (quotient (get-year ym) -100)
        (quotient (get-year ym) 400)
        (quotient (+ (* (get-month ym) 26) 16) 10) day) 7)))
