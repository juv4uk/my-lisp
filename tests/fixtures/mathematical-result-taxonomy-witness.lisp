; #225 — Lisp-owned verifier for the mathematical-result taxonomy.
; The host transports contracts/mathematical-result-taxonomy.lisp into
; `mathematical-result-taxonomy-document`; every semantic expectation is here.

(def mrt-field
  (lambda (entry field)
    (let ((found (assoc field entry)))
      (cond
        ((atom found) (quote ()))
        (t (cdr found))))))

(def mrt-find
  (name entries)
  )
