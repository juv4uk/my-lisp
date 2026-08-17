#lang racket/base
;;;
;;; lang/reader.rkt — точка входу для `#lang my-lisp`.
;;;
;;; Рядок `#lang X` вказує Racket шукати reader саме у модулі
;;; X/lang/reader. Вся логіка живе в ../reader-lib.rkt — тут лише
;;; реекспорт, щоб дотриматися цього контракту.
;;;

(require "../reader-lib.rkt")

(provide (all-from-out "../reader-lib.rkt"))
