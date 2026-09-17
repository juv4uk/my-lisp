; #76 — Lisp-owned replacement for generate-meta-semantic-registry.py.
;
; Semantic authority remains lib/surface/semantic-registry.lisp.  This script
; owns projection mechanics only and deliberately reads the registry as Lisp
; data instead of re-parsing Lisp source with host-language regular expressions.
;
; During the parity phase the generated provenance line still names the Python
; generator so `--check` proves byte-for-byte equivalence with the committed
; pre-migration artifact.  The cutover commit changes that one provenance line
; together with the generated file and removes the Python generator.

(def fail-closed
  (lambda (message)
    (cons (print message) (car (quote ())))))

; The registry's literal apostrophe surface is read as quote shorthand:
;   (sym ' stable) -> (sym (quote stable))
; Normalize that exceptional reader-only spelling to the string "'" so it can
; be excluded explicitly.  All ordinary surfaces remain 3-element rows.
(def normalize-surface
  (lambda (raw)
    (cond
      ((eq (length raw) 3) raw)
      ((and (eq (length raw) 2)
            (not (atom (second raw)))
            (eq (car (second raw)) (quote quote)))
       (list (car raw) "'" (second (second raw))))
      (t raw))))

(def normalize-entry
  (lambda (entry)
    (cons (car entry) (map normalize-surface (cdr entry)))))

(def pad4
  (lambda (n)
    (let ((s (number->string n)))
      (cond
        ((eq (string-length s) 1) (string-append "000" s))
        ((eq (string-length s) 2) (string-append "00" s))
        ((eq (string-length s) 3) (string-append "0" s))
        (t s)))))

(def admitted-status?
  (lambda (status)
    (or (eq status (quote stable))
        (eq status (quote compatibility-only)))))

(def excluded-word?
  (lambda (word)
    (or (eq word (quote —))
        (and (string? word) (equal? word "'")))))

; Seen rows are (spelling semantic-id).  We use equal? for spelling so this
; remains correct for any future read-back-safe atomic spelling representation.
(def find-seen
  (lambda (word seen)
    (cond
      ((atom seen) (quote ()))
      ((equal? word (car (car seen))) (car seen))
      (t (find-seen word (cdr seen))))))

; State is (reversed-output-rows seen-spellings).
; An output row is (spelling semantic-id namespace).
(def collect-surface
  (lambda (sid raw-surface state)
    (let* ((surface (normalize-surface raw-surface))
           (rows (car state))
           (seen (second state)))
      (cond
        ((not (eq (length surface) 3))
         (fail-closed "meta semantic registry generation failed: malformed surface row"))
        (t
         (let* ((namespace (car surface))
                (word (second surface))
                (status (third surface)))
           (cond
             ((not (admitted-status? status)) state)
             ((excluded-word? word) state)
             ((not (symbol? word))
              (fail-closed "meta semantic registry generation failed: admitted surface is not one symbol"))
             (t
              (let ((previous (find-seen word seen)))
                (cond
                  ((atom previous)
                   (list (cons (list word sid namespace) rows)
                         (cons (list word sid) seen)))
                  ((eq (second previous) sid) state)
                  (t
                   (fail-closed "meta semantic registry generation failed: ambiguous admitted surface"))))))))))))

(def collect-surfaces
  (lambda (sid surfaces state)
    (cond
      ((atom surfaces) state)
      (t
       (collect-surfaces sid (cdr surfaces)
                         (collect-surface sid (car surfaces) state))))))

(def collect-entries
  (lambda (entries state)
    (cond
      ((atom entries) state)
      (t
       (let ((entry (normalize-entry (car entries))))
         (collect-entries
           (cdr entries)
           (collect-surfaces (car entry) (cdr entry) state)))))))

(def str+
  (lambda args
    (reduce (lambda (acc s) (string-append acc s)) "" args)))

(def join-newline-onto
  (lambda (strings acc)
    (cond
      ((atom strings) acc)
      ((equal? acc "") (join-newline-onto (cdr strings) (car strings)))
      (t (join-newline-onto (cdr strings)
                            (str+ acc "\n" (car strings)))))))

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

(def registry-form
  (car (read-all (read-file "lib/surface/semantic-registry.lisp"))))

(cond
  ((not (eq (car registry-form) (quote sr/1)))
   (fail-closed "meta semantic registry generation failed: expected sr/1 registry"))
  (t (quote registry-ok)))

(def collected
  (collect-entries (cdr registry-form) (list (quote ()) (quote ()))))
(def projection-rows (reverse (car collected)))

(cond
  ((atom projection-rows)
   (fail-closed "meta semantic registry generation failed: registry has no admitted runtime surfaces"))
  (t (quote projection-ok)))

(def generated (render-projection projection-rows))
(def output-path "lib/generated/meta-semantic-registry.lisp")
(def mode (cond ((atom *argv*) "generate") (t (car *argv*))))

(cond
  ((equal? mode "--check")
   (let ((current (read-file output-path)))
     (cond
       ((equal? current generated)
        (print "meta semantic registry projection is current"))
       (t
        (fail-closed "meta semantic registry projection is stale")))))
  (t
   (cons (write-file output-path generated)
         (print "meta semantic registry projection written"))))
