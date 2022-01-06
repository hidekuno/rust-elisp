(define (test::do-web-application req hdr)
  (define (hello) "Hello,")
  (define (world) "World")

  (filter (lambda (l) (string=? (car l) "FOO")) req)
  (string-append (hello)(world)))
