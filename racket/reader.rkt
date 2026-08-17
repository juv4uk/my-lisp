#lang s-exp syntax/module-reader
my-lisp
#:read my-read
#:read-syntax my-read-syntax
#:info info-proc

(require racket/base)

;; У my-lisp апостроф ' — це частина символу (напр. 'x, об'єкт), а НЕ
;; скорочення для quote. Мапимо його на звичайну поведінку символьного
;; символу (як #\x), і паралельно вмикаємо читання десяткових літералів
;; як exact Rational (0.5 → 1/2, 1e3 → 1000), як у вашому parser.rs.
(define my-readtable
  (make-readtable (current-readtable) #\' #\x #f))

(define (my-read in)
  (parameterize ([current-readtable my-readtable]
                 [read-decimal-as-inexact #f])
    (read in)))

(define (my-read-syntax src in)
  (parameterize ([current-readtable my-readtable]
                 [read-decimal-as-inexact #f])
    (read-syntax src in)))

(define (info-proc key defval default)
  (case key
    [(drracket:default-filters)
     '(("my-lisp (*.my)" "*.my"))]
    [(drracket:default-extension)
     "my"]
    [else
     (default key defval)]))
