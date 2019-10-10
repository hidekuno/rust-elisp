;
; this is a sample program
;
; ex.) ./target/release/lisp samples/queen.scm
;
; hidekuno@gmail.com
(define (conflict col row board)
  (let loop ((x board) (y (- col 1)))
    (cond ((null? x) #f)
          ((or 
            (= (+ col row) (+ y (car x)))
            (= (- col row) (- y (car x)))) #t)
          (else (loop  (cdr x)(- y 1))))))

(define (print-path path)
  (if (= 8 (length path))
      (begin
        (display path)
        (newline))))

(define (8queen xlist path)
  (let loop ((x xlist))
    (cond ((null? x)
           (print-path path))
          (else
           (cond ((not (conflict (length path) (car x) path))
                  (8queen (delete (car x) xlist)
                         (cons (car x) path))))
           (loop (cdr x))))))

(8queen (iota 8 1)(list))
