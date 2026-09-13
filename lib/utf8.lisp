; UTF-8 encoding and decoding semantics owned by Lisp.
; Семантика кодування й декодування UTF-8, якою володіє Lisp.
;
; Decoder input is a proper list of exact byte integers 0..255 and returns:
;   (decoded (codepoint ...))
; or
;   (rejected invalid-byte)
;   (rejected invalid-utf8)
;
; Encoder input is a runtime String and returns its exact UTF-8 bytes as a
; proper list of integers 0..255. Unicode sequence validation and both
; byte<->scalar interpretations stay here in Lisp. The runtime exposes only
; the minimal scalar/string bridges `codepoint->string` and
; `string->codepoint`; neither bridge knows UTF-8.

(def utf8-continuation-byte?
  (lambda (b)
    (and (>= b 128) (<= b 191))))

(def utf8-byte?
  (lambda (b)
    (and (= b b)
         (>= b 0)
         (<= b 255)
         (= (mod b 1) 0))))

(def utf8-all-bytes?
  (lambda (bytes)
    (cond
      ((atom bytes) (eq bytes (quote ())))
      ((not (utf8-byte? (car bytes))) ())
      (t (utf8-all-bytes? (cdr bytes))))))

(def utf8-decode-onto
  (lambda (bytes out)
    (cond
      ((atom bytes) (list (quote decoded) (reverse out)))
      (t
       (let* ((b1 (car bytes))
              (r1 (cdr bytes)))
         (cond
           ; ASCII
           ((<= b1 127)
            (utf8-decode-onto r1 (cons b1 out)))

           ; 2-byte sequence: C2..DF 80..BF
           ((and (>= b1 194) (<= b1 223))
            (cond
              ((atom r1) (list (quote rejected) (quote invalid-utf8)))
              (t
               (let ((b2 (car r1)))
                 (cond
                   ((not (utf8-continuation-byte? b2))
                    (list (quote rejected) (quote invalid-utf8)))
                   (t
                    (utf8-decode-onto
                      (cdr r1)
                      (cons (+ (* (- b1 192) 64)
                               (- b2 128))
                            out))))))))

           ; 3-byte sequence with overlong/surrogate exclusions.
           ((and (>= b1 224) (<= b1 239))
            (cond
              ((or (atom r1) (atom (cdr r1)))
               (list (quote rejected) (quote invalid-utf8)))
              (t
               (let* ((b2 (car r1))
                      (r2 (cdr r1))
                      (b3 (car r2))
                      (second-ok
                        (cond
                          ((= b1 224) (and (>= b2 160) (<= b2 191)))
                          ((= b1 237) (and (>= b2 128) (<= b2 159)))
                          (t (utf8-continuation-byte? b2)))))
                 (cond
                   ((not (and second-ok (utf8-continuation-byte? b3)))
                    (list (quote rejected) (quote invalid-utf8)))
                   (t
                    (utf8-decode-onto
                      (cdr r2)
                      (cons (+ (* (- b1 224) 4096)
                               (* (- b2 128) 64)
                               (- b3 128))
                            out))))))))

           ; 4-byte sequence, restricted to Unicode scalar range <= 10FFFF.
           ((and (>= b1 240) (<= b1 244))
            (cond
              ((or (atom r1) (atom (cdr r1)) (atom (cdr (cdr r1))))
               (list (quote rejected) (quote invalid-utf8)))
              (t
               (let* ((b2 (car r1))
                      (r2 (cdr r1))
                      (b3 (car r2))
                      (r3 (cdr r2))
                      (b4 (car r3))
                      (second-ok
                        (cond
                          ((= b1 240) (and (>= b2 144) (<= b2 191)))
                          ((= b1 244) (and (>= b2 128) (<= b2 143)))
                          (t (utf8-continuation-byte? b2)))))
                 (cond
                   ((not (and second-ok
                              (utf8-continuation-byte? b3)
                              (utf8-continuation-byte? b4)))
                    (list (quote rejected) (quote invalid-utf8)))
                   (t
                    (utf8-decode-onto
                      (cdr r3)
                      (cons (+ (* (- b1 240) 262144)
                               (* (- b2 128) 4096)
                               (* (- b3 128) 64)
                               (- b4 128))
                            out))))))))

           (t (list (quote rejected) (quote invalid-utf8)))))))))

(def utf8-decode
  (lambda (bytes)
    (cond
      ((not (utf8-all-bytes? bytes))
       (list (quote rejected) (quote invalid-byte)))
      (t (utf8-decode-onto bytes (quote ()))))))

(def utf8-valid?
  (lambda (bytes)
    (eq (car (utf8-decode bytes)) (quote decoded))))

; Keep text materialization in tail position. The previous direct recursion
; accumulated one host evaluator frame per Unicode scalar while waiting to
; string-append on unwind; ordinary network payloads could therefore exhaust
; the smaller stacks of worker threads even though byte validation itself was
; tail-recursive.
(def unicode-scalars->string-onto
  (lambda (scalars out)
    (cond
      ((atom scalars) out)
      (t
       (unicode-scalars->string-onto
         (cdr scalars)
         (string-append out (codepoint->string (car scalars))))))))

(def unicode-scalars->string
  (lambda (scalars)
    (unicode-scalars->string-onto scalars "")))

(def utf8-decode-string
  (lambda (bytes)
    (let ((decoded (utf8-decode bytes)))
      (cond
        ((eq (car decoded) (quote decoded))
         (list (quote decoded)
               (unicode-scalars->string (car (cdr decoded)))))
        (t decoded)))))

; Encode one runtime String by observing one Unicode scalar at a time. The
; accumulator is kept in reverse byte order so the recursive call remains in
; tail position and one final reverse restores wire order.
(def utf8-encode-string-onto
  (lambda (text out)
    (cond
      ((string-empty? text) (reverse out))
      (t
       (let* ((character (string-first text))
              (rest (string-rest text))
              (scalar (string->codepoint character)))
         (cond
           ((<= scalar 127)
            (utf8-encode-string-onto rest (cons scalar out)))
           ((<= scalar 2047)
            (utf8-encode-string-onto
              rest
              (cons (+ 128 (mod scalar 64))
                    (cons (+ 192 (quotient scalar 64)) out))))
           ((<= scalar 65535)
            (utf8-encode-string-onto
              rest
              (cons (+ 128 (mod scalar 64))
                    (cons (+ 128 (mod (quotient scalar 64) 64))
                          (cons (+ 224 (quotient scalar 4096)) out)))))
           (t
            (utf8-encode-string-onto
              rest
              (cons (+ 128 (mod scalar 64))
                    (cons (+ 128 (mod (quotient scalar 64) 64))
                          (cons (+ 128 (mod (quotient scalar 4096) 64))
                                (cons (+ 240 (quotient scalar 262144)) out))))))))))))

(def utf8-encode-string
  (lambda (text)
    (utf8-encode-string-onto text (quote ()))))
