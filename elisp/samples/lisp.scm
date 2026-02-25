; ======================================================
; this is a tiny lisp interpreter program.
;
; hidekuno@gmail.com
; ======================================================
;
; how to run this program.
;
; 1. on unix shell
;   cd ${where}/rust-elisp/elisp
;   cargo run --bin lisp
;
; 2. on lisp
;   (load-file "samples/lisp.scm")
;   (repl)
;
(define (caddr l) (car (cdr (cdr l))))
(define (cadddr l) (car (cdr (cdr (cdr l)))))
(define (cdddr l) (cdr (cdr (cdr l))))

(define dbg #f)
(define (debug x) (if dbg (begin (display x)(newline))))

(define (lookup var env)
  (cond ((null? env) '())
        ((eq? var (caar env))
         (cdar env))
        (else
         (lookup var (cdr env)))))

(define (extend-env var val env)
  (cons (cons var val) env))

(define (extend-env* vars vals env)
  (if (null? vars)
      env
      (extend-env* (cdr vars)
                   (cdr vals)
                   (extend-env (car vars) (car vals) env))))

(define (my-eval expr env)
  (cond
   ((number? expr) expr)
   ((string? expr) expr)
   ((char? expr) expr)
   ((boolean? expr) expr)
   ((symbol? expr)
    (lookup expr env))

   ((list? expr)
    (cond
     ((eq? (car expr) 'quote) (cadr expr))

     ((eq? (car expr) 'if)
      (let ((test (my-eval (cadr expr) env)))
        (if (eq? test #f)
            (if (null? (cdddr expr))
                #f
                (my-eval (cadddr expr) env))
            (my-eval (caddr expr) env))))

     ((eq? (car expr) 'define)
      (if (and (list? (caddr expr)) (eq? (car (caddr expr)) 'lambda))
          (let ((l (caddr expr)))
            (set! global-env (cons (cons (cadr expr) l) env))
            (cadr expr))
          (let ((val (my-eval (caddr expr) global-env)))
            (set! global-env
                  (extend-env (cadr expr) val env))
            val)))

     ((eq? (car expr) 'lambda)
      (list 'closure
            (cadr expr)
            (caddr expr)
            env))
     (else
      (let ((func (lookup (car expr) env)))
        (my-apply
         (if (null? func) (car expr) func)
         (map (lambda (arg)
                (my-eval arg env))
              (cdr expr)))))))
   (else
    expr)))

(define (my-apply proc args)
  (debug "================================")
  (debug proc)
  (debug args)
  (debug "================================")
  (cond
   ((procedure? proc)
    (apply proc args))

   ((symbol? proc)
    (apply (my-eval proc global-env) args))

   ((list? proc)
    (let ((params (cadr proc))
          (body   (caddr proc))
          (env    (if (eq? (car proc) 'closure) (cadddr proc) global-env)))
          (my-eval body
                   (extend-env* params args env))))
   (else
    (error "Unknown procedure type" proc))))

(define global-env
  (list
   (cons '+  +)
   (cons '-  -)
   (cons '*  *)
   (cons '/  /)
   (cons '<  <)
   (cons '>  >)
   (cons '<= <=)
   (cons '>= >=)
   (cons '=  =)
   (cons 'modulo modulo)))

(define (repl)
  (display "Tiny-Lisp> ")
  (let ((input (read)))
    (if (eq? input 'quit)
        (display "Bye")
        (begin
          (let ((result (my-eval input global-env)))
            (display result)
            (newline))
          (repl)))))
