(define (test::do-web-application request)
  (define (make-data request)
    (string-append "Hello,World" " "
                   (web-get-header "User-Agent" request)))

  (web-create-response 200 "txt" (make-data request)))
