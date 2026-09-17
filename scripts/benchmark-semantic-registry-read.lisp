; #74 real workload harvested from #76 Python -> my-lisp migration.
; This intentionally measures the public Lisp-owned read-file path against the
; actual semantic registry used by scripts/generate-meta-semantic-registry.lisp.

(def started (mono-ns))
(def registry-text (read-file "lib/surface/semantic-registry.lisp"))
(def finished (mono-ns))

(print
  (list (quote eco-lisp-script-perf/read-semantic-registry)
        (list (quote elapsed-ns) (- finished started))
        (list (quote characters) (string-length registry-text))))
