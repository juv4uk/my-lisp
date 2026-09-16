; tests/fixtures/witness-runner.lisp — Lisp-owned semantic witness verdict protocol.
; tests/fixtures/witness-runner.lisp — протокол вердиктів семантичних свідчень, яким володіє Lisp.
;
; Authority stays in tests/fixtures/conformance.lisp. This file does NOT copy
; expected values. It receives one existing fixture row plus an implementation's
; actual outcome and decides PASS/FAIL as Lisp data. Host code may transport an
; actual value/error into the canonical `(value "...")` / `(error "Kind")`
; envelope, but it must not invent the expected answer.
;
; This runner intentionally lives under tests/fixtures rather than lib/: it is
; executable semantic-test authority, not a public language API surface.
;
; Semantic peer-surface truth stays in lib/surface/semantic-registry.lisp via
; lib/generated/meta-semantic-registry.lisp. The peer witness below consumes that
; projection; it does not own a second surface table.

(def witness-field
  (lambda (key witness)
    (let ((entry (assoc key witness)))
      (cond
        ((atom entry) (quote ()))
        (t (cdr entry))))))

(def witness-malformed-result
  (lambda (reason actual)
    (list (quote witness-result)
          (list (quote status) (quote malformed))
          (list (quote reason) reason)
          (list (quote actual) actual))))

(def witness-result-record
  (lambda (status expected actual)
    (list (quote witness-result)
          (list (quote status) status)
          (list (quote expected) expected)
          (list (quote actual) actual))))

; Convert one authoritative conformance row into a canonical expected-outcome
; envelope without changing/copying its expected/error fact.
(def witness-expected-outcome
  (lambda (witness)
    (let ((expected-entry (assoc (quote expected) witness))
          (error-entry (assoc (quote error) witness)))
      (cond
        ((and expected-entry error-entry)
         (list (quote malformed) (quote expected-and-error)))
        ((and (atom expected-entry) (atom error-entry))
         (list (quote malformed) (quote missing-outcome)))
        (expected-entry
         (list (quote value) (cdr expected-entry)))
        (t
         (list (quote error) (cdr error-entry)))))))

; The normative comparator. Backends provide ACTUAL only. Expected truth is read
; from WITNESS and compared here, in Lisp.
(def witness-verdict
  (lambda (witness actual)
    (let ((expected (witness-expected-outcome witness)))
      (cond
        ((eq (car expected) (quote malformed))
         (witness-malformed-result (second expected) actual))
        ((equal? expected actual)
         (witness-result-record (quote pass) expected actual))
        (t
         (witness-result-record (quote fail) expected actual))))))

; #218/#220 transition: host observers consume an explicit status datum instead
; of asking Lisp for a universal truth value. A witness result has canonical
; shape `(witness-result (status STATUS) ...)`, so status extraction requires no
; boolean interpretation at all.
(def witness-status
  (lambda (result)
    (second (second result))))

; Migration-only adapter for older host observers not yet converted to
; `witness-status`. It dispatches on the explicit status datum; it does NOT
; coerce an arbitrary Lisp value to truth. New/modified observers must use
; `witness-status` directly. Remove this with the last old observer under #220.
(def witness-pass?
  (lambda (result)
    (cond
      ((witness-status result) pass t)
      ((witness-status result) fail (quote ()))
      ((witness-status result) malformed (quote ()))
      (t (quote ())))))

; Meta-eval errors are Lisp data, not host exceptions. Normalize only the named
; correspondence already established by the meta-evaluator evidence. The mapping
; lives in Lisp so a Rust observer does not become the semantic classifier.
(def witness-meta-error-kind
  (lambda (kind)
    (cond
      ((eq kind (quote unbound-symbol)) "UnknownSymbol")
      ((eq kind (quote not-callable)) "Type")
      ((eq kind (quote arity)) "Arity")
      ((eq kind (quote invalid-form)) "InvalidForm")
      (t "UnsupportedMetaError"))))

(def witness-meta-error?
  (lambda (value)
    (cond
      ((atom value) (quote ()))
      ((atom (car value)) (eq (car value) (quote error)))
      (t (quote ())))))

(def witness-meta-outcome
  (lambda (value)
    (cond
      ((witness-meta-error? value)
       (list (quote error) (witness-meta-error-kind (second value))))
      (t
       (list (quote value) (write-to-string value))))))

; Registry-driven peer-surface witness. The semantic ID is a selection input;
; surface→ID truth comes only from my-semantic-surface-registry.
(def witness-peer-surface-count
  (lambda (semantic-id entries)
    (cond
      ((atom entries) 0)
      ((equal? (second (car entries)) semantic-id)
       (+ 1 (witness-peer-surface-count semantic-id (cdr entries))))
      (t
       (witness-peer-surface-count semantic-id (cdr entries))))))

(def witness-peer-surfaces-consistent?
  (lambda (semantic-id entries)
    (cond
      ((atom entries) t)
      ((equal? (second (car entries)) semantic-id)
       (cond
         ((equal? (my-semantic-id-for-surface (car (car entries))) semantic-id)
          (witness-peer-surfaces-consistent? semantic-id (cdr entries)))
         (t (quote ()))))
      (t
       (witness-peer-surfaces-consistent? semantic-id (cdr entries))))))

(def witness-peer-surface-verdict
  (lambda (semantic-id)
    (let ((count (witness-peer-surface-count semantic-id my-semantic-surface-registry)))
      (cond
        ((and (> count 1)
              (witness-peer-surfaces-consistent? semantic-id my-semantic-surface-registry))
         (list (quote witness-result)
               (list (quote status) (quote pass))
               (list (quote semantic-id) semantic-id)
               (list (quote surface-count) count)))
        (t
         (list (quote witness-result)
               (list (quote status) (quote fail))
               (list (quote semantic-id) semantic-id)
               (list (quote surface-count) count)))))))
