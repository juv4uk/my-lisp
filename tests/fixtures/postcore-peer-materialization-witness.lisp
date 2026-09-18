; #469 — executable post-core peer materialization witness.
;
; The CLI loads core + time/process/fs before this file, but it does NOT load
; lib/surface/uk.lisp. Therefore the UK peer below exists only if the post-core
; library materialized it through the registry-driven mechanism.
;
; This witness also loads the generated registry projection as DATA evidence:
; the numeric semantic ID remains authority, candidates remain unadmitted, and
; the runtime peer projection for the exercised post-core range must neither
; invent nor omit an admitted spelling.

(load "lib/generated/meta-semantic-registry.lisp")

(def postcore-registry-surface-count
  (lambda (semantic-id-text entries)
    (cond
      ((atom entries) (structural-kind empty-list)
       0)
      ((atom entries) (structural-kind pair)
       (let ((entry (car entries)))
         (cond
           ((eq (second entry) semantic-id-text) (identity-relation same)
            (+ 1
               (postcore-registry-surface-count
                 semantic-id-text
                 (cdr entries))))
           ((eq (second entry) semantic-id-text) (identity-relation distinct)
            (postcore-registry-surface-count
              semantic-id-text
              (cdr entries)))))))))

(def postcore-peers-match-id
  (lambda (peers semantic-id-text)
    (cond
      ((atom peers) (structural-kind empty-list)
       (quote registry-consistent))
      ((atom peers) (structural-kind pair)
       (cond
         ((eq (my-semantic-id-for-surface (car peers)) semantic-id-text)
          (identity-relation same)
          (postcore-peers-match-id (cdr peers) semantic-id-text))
         ((eq (my-semantic-id-for-surface (car peers)) semantic-id-text)
          (identity-relation distinct)
          (quote registry-drift)))))))

(def postcore-groups-match-generated-registry
  (lambda (groups)
    (cond
      ((atom groups) (structural-kind empty-list)
       (quote registry-consistent))
      ((atom groups) (structural-kind pair)
       (let* ((group (car groups))
              (semantic-id (car group))
              (peers (cdr group))
              (semantic-id-text (number->string semantic-id))
              (projected-count
                (postcore-registry-surface-count
                  semantic-id-text
                  my-semantic-surface-registry)))
         (cond
           ((= projected-count (length peers)) 1
            (cond
              ((eq (postcore-peers-match-id peers semantic-id-text)
                   (quote registry-consistent))
               (identity-relation same)
               (postcore-groups-match-generated-registry (cdr groups)))
              ((eq (postcore-peers-match-id peers semantic-id-text)
                   (quote registry-drift))
               (identity-relation same)
               (quote registry-drift))))
           ((= projected-count (length peers)) 0
            (quote registry-drift))))))))

(def postcore-witness-failure
  (lambda (case actual expected)
    (list
      (quote postcore-peer-materialization-witness)
      (quote (status fail))
      (list (quote case) case)
      (list (quote actual) actual)
      (list (quote expected) expected))))

(def postcore-check-rows
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (quote (postcore-peer-materialization-witness (status pass))))
      ((atom rows) (structural-kind atom)
       (postcore-witness-failure
         (quote malformed-row-tail)
         rows
         (quote ())))
      ((atom rows) (structural-kind pair)
       (let ((row (car rows)))
         (cond
           ((equal? (second row) (third row)) (structural-relation same)
            (postcore-check-rows (cdr rows)))
           ((equal? (second row) (third row)) (structural-relation distinct)
            (postcore-witness-failure
              (car row)
              (second row)
              (third row)))))))))

(def postcore-phase-one
  (postcore-check-rows
    (list
      ; The hand-executed runtime projection must exactly match the generated
      ; admitted registry projection for every currently exercised time ID.
      (list
        (quote generated-registry-parity)
        (postcore-groups-match-generated-registry
          my-postcore-stable-peer-projection)
        (quote registry-consistent))

      ; Both stable spellings route to the same opaque numeric identity.
      (list
        (quote semantic-id-en)
        (my-semantic-id-for-surface (quote utc-from-unix))
        "1080")
      (list
        (quote semantic-id-uk)
        (my-semantic-id-for-surface (quote всч-із-юнікс))
        "1080")

      ; The full UKR spelling is still candidate. It must be absent from both
      ; the admitted registry projection and the live environment.
      (list
        (quote candidate-not-admitted)
        (atom
          (my-semantic-id-for-surface
            (quote всесвітній-координований-час-із-часу-юнікс)))
        (quote (structural-kind empty-list)))
      (list
        (quote candidate-not-bound)
        (my-postcore-binding-status
          (quote всесвітній-координований-час-із-часу-юнікс)
          (env))
        (quote absent))

      ; Stable EN/UK peers begin as exactly the same closure and execute with
      ; the same deterministic result.
      (list
        (quote initial-peer-identity)
        (eq utc-from-unix всч-із-юнікс)
        (quote (identity-relation same)))
      (list
        (quote invocation-parity)
        (equal? (utc-from-unix 0 0) (всч-із-юнікс 0 0))
        (quote (structural-relation same)))

      ; Ordinary lexical shadowing is independent: rebinding one spelling does
      ; not mutate or retarget the other spelling.
      (list
        (quote lexical-shadowing-independent)
        (let ((всч-із-юнікс
                (lambda (seconds nanosecond) (quote shadowed))))
          (eq utc-from-unix всч-із-юнікс))
        (quote (identity-relation distinct))))))

; Stronger idempotence law: install an explicit existing peer binding, invoke
; materialization again at top level, and prove the macro does NOT overwrite it.
; This mutation is local to this one-shot witness process.
(def postcore-existing-peer
  (lambda (seconds nanosecond) (quote preserved-peer)))

(def всч-із-юнікс postcore-existing-peer)
(my-postcore-materialize-stable-peers 1080 utc-from-unix)

(def postcore-phase-two
  (postcore-check-rows
    (list
      (list
        (quote phase-one)
        postcore-phase-one
        (quote (postcore-peer-materialization-witness (status pass))))
      (list
        (quote rematerialization-preserves-existing-binding)
        (eq всч-із-юнікс postcore-existing-peer)
        (quote (identity-relation same)))
      (list
        (quote rematerialization-does-not-retarget-source)
        (eq всч-із-юнікс utc-from-unix)
        (quote (identity-relation distinct)))
      (list
        (quote preserved-binding-invocation)
        (всч-із-юнікс 0 0)
        (quote preserved-peer)))))

postcore-phase-two
