; File text policy owned by Lisp.
; Політика текстового читання/запису файлів, якою володіє Lisp.
;
; Host capabilities `read-file-bytes` and `write-file-bytes` expose only raw
; byte mechanism (with allowlist enforcement already applied at that
; boundary); this layer owns UTF-8 decoding/encoding and the public meanings
; of `read-file` and `write-file`, mirroring the split already established
; for `process-run` (lib/process.lisp) and `tcp-read`/`tcp-write` (lib/tcp.lisp).
; Load `lib/utf8.lisp` before this file.

; `read-file-bytes` materializes std::fs::read's Vec<u8> as a proper Lisp list
; of exact integers 0..255. That mechanism fact is already stronger than the
; generic arbitrary-list precondition of `utf8-decode`, so `read-file` enters
; the existing Lisp-owned UTF-8 sequence decoder directly instead of proving
; the byte domain a second time. UTF-8 meaning and rejection policy remain in
; Lisp; only the redundant provenance check is skipped on this bounded path.
(def read-file
  (lambda (path)
    (let ((decoded
            (utf8-decode-onto
              (read-file-bytes path)
              (quote ()))))
      (cond
        ((eq (car decoded) (quote decoded))
         (unicode-scalars->string (second decoded)))
        (t decoded)))))

; Historical `write-file` returns the text it was given after a successful
; write; encoding policy belongs to Lisp, the host only persists bytes.
(def write-file
  (lambda (path text)
    (second (list (write-file-bytes path (utf8-encode-string text)) text))))
