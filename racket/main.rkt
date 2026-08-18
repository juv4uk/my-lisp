#lang racket
;;;
;;; main.rkt — точка входу мови #lang my-lisp для Racket/DrRacket.
;;;
;;; Вся реальна семантика живе в interpreter.rkt; цей файл лише
;;; адаптує її до контракту Racket-модуля (#%module-begin,
;;; #%top-interaction) та echo-політики REPL.
;;;

(require "interpreter.rkt")

;; Racket's default REPL value-echo (`current-print`) prints via
;; `print`, which prefixes quote-sugar on symbols/pairs (`'radio`,
;; `'(1 2)`) — the Rust reference REPL never does that (`radio`,
;; `(1 2)`). Route the REPL's own echo through the same
;; `my-format-string` the `print` primitive uses, so entering `(quote
;; radio)` at the REPL and calling `(print (quote radio))` show the
;; same thing, matching the Rust CLI.
(current-print
 (lambda (v)
   (unless (void? v)
     (displayln (my-format-string v)))))

;; Спільне середовище для REPL. Ініціалізується ліниво, щоб не
;; завантажувати core.my, поки користувач просто відкрив файл.
(define repl-env-box (box #f))

(define (ensure-repl-env!)
  (unless (unbox repl-env-box)
    (set-box! repl-env-box (make-initial-env)))
  (unbox repl-env-box))

;; -----------------------------------------------------------------
;; module-begin: інтерпретуємо тіло модуля як послідовність форм my-lisp
;; -----------------------------------------------------------------

(define-syntax (my-module-begin stx)
  (syntax-case stx ()
    [(_ . forms)
     (let ([datums (map syntax->datum (syntax->list #'forms))])
       (with-syntax ([datum-list (datum->syntax stx datums)])
         #'(#%module-begin
            (define module-env (make-initial-env))
            ;; let пригнічує host-echo: #%module-begin друкує значення
            ;; кожного top-level виразу, а результат eval-sequence є
            ;; внутрішньою справою my-lisp, не виводом програми.
            ;; (begin не працює — Racket flatten'ає top-level begin.)
            (let ([_ (eval-sequence 'datum-list module-env my-eval)])
              (set-box! repl-env-box module-env)
              (void)))))]))

;; -----------------------------------------------------------------
;; top-interaction: REPL / вікно Interactions у DrRacket
;; -----------------------------------------------------------------

(define (run-repl-form form)
  (define env (ensure-repl-env!))
  (cond
    ;; Одиночний нев'язаний ідентифікатор → echo.
    [(and (symbol? form) (not (env-bound? env form)))
     (display "echo ")
     (displayln form)
     (void)]
    [else
     (my-eval form env)]))

(define-syntax (my-top-interaction stx)
  (syntax-case stx ()
    [(_ . form)
     (let ([datum (syntax->datum #'form)])
       (with-syntax ([d (datum->syntax stx datum)])
         #'(run-repl-form 'd)))]))

;; -----------------------------------------------------------------
;; Експорт для Racket-модуля мови мови
;; -----------------------------------------------------------------

(provide
 (rename-out [my-module-begin #%module-begin]
             [my-top-interaction #%top-interaction]))
