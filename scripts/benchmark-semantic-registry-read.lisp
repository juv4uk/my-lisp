; #74 real workload harvested from #76 Python -> my-lisp migration.
; Measure the exact heavy phase profiled in PR #317: public Lisp-owned
; read-file followed by read-all of the real semantic registry. Do not call
; core.lisp's recursive string-length here: on a registry-sized String that is
; a separate non-tail recursion benchmark and can overflow the host stack.

(def started (mono-ns))
(def registry-form
  (car (read-all (read-file "lib/surface/semantic-registry.lisp"))))
(def finished (mono-ns))

(print
  (list (quote eco-lisp-script-perf/read-semantic-registry)
        (list (quote elapsed-ns) (- finished started))
        (list (quote root) (car registry-form))))
