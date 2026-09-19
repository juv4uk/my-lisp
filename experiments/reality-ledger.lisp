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


; ---------------------------------------------------------------------------
; Surface / "semantic identity" reality ledger.
;
; Goal: distinguish what is actually observed from the stronger interpretation
; "these spellings ARE one semantic identity".
;
; For Canon CAR surfaces, the current apparatus gives us several independent
; observations:
;   - registry row groups spellings under one opaque BitPattern8;
;   - evaluating the spellings yields behavior compatible with CAR;
;   - the host currently materializes one SemanticRef for those spellings.
;
; The first two can be probed from Lisp. The third is host-representation
; evidence and must not silently become an ontological theorem.

(def surface-behavior-observation
  (lambda (surface-call expected)
    (observed
      surface-call
      (list
        (quote expected) expected))))

; Execute three human/symbolic CAR surfaces over the same concrete sample.
; We observe same output. We do NOT conclude that the names are literally one
; thing in every possible implementation.
(def car-surface-behavior-ledger
  (lambda ()
    (let ((sample (quote (radio antenna))))
      (list
        (observed
          (quote en-car)
          (list (quote output) (car sample)))
        (observed
          (quote uk-car)
          (list (quote output) (перше sample)))
        (observed
          (quote sa-car)
          (list (quote output) (ādi sample)))
        (observed
          (quote sym-car)
          (list (quote output) (:п sample)))
        (derived
          (quote sampled-behavioral-agreement)
          (list
            (quote en-vs-uk) (equal? (car sample) (перше sample))
            (quote en-vs-sa) (equal? (car sample) (ādi sample))
            (quote en-vs-sym) (equal? (car sample) (:п sample))))
        (introduced
          (quote registry-grouping)
          (quote (00000101 car перше ādi :п)))
        (not-claimed
          (quote ontology)
          (quote grouped-spellings-are-literally-one-entity))
        (not-claimed
          (quote portability)
          (quote shared-host-representation-is-required-by-reality))))))

; Contrast case: two distinct registry IDs may currently agree behaviorally.
; This guards against the inverse mistake "different ID => necessarily
; different behavior".
(def distinct-id-overlap-ledger
  (lambda ()
    (let ((sample (quote (a b c d))))
      (list
        (introduced
          (quote registry-distinction)
          (quote (00101111 second 00110100 cadr)))
        (observed
          (quote second-output)
          (second sample))
        (observed
          (quote cadr-output)
          (cadr sample))
        (derived
          (quote sampled-behavioral-agreement)
          (equal? (second sample) (cadr sample)))
        (not-claimed
          (quote identity)
          (quote different-registry-id-implies-different-behavior))
        (not-claimed
          (quote identity)
          (quote same-behavior-implies-same-registry-id))))))

(def semantic-identity-reality-ledger
  (lambda ()
    (list
      (list
        (quote grouped-surfaces)
        (car-surface-behavior-ledger))
      (list
        (quote distinct-id-overlap)
        (distinct-id-overlap-ledger))
      (not-claimed
        (quote ontology)
        (quote semantic-identity-is-fundamental-kind-of-reality))
      (not-claimed
        (quote authority)
        (quote registry-row-proves-meaning-by-itself)))))

