#lang info
;;;
;;; info.rkt — опис пакета my-lisp для системи пакетів Racket (raco)
;;;

;; Назва колекції: саме за нею мова буде доступна як `#lang my-lisp`.
(define collection "my-lisp")

(define version "0.1")
(define pkg-desc
  "my-lisp — канонічний Lisp (примітиви Маккарті) як мова #lang для Racket/DrRacket")
(define pkg-authors '(my-lisp))
(define license 'MIT)

;; Залежності пакета. "base" — мінімальне ядро Racket;
;; у повному дистрибутиві (з DrRacket) усе решта вже є.
(define deps '("base"))
(define build-deps '())

;; ---------------------------------------------------------------------
;; Реєстрація мови в DrRacket.
;;
;; drracket-language-modules — список (шлях-до-модуля назва...):
;;   * "main.rkt"  — модуль мови (шлях відносно цієї колекції);
;;   * "my-lisp"   — назва, яку DrRacket покаже в діалозі вибору мови
;;                   (Language → Choose Language…).
;;
;; Завдяки цьому DrRacket розуміє файли, що починаються з
;; `#lang my-lisp` (зокрема канонічні файли *.my цього проєкту).
;; Сам же синтаксис `#lang my-lisp` обслуговує lang/reader.rkt.
;; ---------------------------------------------------------------------
(define drracket-language-modules
  '(("main.rkt" "my-lisp")))

;; Позиція мови в ієрархії діалогу вибору мови DrRacket:
;; верхній рівень, назва пункту — "my-lisp".
(define drracket-language-positions
  '(("my-lisp")))
