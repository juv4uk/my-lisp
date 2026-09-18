; scripts/generate-postcore-peer-cache.lisp — #606
;
; Generate the capability-free post-core stable-peer cache embedded in
; lib/core.lisp from the one semantic spelling authority:
;   lib/surface/semantic-registry.lisp
;
; Post-core libraries contribute only numeric semantic IDs + the source binding
; they just defined through (my-postcore-materialize-stable-peers ID SOURCE).
; They never contribute peer spellings. Candidate surfaces are ignored.
;
; The generator writes lib/generated/postcore-stable-peer-projection.lisp.
; CI requires the marked block in core.lisp to byte-match that projection, so
; bare-core/WASM bootstraps still need no filesystem capability at runtime.
;
; Usage from repository root:
;   cargo run -p my-lisp-cli --bin my-lisp -- scripts/generate-postcore-peer-cache.lisp
;
; The source path list below is library topology, not surface authority. Adding
; another post-core library that uses the materialization macro must add its
; source path here; all peer names still come exclusively from the registry.

(def postcore-cache-source-paths
  (quote (
    "lib/time.lisp"
    "lib/process.lisp"
    "lib/tcp.lisp"
    "lib/fs.lisp"
  )))

; Host load owns only fast file transport/parsing. This temporary macro owns
; the interpretation: the registry's sr/1 envelope becomes quoted data.
(defmacro sr/1 rows
  (list (quote quote) rows))

(def postcore-cache-authority-rows
  (load "lib/surface/semantic-registry.lisp"))
(print (quote (postcore-cache-debug authority-loaded)))

(def postcore-cache-member-status
  (lambda (needle items)
    (cond
      ((atom items) (structural-kind empty-list)
       (quote absent))
      ((atom items) (structural-kind pair)
       (cond
         ((eq needle (car items)) (identity-relation same)
          (quote present))
         ((eq needle (car items)) (identity-relation distinct)
          (postcore-cache-member-status needle (cdr items))))))))

(def postcore-cache-find-authority-row
  (lambda (semantic-id rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (quote ()))
      ((atom rows) (structural-kind pair)
       (let ((row (car rows)))
         (cond
           ((= semantic-id (car row)) 1
            row)
           ((= semantic-id (car row)) 0
            (postcore-cache-find-authority-row semantic-id (cdr rows)))))))))

(def postcore-cache-surfaces-with-status-onto
  (lambda (status surfaces acc)
    (cond
      ((atom surfaces) (structural-kind empty-list)
       (reverse acc))
      ((atom surfaces) (structural-kind pair)
       (let* ((surface (car surfaces))
              (word (second surface))
              (surface-status (third surface)))
         (cond
           ((eq surface-status status) (identity-relation same)
            (cond
              ((eq word (quote —)) (identity-relation same)
               (postcore-cache-surfaces-with-status-onto
                 status
                 (cdr surfaces)
                 acc))
              ((eq word (quote —)) (identity-relation distinct)
               (cond
                 ((eq
                    (postcore-cache-member-status word acc)
                    (quote present))
                  (identity-relation same)
                  (postcore-cache-surfaces-with-status-onto
                    status
                    (cdr surfaces)
                    acc))
                 ((eq
                    (postcore-cache-member-status word acc)
                    (quote absent))
                  (identity-relation same)
                  (postcore-cache-surfaces-with-status-onto
                    status
                    (cdr surfaces)
                    (cons word acc)))))))
           ((eq surface-status status) (identity-relation distinct)
            (postcore-cache-surfaces-with-status-onto
              status
              (cdr surfaces)
              acc))))))))

(def postcore-cache-stable-surfaces
  (lambda (semantic-id)
    (let ((row
            (postcore-cache-find-authority-row
              semantic-id
              postcore-cache-authority-rows)))
      (cond
        ((atom row) (structural-kind empty-list)
         (quote ()))
        ((atom row) (structural-kind pair)
         (postcore-cache-surfaces-with-status-onto
           (quote stable)
           (cdr row)
           (quote ())))))))

; Collect numeric declarations through a tiny filesystem manifest. LOAD is the
; fast host mechanism; this Lisp macro owns the declaration shape. The manifest
; is reset on every generator run, so removed declarations cannot survive as
; stale data between local runs.
(def postcore-cache-declaration-manifest
  "/tmp/my-lisp-postcore-cache-declarations.lisp")

(def postcore-cache-manifest-reset
  (write-file postcore-cache-declaration-manifest "(quote ())"))

