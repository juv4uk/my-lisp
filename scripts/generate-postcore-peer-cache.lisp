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
; The generated block lives between explicit BEGIN/END markers in core.lisp so
; bare-core/WASM bootstraps need no filesystem capability at runtime.
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

(def postcore-cache-authority-form
  (car (read-all (read-file "lib/surface/semantic-registry.lisp"))))

(def postcore-cache-authority-rows (cdr postcore-cache-authority-form))

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

(def postcore-cache-materialization-declarations-onto
  (lambda (forms acc)
    (cond
      ((atom forms) (structural-kind empty-list)
       (reverse acc))
      ((atom forms) (structural-kind pair)
       (let ((form (car forms)))
         (cond
           ((atom form) (structural-kind pair)
            (cond
              ((eq
                 (car form)
                 (quote my-postcore-materialize-stable-peers))
               (identity-relation same)
               (postcore-cache-materialization-declarations-onto
                 (cdr forms)
                 (cons (list (second form) (third form)) acc)))
              ((eq
                 (car form)
                 (quote my-postcore-materialize-stable-peers))
               (identity-relation distinct)
               (postcore-cache-materialization-declarations-onto
                 (cdr forms)
                 acc))))
           ((atom form) (structural-kind atom)
            (postcore-cache-materialization-declarations-onto
              (cdr forms)
              acc))
           ((atom form) (structural-kind empty-list)
            (postcore-cache-materialization-declarations-onto
              (cdr forms)
              acc))))))))

(def postcore-cache-materialization-declarations
  (lambda (path)
    (postcore-cache-materialization-declarations-onto
      (read-all (read-file path))
      (quote ()))))

(def postcore-cache-collect-declarations
  (lambda (paths acc)
    (cond
      ((atom paths) (structural-kind empty-list)
       acc)
      ((atom paths) (structural-kind pair)
       (postcore-cache-collect-declarations
         (cdr paths)
         (append
           acc
           (postcore-cache-materialization-declarations (car paths))))))))

(def postcore-cache-declarations
  (postcore-cache-collect-declarations
    postcore-cache-source-paths
    (quote ())))

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

(def postcore-cache-begin-marker
  "; BEGIN GENERATED POSTCORE STABLE PEER PROJECTION — scripts/generate-postcore-peer-cache.lisp")

(def postcore-cache-end-marker
  "; END GENERATED POSTCORE STABLE PEER PROJECTION")

(def postcore-cache-find-marker-from
  (lambda (text marker index)
    (let ((end (+ index (string-length marker))))
      (cond
        ((> end (string-length text)) 1
         -1)
        ((> end (string-length text)) 0
         (let ((candidate (string-slice text index end)))
           (cond
             ((equal? candidate marker) (structural-relation same)
              index)
             ((equal? candidate marker) (structural-relation distinct)
              (postcore-cache-find-marker-from
                text
                marker
                (+ index 1))))))))))

(def postcore-cache-rewrite-core
  (lambda (core-text)
    (let ((begin
            (postcore-cache-find-marker-from
              core-text
              postcore-cache-begin-marker
              0)))
      (cond
        ((= begin -1) 1
         (postcore-cache-generator-missing-begin-marker))
        ((= begin -1) 0
         (let* ((body-start
                  (+ begin (string-length postcore-cache-begin-marker)))
                (end
                  (postcore-cache-find-marker-from
                    core-text
                    postcore-cache-end-marker
                    body-start)))
           (cond
             ((= end -1) 1
              (postcore-cache-generator-missing-end-marker))
             ((= end -1) 0
              (string-append
                (string-slice core-text 0 body-start)
                (string-append
                  "\n"
                  (string-append
                    (postcore-cache-render-definition
                      postcore-cache-expected-groups)
                    (string-append
                      "\n"
                      (string-slice
                        core-text
                        end
                        (string-length core-text))))))))))))))

(cond
  ((eq
     (postcore-cache-declarations-valid? postcore-cache-declarations)
     (quote valid))
   (identity-relation same)
   (let* ((core-path "lib/core.lisp")
          (core-text (read-file core-path))
          (rewritten (postcore-cache-rewrite-core core-text)))
     (write-file core-path rewritten)
     (print
       (list
         (quote postcore-peer-cache-generator)
         (list (quote groups) (length postcore-cache-expected-groups))
         (quote (status written))))))
  ((eq
     (postcore-cache-declarations-valid? postcore-cache-declarations)
     (quote invalid))
   (identity-relation same)
   (postcore-cache-generator-invalid-source-declaration)))
