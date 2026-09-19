; Reality ledger experiment.
;
; Purpose: prevent a language label from silently promoting itself into a fact
; about reality.  This file does NOT claim that "atom" is false or unreal.
; It records only what the current apparatus actually returns, what vocabulary
; we introduced to describe it, and what the observation does not establish.
;
; Research bookkeeping words below are ordinary Lisp data, not new runtime
; semantic categories.

(def reality-entry
  (lambda (status subject evidence)
    (list status subject evidence)))

(def observed
  (lambda (subject evidence)
    (reality-entry (quote observed) subject evidence)))

(def introduced
  (lambda (subject evidence)
    (reality-entry (quote introduced) subject evidence)))

(def derived
  (lambda (subject evidence)
    (reality-entry (quote derived) subject evidence)))

(def not-claimed
  (lambda (subject evidence)
    (reality-entry (quote not-claimed) subject evidence)))

; Keep the apparatus visible.  Calling ATOM is itself an introduced choice of
; instrument; its output cannot independently prove the ontology encoded in its
; own result labels.
(def atom-apparatus-observations
  (lambda ()
    (list
      (observed
        (quote symbol-sample)
        (list (quote input) (quote radio)
              (quote raw-result) (atom (quote radio))))
      (observed
        (quote empty-list-sample)
        (list (quote input) (quote ())
              (quote raw-result) (atom (quote ()))))
      (observed
        (quote pair-sample)
        (list (quote input) (quote (radio antenna))
              (quote raw-result) (atom (quote (radio antenna))))))))

; Independent structural probes available in the current language apparatus.
; They do not classify the whole value domain.  They merely demonstrate that
; the pair sample can be projected and reconstructed through CAR/CDR/CONS.
(def pair-projection-observation
  (lambda ()
    (let ((sample (quote (radio . antenna))))
      (list
        (observed
          (quote car-projection)
          (list (quote input) sample
                (quote output) (car sample)))
        (observed
          (quote cdr-projection)
          (list (quote input) sample
                (quote output) (cdr sample)))
        (derived
          (quote reconstruction)
          (list
            (quote observed-value)
            (cons (car sample) (cdr sample))
            (quote comparison)
            (equal? sample (cons (car sample) (cdr sample)))))))))

; A callable observation deliberately avoids asserting the noun "function".
; We only observe application behavior for a concrete value.
(def callable-application-observation
  (lambda ()
    (let ((candidate (lambda (x) x)))
      (observed
        (quote application)
        (list
          (quote argument) (quote probe)
          (quote output) (candidate (quote probe)))))))

(def atom-reality-ledger
  (lambda ()
    (list
      (introduced
        (quote apparatus)
        (quote atom-car-cdr-cons-equal))
      (introduced
        (quote vocabulary)
        (quote (structural-kind atom pair empty-list)))
      (list
        (quote observations)
        (atom-apparatus-observations))
      (list
        (quote independent-pair-probes)
        (pair-projection-observation))
      (list
        (quote callable-probe)
        (callable-application-observation))
      (not-claimed
        (quote ontology)
        (quote atom-is-fundamental-kind-of-reality))
      (not-claimed
        (quote ontology)
        (quote every-callable-is-a-fundamental-function-kind))
      (not-claimed
        (quote identity)
        (quote result-label-proves-its-own-semantic-truth)))))

(atom-reality-ledger)
