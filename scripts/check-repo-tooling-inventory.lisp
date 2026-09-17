; #382 — RED-first witness for the Lisp-owned repository tooling inventory.
; Production validation deliberately does not exist in this commit.

(def repo-tooling-selftest-unregistered
  (lambda ()
    (repo-tooling-verdict
      (quote
        ((tool
           (path "scripts/a.lisp")
           (kind check)
           (language lisp)
           (role sample)
           (lifecycle active)
           (callers ())
           (authority-source (issue 382))
           (migration-issue ())
           (replacement ())
           (removal-condition ())))))
      (quote ("a.lisp" "b.lisp")))))

(print (repo-tooling-selftest-unregistered))
