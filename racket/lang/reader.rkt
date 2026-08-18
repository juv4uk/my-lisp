#lang s-exp syntax/module-reader
my-lisp
#:read my-read
#:read-syntax my-read-syntax
#:info info-proc

;;;
;;; lang/reader.rkt — точка входу для `#lang my-lisp`.
;;;
;;; Рядок `#lang X` вказує Racket шукати reader саме у модулі
;;; X/lang/reader. syntax/module-reader автоматично експортує
;;; read / read-syntax / info, використовуючи наші my-read,
;;; my-read-syntax та info-proc з reader-lib.rkt.
;;;

(require "../reader-lib.rkt")
