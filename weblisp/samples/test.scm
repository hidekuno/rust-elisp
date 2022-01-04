(define (test::do-web-application req)
  (define (hello) "Hello,")
  (define (world) "World")

  (string-append (hello)(world)))
