; #76 — Lisp-owned replacement for generate-meta-semantic-registry.py.
;
; Semantic authority remains lib/surface/semantic-registry.lisp. This script
; owns projection mechanics only and deliberately reads the registry as Lisp
; data instead of re-parsing Lisp source with host-language regular expressions.
;
; New control authored here uses canonical three-part cond dispatch. We do not
; add new callers of the migration-only Value -> bool bridge while #217/#218
; are moving structural and identity observations to explicit result domains.
;
; During the parity phase the generated provenance line still names the Python
; generator so --check proves byte-for-byte equivalence with the committed
; pre-migration artifact. The cutover commit changes that one provenance line
; together with the generated file and removes the Python generator.

; Temporary #74 profiling witness. Removed after the dominant stage is proven.
(def profile-script-start (mono-ns))

(def fail-closed
  (lambda (message)
    (cons (print message) (car (quote ())))))

; The registry's literal apostrophe surface is read as quote shorthand:
;   (sym ' stable) -> (sym (quote stable))
; Normalize that exceptional reader-only spelling to the string "'" so it can
; be excluded explicitly. All ordinary surfaces remain 3-element rows.
(def normalize-surface
  (lambda (raw)
    (cond
      ((eq (length raw) 3) (identity-relation same)
       raw)
      ((eq (length raw) 2) (identity-relation same)
       (cond
         ((atom (second raw)) (structural-kind pair)
          (cond
            ((eq (car (second raw)) (quote quote)) (identity-relation same)
             (list (car raw) "'" (second (second raw))))
            ((eq (car (second raw)) (quote quote)) (identity-relation distinct)
             raw)))
         ((atom (second raw)) (structural-kind empty-list) raw)
         ((atom (second raw)) (structural-kind atom) raw)))
      (t t raw))))

(def normalize-entry
  (lambda (entry)
    (cons (car entry) (map normalize-surface (cdr entry)))))

(def pad4
  (lambda (n)
    (let ((s (number->string n)))
      (cond
        ((eq (string-length s) 1) (identity-relation same)
         (string-append "000" s))
        ((eq (string-length s) 2) (identity-relation same)
         (string-append "00" s))
        ((eq (string-length s) 3) (identity-relation same)
         (string-append "0" s))
        (t t s)))))

(def admitted-status
  (lambda (status)
    (cond
      ((eq status (quote stable)) (identity-relation same)
       (quote admitted))
      ((eq status (quote compatibility-only)) (identity-relation same)
       (quote admitted))
      (t t (quote denied)))))

(def word-admission
  (lambda (word)
    (cond
      ((equal? word (quote —)) (structural-relation same)
       (quote excluded))
      ((equal? word "'") (structural-relation same)
       (quote excluded))
      (t t (quote included)))))

; Seen rows are (spelling semantic-id). equal? is consumed through its explicit
; structural-relation result, never through the temporary boolean bridge.
(def find-seen
  (lambda (word seen)
    (cond
      ((atom seen) (structural-kind empty-list)
       (quote ()))
      ((atom seen) (structural-kind pair)
       (cond
         ((equal? word (car (car seen))) (structural-relation same)
          (car seen))
         ((equal? word (car (car seen))) (structural-relation distinct)
          (find-seen word (cdr seen)))))
      ((atom seen) (structural-kind atom)
       (fail-closed "meta semantic registry generation failed: malformed seen table")))))

; State is (reversed-output-rows seen-spellings).
; An output row is (spelling semantic-id namespace).
(def collect-surface
  (lambda (sid raw-surface state)
    (let* ((surface (normalize-surface raw-surface))
           (rows (car state))
           (seen (second state)))
      (cond
        ((eq (length surface) 3) (identity-relation distinct)
         (fail-closed "meta semantic registry generation failed: malformed surface row"))
        ((eq (length surface) 3) (identity-relation same)
         (let* ((namespace (car surface))
                (word (second surface))
                (status (third surface)))
           (cond
             ((eq (admitted-status status) (quote denied)) (identity-relation same)
              state)
             ((eq (word-admission word) (quote excluded)) (identity-relation same)
              state)
             ((eq (symbol? word) t) (identity-relation distinct)
              (fail-closed "meta semantic registry generation failed: admitted surface is not one symbol"))
             ((eq (symbol? word) t) (identity-relation same)
              (let ((previous (find-seen word seen)))
                (cond
                  ((atom previous) (structural-kind empty-list)
                   (list (cons (list word sid namespace) rows)
                         (cons (list word sid) seen)))
                  ((atom previous) (structural-kind pair)
                   (cond
                     ((eq (second previous) sid) (identity-relation same)
                      state)
                     ((eq (second previous) sid) (identity-relation distinct)
                      (fail-closed "meta semantic registry generation failed: ambiguous admitted surface"))))
                  ((atom previous) (structural-kind atom)
                   (fail-closed "meta semantic registry generation failed: malformed seen row"))))))))))))

(def collect-surfaces
  (lambda (sid surfaces state)
    (cond
      ((atom surfaces) (structural-kind empty-list)
       state)
      ((atom surfaces) (structural-kind pair)
       (collect-surfaces sid (cdr surfaces)
                         (collect-surface sid (car surfaces) state)))
      ((atom surfaces) (structural-kind atom)
       (fail-closed "meta semantic registry generation failed: improper surface list")))))

(def collect-entries
  (lambda (entries state)
    (cond
      ((atom entries) (structural-kind empty-list)
       state)
      ((atom entries) (structural-kind pair)
       (let ((entry (normalize-entry (car entries))))
         (collect-entries
           (cdr entries)
           (collect-surfaces (car entry) (cdr entry) state))))
      ((atom entries) (structural-kind atom)
       (fail-closed "meta semantic registry generation failed: improper registry entry list")))))

(def str+
  (lambda args
    (reduce (lambda (acc s) (string-append acc s)) "" args)))

(def join-newline-onto
  (lambda (strings acc)
    (cond
      ((atom strings) (structural-kind empty-list)
       acc)
      ((atom strings) (structural-kind pair)
       (cond
         ((equal? acc "") (structural-relation same)
          (join-newline-onto (cdr strings) (car strings)))
         ((equal? acc "") (structural-relation distinct)
          (join-newline-onto (cdr strings)
                             (str+ acc "\n" (car strings))))))
      ((atom strings) (structural-kind atom)
       (fail-closed "meta semantic registry generation failed: improper rendered row list")))))

(def join-newline
  (lambda (strings) (join-newline-onto strings "")))

(def render-row
  (lambda (row)
    (str+ "    (" (write-to-string (car row))
          " \"" (pad4 (second row)) "\") ; "
          (write-to-string (third row)))))

(def render-projection
  (lambda (rows)
    (str+
      "; GENERATED FILE — DO NOT EDIT.\n"
      "; Source authority: lib/surface/semantic-registry.lisp\n"
      "; Generator: scripts/generate-meta-semantic-registry.py\n"
      "; stable + compatibility-only runtime surfaces only; exact ' is reader syntax.\n\n"
      "(def my-semantic-surface-registry\n"
      "  (quote (\n"
      (join-newline (map render-row rows))
      "\n  )))\n\n"
      "(def my-semantic-id-for-surface\n"
      "  (lambda (name)\n"
      "    (let ((entry (assoc name my-semantic-surface-registry)))\n"
      "      (cond\n"
      "        ((atom entry) (quote ()))\n"
      "        (t (second entry))))))\n")))

(def profile-before-registry-read (mono-ns))
(def registry-form
  (car (read-all (read-file "lib/surface/semantic-registry.lisp"))))
(def profile-after-registry-read (mono-ns))

(cond
  ((eq (car registry-form) (quote sr/1)) (identity-relation same)
   (quote registry-ok))
  ((eq (car registry-form) (quote sr/1)) (identity-relation distinct)
   (fail-closed "meta semantic registry generation failed: expected sr/1 registry")))

(def collected
  (collect-entries (cdr registry-form) (list (quote ()) (quote ()))))
(def profile-after-collect (mono-ns))
(def projection-rows (reverse (car collected)))

(cond
  ((atom projection-rows) (structural-kind empty-list)
   (fail-closed "meta semantic registry generation failed: registry has no admitted runtime surfaces"))
  ((atom projection-rows) (structural-kind pair)
   (quote projection-ok))
  ((atom projection-rows) (structural-kind atom)
   (fail-closed "meta semantic registry generation failed: malformed projection rows")))

(def generated (render-projection projection-rows))
(def profile-after-render (mono-ns))
(def output-path "lib/generated/meta-semantic-registry.lisp")
(def mode
  (cond
    ((atom *argv*) (structural-kind empty-list) "generate")
    ((atom *argv*) (structural-kind pair) (car *argv*))
    ((atom *argv*) (structural-kind atom)
     (fail-closed "meta semantic registry generation failed: malformed argv"))))

(print
  (list (quote meta-registry-profile-ns)
        (list (quote definitions)
              (- profile-before-registry-read profile-script-start))
        (list (quote registry-read)
              (- profile-after-registry-read profile-before-registry-read))
        (list (quote collect)
              (- profile-after-collect profile-after-registry-read))
        (list (quote render)
              (- profile-after-render profile-after-collect))
        (list (quote total-to-render)
              (- profile-after-render profile-script-start))))

(cond
  ((equal? mode "--check") (structural-relation same)
   (let ((current (read-file output-path)))
     (cond
       ((equal? current generated) (structural-relation same)
        (print "meta semantic registry projection is current"))
       ((equal? current generated) (structural-relation distinct)
        (fail-closed "meta semantic registry projection is stale")))))
  ((equal? mode "--check") (structural-relation distinct)
   (cons (write-file output-path generated)
         (print "meta semantic registry projection written"))))
