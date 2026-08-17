#lang racket
;;;
;;; interpreter.rkt — повноцінний інтерпретатор my-lisp поверх Racket.
;;;
;;; Це не мапінг my-lisp на Racket-семантику, а власний evaluator,
;;; який працює зі значеннями my-lisp і може завантажувати справжню
;;; bootstrap-бібліотеку lib/core.my (та інші *.my файли).
;;;

(require "reader-lib.rkt")
(require racket/runtime-path)

;; -----------------------------------------------------------------
;; Значення my-lisp
;; -----------------------------------------------------------------

;; t — друкується як "t" і є істиною.
(struct my-true () #:transparent
  #:methods gen:custom-write
  [(define (write-proc v port mode)
     (write-string "t" port))])

;; Замикання: (lambda params body...) або макро-трансформер.
(struct my-closure (params variadic? body env) #:transparent)

;; Макрос — це замикання, яке викликається на невиражених аргументах.
(struct my-macro (closure) #:transparent)

;; Примітив — Racket-процедура, обгортка для вбудованих функцій.
(struct my-primitive (name proc) #:transparent)

(define t (my-true))
(define nil '())

;; -----------------------------------------------------------------
;; Середовище (chain of frames)
;; -----------------------------------------------------------------

(struct env (frame parent) #:transparent)

(define (make-env [parent #f])
  (env (make-hash) parent))

(define (env-lookup e name)
  (cond
    [(not e) (error 'my-lisp "unbound identifier: ~a" name)]
    [(hash-ref (env-frame e) name (lambda () #f)) => values]
    [else (env-lookup (env-parent e) name)]))

(define (env-bound? e name)
  (cond
    [(not e) #f]
    [(hash-has-key? (env-frame e) name) #t]
    [else (env-bound? (env-parent e) name)]))

(define (env-define! e name val)
  (hash-set! (env-frame e) name val))

(define (env-set! e name val)
  (cond
    [(hash-has-key? (env-frame e) name)
     (hash-set! (env-frame e) name val)]
    [(env-parent e) (env-set! (env-parent e) name val)]
    [else (error 'my-lisp "unbound identifier: ~a" name)]))

;; -----------------------------------------------------------------
;; Помічники істини / atom / eq
;; -----------------------------------------------------------------

(define (truthy? x)
  (not (null? x)))

(define (atom-val x)
  (if (pair? x) nil t))

(define (eq-val a b)
  (when (or (pair? a) (pair? b))
    (error 'eq "eq expects two atoms"))
  (if (and (number? a) (number? b))
      (if (and (= a b) (eq? (exact? a) (exact? b))) t nil)
      (if (equal? a b) t nil)))

(define (my-pred proc)
  (lambda args (if (apply proc args) t nil)))

;; -----------------------------------------------------------------
;; Читання файлів
;; -----------------------------------------------------------------

(define (read-file path)
  (define in (open-input-file path))
  (begin0
    (let loop ()
      (define v (my-read in))
      (if (eof-object? v) '() (cons v (loop))))
    (close-input-port in)))

(define (find-core-path)
  (define here (this-expression-source-directory))
  (define candidates
    (list (build-path here 'up "lib" "core.my")
          (build-path here "boot" "core.my")))
  (findf file-exists? candidates))

;; -----------------------------------------------------------------
;; Примітиви
;; -----------------------------------------------------------------

(define (register-primitives! e)
  ;; Арифметика
  (env-define! e '+ (my-primitive '+ (lambda args (apply + args))))
  (env-define! e '- (my-primitive '- (lambda args (apply - args))))
  (env-define! e '* (my-primitive '* (lambda args (apply * args))))
  (env-define! e '/ (my-primitive '/ (lambda args (apply / args))))
  ;; Порівняння
  (env-define! e '< (my-primitive '< (my-pred <)))
  (env-define! e '> (my-primitive '> (my-pred >)))
  (env-define! e '= (my-primitive '= (my-pred =)))
  ;; Списки / Маккарті
  (env-define! e 'atom (my-primitive 'atom atom-val))
  (env-define! e 'eq   (my-primitive 'eq   eq-val))
  (env-define! e 'car  (my-primitive 'car  (lambda (p) (if (pair? p) (car p) (error 'car "expected pair")))))
  (env-define! e 'cdr  (my-primitive 'cdr  (lambda (p) (if (pair? p) (cdr p) (error 'cdr "expected pair")))))
  (env-define! e 'cons (my-primitive 'cons cons))
  ;; Рядки / символи
  (env-define! e 'string-first  (my-primitive 'string-first  (lambda (s) (substring s 0 1))))
  (env-define! e 'string-rest   (my-primitive 'string-rest   (lambda (s) (substring s 1))))
  (env-define! e 'string-append (my-primitive 'string-append (lambda args (apply string-append args))))
  (env-define! e 'symbol->string (my-primitive 'symbol->string symbol->string))
  (env-define! e 'string->symbol (my-primitive 'string->symbol string->symbol))
  (env-define! e 'write-to-string
    (my-primitive 'write-to-string
                  (lambda (v)
                    (define out (open-output-string))
                    (parameterize ([print-as-expression #f])
                      (write v out))
                    (get-output-string out))))
  ;; Предикати
  (env-define! e 'number? (my-primitive 'number? (my-pred number?)))
  (env-define! e 'string? (my-primitive 'string? (my-pred string?)))
  ;; Введення-виведення
  (env-define! e 'display   (my-primitive 'display   (lambda (x) (display x) x)))
  (env-define! e 'displayln (my-primitive 'displayln (lambda (x) (displayln x) x)))
  (env-define! e 'print     (my-primitive 'print     (lambda (x) (print x) x)))
  (env-define! e 'princ     (my-primitive 'princ     (lambda (x) (display x) x)))
  (env-define! e 'read
    (my-primitive 'read
                  (case-lambda
                    [() (my-read (current-input-port))]
                    [(s) (if (string? s)
                             (my-read (open-input-string s))
                             (my-read s))])))
  (env-define! e 'read-all
    (my-primitive 'read-all
                  (lambda (s)
                    (define in (open-input-string s))
                    (let loop ()
                      (define v (my-read in))
                      (if (eof-object? v) '() (cons v (loop)))))))
  ;; Значення істини
  (env-define! e 't t)
  (env-define! e 'nil nil))

;; -----------------------------------------------------------------
;; Замикання / макроси
;; -----------------------------------------------------------------

(define (bind-args! params args new-env)
  (cond
    [(null? params)
     (unless (null? args) (error 'my-lisp "too many arguments"))]
    [(null? args) (error 'my-lisp "too few arguments")]
    [else
     (env-define! new-env (car params) (car args))
     (bind-args! (cdr params) (cdr args) new-env)]))

(define (make-closure params body env)
  (if (symbol? params)
      (my-closure params #t body env)
      (my-closure params #f body env)))

(define (apply-closure clo args env eval-loop)
  (define new-env (make-env (my-closure-env clo)))
  (if (my-closure-variadic? clo)
      (env-define! new-env (my-closure-params clo) args)
      (bind-args! (my-closure-params clo) args new-env))
  (eval-sequence (my-closure-body clo) new-env eval-loop))

(define (define-macro! env args)
  (match args
    [(list (cons name params) body ...)
     (define clo (make-closure params body env))
     (env-define! env name (my-macro clo))
     nil]
    [(list name params body ...)
     (define-macro! env (list (cons name params) body ...))]
    [_ (error 'my-lisp "bad defmacro syntax")]))

;; -----------------------------------------------------------------
;; Послідовності
;; -----------------------------------------------------------------

(define (eval-sequence exprs env eval-loop)
  (cond
    [(null? exprs) nil]
    [(null? (cdr exprs)) (eval-loop (car exprs) env)]
    [else
     (eval-loop (car exprs) env)
     (eval-sequence (cdr exprs) env eval-loop)]))

(define (eval-cond clauses env eval-loop)
  (if (null? clauses)
      nil
      (let ([clause (car clauses)])
        (if (truthy? (eval-loop (car clause) env))
            (eval-sequence (cdr clause) env eval-loop)
            (eval-cond (cdr clauses) env eval-loop)))))

(define (apply-proc proc args env eval-loop)
  (cond
    [(my-closure? proc) (apply-closure proc args env eval-loop)]
    [(my-primitive? proc) (apply (my-primitive-proc proc) args)]
    [(procedure? proc) (apply proc args)]
    [else (error 'my-lisp "not a function: ~a" proc)]))

;; -----------------------------------------------------------------
;; Головний evaluator
;; -----------------------------------------------------------------

(define (my-eval expr env)
  (let eval-loop ([expr expr] [env env])
    (cond
      [(or (number? expr) (string? expr) (my-true? expr) (null? expr)) expr]
      [(symbol? expr) (env-lookup env expr)]
      [(not (pair? expr)) expr]
      [else
       (define op (car expr))
       (define args (cdr expr))
       (case op
         [(quote)
          (if (null? args) nil (car args))]
         [(atom)
          (atom-val (eval-loop (car args) env))]
         [(eq)
          (eq-val (eval-loop (car args) env)
                  (eval-loop (cadr args) env))]
         [(car)
          (define v (eval-loop (car args) env))
          (if (pair? v) (car v) (error 'car "expected pair"))]
         [(cdr)
          (define v (eval-loop (car args) env))
          (if (pair? v) (cdr v) (error 'cdr "expected pair"))]
         [(cons)
          (cons (eval-loop (car args) env)
                (eval-loop (cadr args) env))]
         [(cond)
          (eval-cond args env eval-loop)]
         [(if)
          (if (truthy? (eval-loop (car args) env))
              (eval-loop (cadr args) env)
              (if (null? (cddr args))
                  nil
                  (eval-loop (caddr args) env)))]
         [(lambda)
          (make-closure (car args) (cdr args) env)]
         [(def)
          (env-define! env (car args) (eval-loop (cadr args) env))
          nil]
         [(defmacro)
          (define-macro! env args)
          nil]
         [(begin)
          (eval-sequence args env eval-loop)]
         [(load)
          (define path (eval-loop (car args) env))
          (unless (string? path) (error 'load "expected string path"))
          (eval-sequence (read-file path) env eval-loop)]
         [(set!)
          (env-set! env (car args) (eval-loop (cadr args) env))
          nil]
         [else
          (define proc (eval-loop op env))
          (cond
            [(my-macro? proc)
             (define expanded (apply-closure (my-macro-closure proc) args env eval-loop))
             (eval-loop expanded env)]
            [else
             (define evaled-args (map (lambda (a) (eval-loop a env)) args))
             (apply-proc proc evaled-args env eval-loop)])])])))

;; -----------------------------------------------------------------
;; Початкове середовище з lib/core.my
;; -----------------------------------------------------------------

(define (make-initial-env)
  (define e (make-env))
  (register-primitives! e)
  (define core-path (find-core-path))
  (unless core-path
    (error 'my-lisp "cannot find core.my; expected ../lib/core.my or boot/core.my"))
  (define forms (read-file core-path))
  (eval-sequence forms e my-eval)
  e)

;; -----------------------------------------------------------------
;; Запуск модуля / REPL
;; -----------------------------------------------------------------

(define (run-module forms)
  (define env (make-initial-env))
  (eval-sequence forms env my-eval))

(provide
 ;; evaluator / sequences
 my-eval eval-sequence
 ;; environments
 make-env env-lookup env-bound? env-define! env-set!
 ;; values
 t nil my-true? my-closure? my-macro? my-primitive?
 ;; module / file loading
 make-initial-env run-module read-file
 ;; needed by main.rkt for REPL echo
 atom-val eq-val)