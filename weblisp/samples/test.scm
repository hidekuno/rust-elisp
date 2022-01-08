(define (test::do-web-application request)
  (define (get-user-agnet request)
    (web-get-header "User-Agent" request))

  (display (web-get-resource request))
  (string-append "Hello,World" " " (get-user-agnet request)))
