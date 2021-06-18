;
; this is a sample program
;
; change from number to kansuji.
; but rust-elisp's integer is 64 or 128bit.
;
; hidekuno@gmail.com
;
(define unit
  (list
        ""
        "万"
        "億"
        "兆"
        "京"
        "垓"
        "𥝱"
        "穣"
        "溝"
        "澗"
        "正"
        "載"
        "極"
        "恒河沙"
        "阿僧祇"
        "那由他"
        "不可思議"
        "無量大数"))

(define (make-unit item unit) (cons item unit))
(define (get-unit l)
  (let ((n (string->number (car l))))
    (if (= n 0) ""
        (string-append (format "~d" n)(cdr l)))))

(define (cddddr l)
  (let loop ((i 0)(m l))
    (if (or (null? m)(>= i 4)) m
        (loop (+ 1 i)(cdr m)))))

(define (make-answer answer)
  (let loop ((l answer)(s ""))
    (if (null? l) s
        (loop (cdr l)(string-append s (get-unit (car l)))))))

(define (make-kansuji answer l cnt)
  (let loop ((i 0)(m l)(buf (list)))
    (if (or (null? m)(>= i 4)) (cons (make-unit (list->string buf)
                                                (list-ref unit (quotient cnt 4))) answer)
        (loop (+ 1 i)(cdr m)(cons (car m) buf)))))

(define (to-kansuji num)
  (let loop ((l (reverse (string->list (number->string num))))
             (i 0)
             (answer (list)))
    (if (null? l) (make-answer answer)
        (loop (cddddr l)(+ i 4)(make-kansuji answer l i)))))
