(define (test::do-web-application req hdr)
  (define (hello) "Hello,")
  (define (world) "World")

  (display (get-header "User-Agent" hdr))
  (newline)
  (string-append (hello)(world)))
