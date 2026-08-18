#lang racket
;;;
;;; racket/tests/script-mode.rkt — checks that `racket file.my` prints
;;; the last top-level form's value, matching the Rust CLI's file-mode
;;; behavior (`my-lisp file.my` with a bare `(/ 5 336)` prints
;;; `5/336`). Found broken 2026-08-18: `#%module-begin` suppressed ALL
;;; top-level output to avoid double-printing every form (Racket's
;;; default module-begin echoes each one), but went too far and
;;; suppressed the final value too — a script whose last form wasn't
;;; wrapped in `print` produced no output at all. This can't be tested
;;; by requiring interpreter.rkt directly (conformance.rkt does that);
;;; the bug lived in main.rkt's `my-module-begin`, which only runs when
;;; a real `#lang my-lisp` module is instantiated, so this spawns
;;; `racket` as a subprocess against a temp file the same way a user
;;; actually would.

(module+ test
  (require rackunit racket/system racket/port)

  (define (run-my-lisp-file source)
    (define path (make-temporary-file "script-mode-check~a.my"))
    (call-with-output-file path #:exists 'replace
      (lambda (out) (displayln source out)))
    (define-values (proc out in err)
      (subprocess #f #f #f (find-executable-path "racket") (path->string path)))
    (close-output-port in)
    (define stdout (port->string out))
    (subprocess-wait proc)
    (delete-file path)
    (string-trim stdout))

  (check-equal? (run-my-lisp-file "#lang my-lisp\n(/ 5 336)")
                "5/336"
                "a bare final expression must print its value, not silently produce nothing")
  (check-equal? (run-my-lisp-file "#lang my-lisp\n(+ 1 1)\n(+ 2 2)\n(/ 5 336)")
                "5/336"
                "only the last top-level form's value prints, matching eval_program")
  (check-equal? (run-my-lisp-file "#lang my-lisp\n(print (+ 1 1))")
                "2\n2"
                "print's own side-effect line plus the program's final-value line both print, same as the Rust CLI's println! per output line + result.value")
  (printf "script-mode: racket file.my prints the program's final value, matching the Rust CLI\n"))
