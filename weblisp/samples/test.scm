(define (test::main request sid)
  (define (make-data)
    (string-append "Hello,World" " "
                   (web-get-header "User-Agent" request)))

  (define (inc-value)
    (let ((s-value (web-get-session sid))
          (value (web-get-parameter "Value" request)))

      (if (null? s-value)(web-set-session sid (if (undefined? value) 0 (string->number value)))
          (web-set-session sid (+ 1 s-value))))
    (web-debug (web-get-session sid)))

  (inc-value)
  (web-create-response 200 "txt" (make-data)))
