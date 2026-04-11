;;
;; Test programme when executed with the following command line
;;
;; cargo run --bin lisp -- --compile
;;
(define (map callable l)
  (define (map-iter l ml)
    (if (null? l) ml
        (map-iter (cdr l)(cons (callable (car l)) ml))))
  (map-iter (reverse l) '()))

(define (filter callable l)
  (if (null? l) '()
      (if (callable (car l)) (cons (car l) (filter callable (cdr l)))
          (filter callable (cdr l)))))

(define (reduce callable init l)
  (let loop ((x l)(r init))
    (if (null? x) r
        (loop (cdr x)(callable r (car x))))))

(define (gcm n m) (let ((mod (modulo n m))) (if (= 0 mod) m (gcm m mod))))
(gcm 36 27)

(define (lcm n m)(quotient (* n m)(gcm n m)))
(lcm 12 18)

(define (hanoi from to work n)
  (if (= 0 n) '()
      (append
       (hanoi from work to (- n 1))
       (list (cons from to) n)
       (hanoi work to from (- n 1)))))
(hanoi 'a 'b 'c 3)

(define (prime l)
  (if (>= (car l)(sqrt (last l))) l
      (cons (car l)(prime (filter (lambda (n) (not (= 0 (modulo n (car l))))) (cdr l))))))
(prime (iota 30 2))

(define (perm l n)
  (if (>= 0 n) (list '())
      (reduce (lambda (a b) (append a b)) '()
              (map (lambda (x) (map (lambda (p) (cons x p)) (perm (delete x l)(- n 1)))) l))))
(perm '(1 2 3) 2)

(define (comb l n)
  (if (null? l) l
      (if (= n 1)(map list l)
          (append
           (map (lambda (x) (cons (car l) x)) (comb (cdr l)(- n 1)))
           (comb (cdr l) n)))))
(comb '(1 2 3) 2)

(define xlist '(72 31 79 95 50 17 2 35 55 8 75 70 57 82 74 1 17 77 69 85 46 47 49 97 63 98 46 74 67 63 62 33 40 72 94 91 22 48 33 66))

(define (bubble-iter x l)
  (if (or (null? l)(< x (car l))) (cons x l)
      (cons (car l)(bubble-iter x (cdr l)))))
(define (bsort l)
  (if (null? l) l
      (bubble-iter (car l)(bsort (cdr l)))))
(bsort xlist)

(define (qsort l)
  (if (null? l) l
      (append
       (qsort (filter (lambda (n) (< n (car l))) (cdr l)))
       (cons (car l)(qsort (filter (lambda (n) (>= n (car l))) (cdr l)))))))
(qsort xlist)

(define (merge a b)
  (if (or (null? a)(null? b))(append a b)
      (if (< (car a)(car b))
          (cons (car a)(merge (cdr a) b))
          (cons (car b)(merge a (cdr b))))))

(define (msort l)
  (let ((len (length l)))
    (cond ((>= 1 len) l)
          ((= 2 len) (if (<(car l)(cadr l)) l (reverse l)))
          (else
           (let ((mid (quotient len 2)))
             (merge (msort (take l mid))(msort (drop l mid))))))))
(msort xlist)
