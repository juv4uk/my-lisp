; #115/#220 — Lisp owns semantic equality for content-store transition witnesses.
; Rust may observe deterministic bytes and store cardinality, but it must not
; reinterpret `(structural-relation same|distinct)` as a host boolean.

(def content-store-semantic-witness
  (lambda ()
    (let* ((value (quote (lambda (x) x)))
           (written (fs-write (fs-empty) "code" value))
           (fs (car written))
           (root-a (fs-serialize-root fs))
           (root-b (fs-serialize-root fs))
           (object-a (fs-serialize-object value))
           (object-b (fs-serialize-object value))
           (direct
             (world-tell (empty-world) (quote zoo) (quote ((has-fur cat)))))
           (retold
             (world-tell
               (world-retract
                 (world-tell (empty-world) (quote zoo) (quote ((has-fur cat))))
                 (quote zoo) (quote ((has-fur cat))))
               (quote zoo) (quote ((has-fur cat)))))
           (root-relation (equal? root-a root-b))
           (object-relation (equal? object-a object-b))
           (projection-relation
             (equal? (world-clauses direct (quote zoo))
                     (world-clauses retold (quote zoo)))))
      (cond
        (root-relation (structural-relation same)
          (cond
            (object-relation (structural-relation same)
              (cond
                (projection-relation (structural-relation same)
                  (quote (content-store-witness (status pass))))
                (projection-relation (structural-relation distinct)
                  (quote (content-store-witness
                           (status fail)
                           (reason world-projection-distinct))))))
            (object-relation (structural-relation distinct)
              (quote (content-store-witness
                       (status fail)
                       (reason object-image-distinct))))))
        (root-relation (structural-relation distinct)
          (quote (content-store-witness
                   (status fail)
                   (reason root-image-distinct))))))))
