#lang racket
;;;
;;; main.rkt — ядро мови my-lisp для Racket/DrRacket
;;;
;;; Відповідає семантиці my-lisp (Rust-реалізації):
;;;   * quote тільки явний — (quote x); апостроф ' є частиною символу;
;;;   * десяткові літерали читаються як exact (0.5 → 1/2);
;;;   * істина = t, хиба = ();
;;;   * atom/eq повертають t або ();
;;;   * eq порівнює рядки за вмістом, символи за іменем, числа за
;;;     значенням і exactness (як у Value::PartialEq вашого value.rs);
;;;   * def — синонім define;
;;;   * echo-політика REPL: нев'язаний одиночний ідентифікатор друкує
;;;     "echo <id>" замість помилки (через #%top-interaction).
;;;

;; -----------------------------------------------------------------
;; Значення істини
;; -----------------------------------------------------------------

;; t — друкується як "t" і є істиною.
(struct my-lisp-true ()
  #:methods gen:custom-write
  [(define (write-proc v port mode)
     (write-string "t" port))])

(define t (my-lisp-true))
(define nil '())

(define (my-truthy? x)
  ;; Хибні лише () і Racket-#f (остання страховка).
  (and (not (null? x)) (not (eq? x #f))))

;; -----------------------------------------------------------------
;; Примітиви Маккарті
;; -----------------------------------------------------------------

(define (atom x)
  ;; atom? повертає t для атомів, () для пар.
  (if (pair? x) '() t))

(define atom? atom)

(define (my-eq a b)
  ;; eq працює тільки для атомів; пари — Type-помилка.
  (cond
    [(or (pair? a) (pair? b))
     (raise-arguments-error 'eq "eq expects two atoms")]
    [(and (number? a) (number? b))
     ;; Числа рівні, якщо значення і exactness збігаються.
     (if (and (= a b) (eq? (exact? a) (exact? b))) t '())]
    [(and (string? a) (string? b))
     (if (string=? a b) t '())]
    [(and (symbol? a) (symbol? b))
     (if (eq? a b) t '())]
    [(and (null? a) (null? b)) t]
    [(and (boolean? a) (boolean? b))
     (if (eq? a b) t '())]
    [(and (my-lisp-true? a) (my-lisp-true? b)) t]
    [(or (my-lisp-true? a) (my-lisp-true? b)) '()]
    [else (if (equal? a b) t '())]))

(define-syntax (my-if stx)
  (syntax-case stx ()
    ;; Класичний if: else можна опускати (результат — ()).
    [(_ test then)       #'(if (my-truthy? test) then (quote ()))]
    [(_ test then else)  #'(if (my-truthy? test) then else)]))

(define-syntax (my-cond stx)
  (syntax-case stx ()
    ;; Кожна умова — рівно (test expr); t/() працюють як істина/хиба.
    [(_ (test expr))     #'(if (my-truthy? test) expr (quote ()))]
    [(_ (test expr) rest ...)
     #'(if (my-truthy? test) expr (my-cond rest ...))]
    [(_)                 #'(quote ())]))

;; -----------------------------------------------------------------
;; Точне ділення
;; -----------------------------------------------------------------

(define (my-/ . args)
  ;; Racket сам поводиться правильно: exact-операнди → exact,
  ;; один inexact → inexact. Ділення на нуль кине Racket-виняток.
  (when (null? args)
    (raise-argument-error '/ "at least one argument" args))
  (apply / args))

;; -----------------------------------------------------------------
;; def — як у вашому evaluate_definition
;; -----------------------------------------------------------------

(define-syntax (my-def stx)
  (syntax-case stx ()
    [(_ . args) #'(define . args)]))

;; -----------------------------------------------------------------
;; print/princ (відповідно до value.rs Display/to_princ_string)
;; -----------------------------------------------------------------

(define (my-print x)
  (print x)
  x)

(define princ display)

;; -----------------------------------------------------------------
;; defmacro — традиційні ліспові макроси (compile-time).
;;
;; Працює для макросів, тіло яких використовує лише базові форми
;; (quote, cons, car, cdr, if, cond, lambda...). Макроси, що в тілі
;; посилаються на інші користувацькі визначення того ж модуля
;; (напр. (defmacro let ...) з lib/core.my, що викликає second/map),
;; потребують runtime-макросів, які Racket не підтримує напряму.
;; -----------------------------------------------------------------

(define-syntax (defmacro stx)
  (syntax-case stx ()
    [(_ (name . params) body ...)
     #'(define-syntax (name stx)
         (datum->syntax
          stx
          (apply (lambda params body ...)
                 (cdr (syntax->datum stx)))))]
    [(_ name params body ...)
     #'(defmacro (name . params) body ...)]))

;; -----------------------------------------------------------------
;; Порожній список () має бути самооцінюваним (Nil), як у my-lisp.
;; Racket за замовчуванням сприймає () як порожній виклик (#%app) —
;; перехоплюємо це на рівні #%app.
;; -----------------------------------------------------------------

(define-syntax (my-app stx)
  (syntax-case stx ()
    ;; () → '()
    [(_) #'(quote ())]
    [(_ rator rand ...) #'(#%app rator rand ...)]))

(define-syntax (my-datum stx)
  (syntax-case stx ()
    [(_ . v) #'(#%datum . v)]))

;; -----------------------------------------------------------------
;; module-begin: вимикаємо quote-стиль друку символів, щоб x друкувався
;; як x, а не як 'x.
;; -----------------------------------------------------------------

(define-syntax (my-module-begin stx)
  (syntax-case stx ()
    [(_ . forms)
     (let ([setup (datum->syntax stx '(current-print
                                       (let ([p (current-print)])
                                         (lambda (v)
                                           (parameterize ([print-as-expression #f])
                                             (p v))))))])
       (datum->syntax stx (cons #'#%module-begin
                                (cons setup (syntax->list #'forms)))))]))

;; -----------------------------------------------------------------
;; echo-політика інтерактивного REPL / DrRacket
;; -----------------------------------------------------------------

(define-syntax (my-top-interaction stx)
  (syntax-case stx ()
    ;; Одиночний нев'язаний ідентифікатор → echo.
    [(_ . id)
     (identifier? #'id)
     (if (identifier-binding #'id)
         #'(#%top-interaction . id)
         #'(begin
             (printf "echo ~a\n" 'id)
             (void)))]
    [(_ . form)
     #'(#%top-interaction . form)]))

;; -----------------------------------------------------------------
;; Експорт мови
;; -----------------------------------------------------------------

(provide
 ;; Базовий Racket (lambda, let, car, cdr, cons, quote, require, provide,
 ;; #%module-begin, #%app, #%datum, #%top), але замінюємо / if cond define
 ;; та #%top-interaction:
 (except-out (all-from-out racket) / if cond define #%module-begin #%app #%datum #%top-interaction print)
 (rename-out [my-/ /]
             [my-if if]
             [my-cond cond]
             [my-def def]
             [my-module-begin #%module-begin]
             [my-app #%app]
             [my-datum #%datum]
             [my-top-interaction #%top-interaction]
             [my-print print]
             [my-eq eq])
 ;; Значення істини:
 t nil
 ;; Примітиви Маккарті:
 atom atom?
 ;; Макроси:
 defmacro
 ;; Арифметика/порівняння (Racket-семантика точна для exact/inexact):
 + - * < > =
 ;; Вивід:
 display displayln princ)
