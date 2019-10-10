; ======================================================
; this is a sample program
;
; hidekuno@gmail.com
; ======================================================
; how to encode
;　3byte read
; ex.) 00001000　11101110　11101110

;　6bit
; ex.) 000010　001110　111011　101110

;　hit base64 table("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
; ex.）   C      O       7       u

; last byte　
;ex.) 00001000　11101110
;　　000010 001110 111000
;　　　 C      0       2       =

(define X0f 15)
(define X3f 63)
(define X03 3)
(define (base64-encode word)
  (define (make-key-table)
    (string->list 
     "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"))

  (define (get-param count num-str)
    (let ((num (char->integer num-str)))
      (case (modulo count 3)
        ((1) (ash (logand num 3) 4))
        ((2) (ash (logand num X0f) 2))
        ((0) -1))))

  (define (get-result count num-str param answer)
    (let ((key (make-key-table))(num (char->integer num-str)))
      (cond ((= 0 (modulo count 3))
             (cons (list-ref key (logand num X3f))
                   (cons (list-ref key (+ param (ash num -6))) answer)))
            (else
             (cons (list-ref 
                    key
                    (case (modulo count 3)
                      ((1) (ash num -2))
                      ((2) (+ param (ash num -4))))) answer)))))

  (define (get-result-last count param answer)
    (let ((key (make-key-table)))
      (cond ((= 1 (modulo count 3)) answer)
            (else
             (case (modulo count 3)
               ((0)
                (cons #\= (cons (list-ref key (+ param (ash 0 -6))) answer)))
               ((2)
                (cons #\= (cons #\= 
                                (cons (list-ref key (+ param (ash 0 -4)))
                                      answer)))))))))
  (let loop ((i 1)
             (param -1)
             (wlist (string->list word))
             (answer '()))
    (cond ((null? wlist) 
           (list->string (reverse (get-result-last i param answer))))
          (else
           (loop (+ i 1)
                 (get-param i (car wlist))
                 (cdr wlist)
                 (get-result i (car wlist) param answer))))))

;1byte 2bit <
;2byte 4bit >
;2byte 4bit <
;3byte 2bit >
;3byte 6bit <
(define (base64-decode word)
  (define (make-key-table)
     (string->list 
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/="))
  (define (get-idx chr)
    (let ((key (make-key-table)))
      (let loop ((i 0))
        (cond ((char=? (list-ref key i)  chr)
               i)
              (else (loop (+ i 1))))))) 
  (define (get-param count num)
    (case (modulo count 4)
      ((1) (ash num 2))
      ((2) (ash (logand num X0f) 4))
      ((3) (ash (logand X03 num) 6))
      ((0) num)))
  (define (get-answer count num param answer)
    (if (= 1 (modulo count 4)) answer
        (cons 
         (integer->char
          (case (modulo count 4)
            ((2) (+ param (ash num -4)))
            ((3) (+ param (ash num -2) ))
            ((0) (+ param num))))
         answer)))
    (let loop ((i 1)(wlist (string->list word))(param #f)(answer '()))
      (cond ((or (null? wlist) (= 64 (get-idx (car wlist))))
             (list->string (reverse answer)))
            (else
             (loop (+ i 1)
                   (cdr wlist)
                   (get-param  i (get-idx (car wlist)))
                   (get-answer i (get-idx (car wlist)) param answer))))))