(defmacro my-postcore-materialize-stable-peers args
  (let* ((semantic-id (car args))
         (source (second args))
         (declaration (list semantic-id source)))
    (list
      (quote write-file)
      postcore-cache-declaration-manifest
      (list
        (quote write-to-string)
        (list
          (quote list)
          (list (quote quote) (quote quote))
          (list
            (quote cons)
            (list (quote quote) declaration)
            (list (quote load) postcore-cache-declaration-manifest)))))))

(def postcore-cache-load-sources
  (lambda (paths)
    (cond
      ((atom paths) (structural-kind empty-list)
       (quote loaded))
      ((atom paths) (structural-kind pair)
       (let ((loaded (load (car paths))))
         (postcore-cache-load-sources (cdr paths)))))))

(def postcore-cache-source-load-status
  (postcore-cache-load-sources postcore-cache-source-paths))

(def postcore-cache-declarations
  (reverse (load postcore-cache-declaration-manifest)))

(print
  (list
    (quote postcore-cache-debug)
    (quote declarations-loaded)
    (length postcore-cache-declarations)))

(def postcore-cache-declarations-valid?
  (lambda (declarations)
    (cond
      ((atom declarations) (structural-kind empty-list)
       (quote valid))
      ((atom declarations) (structural-kind pair)
       (let* ((declaration (car declarations))
              (semantic-id (car declaration))
              (source (second declaration))
              (stable-peers
                (postcore-cache-stable-surfaces semantic-id)))
         (cond
           ((eq
              (postcore-cache-member-status source stable-peers)
              (quote present))
            (identity-relation same)
            (postcore-cache-declarations-valid? (cdr declarations)))
           ((eq
              (postcore-cache-member-status source stable-peers)
              (quote absent))
            (identity-relation same)
            (quote invalid))))))))

(def postcore-cache-expected-groups-onto
  (lambda (declarations acc)
    (cond
      ((atom declarations) (structural-kind empty-list)
       (reverse acc))
      ((atom declarations) (structural-kind pair)
       (let* ((semantic-id (car (car declarations)))
              (stable-peers
                (postcore-cache-stable-surfaces semantic-id)))
         (cond
           ((> (length stable-peers) 1) 1
            (postcore-cache-expected-groups-onto
              (cdr declarations)
              (cons (cons semantic-id stable-peers) acc)))
           ((> (length stable-peers) 1) 0
            (postcore-cache-expected-groups-onto
              (cdr declarations)
              acc))))))))

(def postcore-cache-expected-groups
  (postcore-cache-expected-groups-onto
    postcore-cache-declarations
    (quote ())))
(print (quote (postcore-cache-debug groups-derived)))

(def postcore-cache-render-peers-onto
  (lambda (peers acc)
    (cond
      ((atom peers) (structural-kind empty-list)
       acc)
      ((atom peers) (structural-kind pair)
       (postcore-cache-render-peers-onto
         (cdr peers)
         (string-append
           acc
           (string-append " " (write-to-string (car peers)))))))))

(def postcore-cache-render-group
  (lambda (group)
    (string-append
      "    ("
      (string-append
        (number->string (car group))
        (string-append
          (postcore-cache-render-peers-onto (cdr group) "")
          ")")))))

(def postcore-cache-render-groups-onto
  (lambda (groups acc)
    (cond
      ((atom groups) (structural-kind empty-list)
       acc)
      ((atom groups) (structural-kind pair)
       (postcore-cache-render-groups-onto
         (cdr groups)
         (string-append
           acc
           (string-append
             (postcore-cache-render-group (car groups))
             "\n")))))))

(def postcore-cache-render-definition
  (lambda (groups)
    (string-append
      "(def my-postcore-stable-peer-projection\n  (quote (\n"
      (string-append
        (postcore-cache-render-groups-onto groups "")
        "  )))"))))

(def postcore-cache-output-path
  "lib/generated/postcore-stable-peer-projection.lisp")

(print (quote (postcore-cache-debug render-ready)))

(cond
  ((eq
     (postcore-cache-declarations-valid? postcore-cache-declarations)
     (quote valid))
   (identity-relation same)
   (let ((output
           (string-append
             (postcore-cache-render-definition
               postcore-cache-expected-groups)
             "\n")))
     (let ((written
             (write-file postcore-cache-output-path output)))
       (print
         (list
           (quote postcore-peer-cache-generator)
           (list (quote groups) (length postcore-cache-expected-groups))
           (list (quote write-result) written)
           (quote (status written)))))))
  ((eq
     (postcore-cache-declarations-valid? postcore-cache-declarations)
     (quote invalid))
   (identity-relation same)
   (postcore-cache-generator-invalid-source-declaration)))
