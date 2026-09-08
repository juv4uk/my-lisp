; CANON 0+7 — executable semantic contract written in my-lisp.
; CANON 0+7 — виконуваний семантичний контракт, написаний самою my-lisp.
;
; Authority rule:
;   the language states the laws; host runtimes implement mechanisms and
;   must conform to these laws. This file is semantic evidence over an already
;   existing immutable Canon; it must never create or rebind Canon spellings.
;
; Contract 6.0 boundary:
;   (), quote/як-є/svarūpa, atom/атом?/aṇu, eq/тотожне?/abheda,
;   cons/сполучити/saṃyuj, car/перше/ādi, cdr/решта/śeṣa and
;   cond/за-умовою/anukrama are supplied by the immutable Canon resolver.
;   This file only names witnesses and laws above that foundation.

; Canon 0 has no lexical alias as part of Canon itself. This ordinary witness
; is intentionally outside the reserved set and is used only by the laws below.
(def canon-empty-list (quote ()))

; QUOTE and COND are evaluation-control forms, not ordinary first-class values.
; Callable Canon primitives are first-class immutable operation handles.
;
; Keyboard-symbol surface (typed without leaving the Ukrainian layout):
;   '  .?  =?  :  :п  :р  ?:
; These are spellings of the same 0+7 identities, never additional primitives.

; --- Constitutive laws ----------------------------------------------------
; Each function returns t exactly when the runtime satisfies the stated law
; for the supplied witnesses. These are executable semantics, not examples.

(def canon-law-empty-list
  (lambda ()
    (тотожне? canon-empty-list (quote ()))))

(def canon-law-atom-cons
  (lambda (x y)
    (тотожне? (атом? (сполучити x y)) (quote ()))))

(def canon-law-car-cons
  (lambda (x y)
    (тотожне? (перше (сполучити x y)) x)))

(def canon-law-cdr-cons
  (lambda (x y)
    (тотожне? (решта (сполучити x y)) y)))

(def canon-law-eq-reflexive-atom
  (lambda (x)
    (за-умовою
      ((атом? x) (тотожне? x x))
      (t (quote ())))))

; `решта` must be a pair projection, not a human-language "second element".
(def canon-law-cdr-dotted
  (lambda ()
    (тотожне?
      (решта (сполучити (quote кіт) 42))
      42)))

; EQ is atom-only. Therefore the proper-list witness is checked by projecting
; the returned pair chain into atoms instead of comparing two lists structurally.
(def canon-law-cdr-proper
  (lambda ()
    (за-умовою
      ((тотожне? (перше (решта (як-є (1 2 3)))) 2)
       (за-умовою
         ((тотожне? (перше (решта (решта (як-є (1 2 3))))) 3)
          (тотожне? (решта (решта (решта (як-є (1 2 3))))) (як-є ())))
         (t (як-є ()))))
      (t (як-є ())))))

; EQ is atom-only, so structural equality is not used for the improper result;
; project it back into atom witnesses instead.
(def canon-law-cdr-improper
  (lambda ()
    (за-умовою
      ((тотожне? (перше (решта (як-є (1 2 . 3)))) 2)
       (тотожне? (решта (решта (як-є (1 2 . 3)))) 3))
      (t (quote ())))))

; Evaluation-control laws: unselected/quoted unknown symbols must never be
; evaluated. If a host eagerly evaluates them, these functions fail before
; returning t.
(def canon-law-quote-suppresses-evaluation
  (lambda ()
    (тотожне? (як-є never-defined-canon-symbol)
              (quote never-defined-canon-symbol))))

(def canon-law-cond-first-true-short-circuit
  (lambda ()
    (тотожне?
      (за-умовою
        (t (як-є selected))
        ((never-defined-canon-predicate) (як-є forbidden)))
      (quote selected))))

; The compact Ukrainian-keyboard surface must denote the same seven
; operations. This law deliberately exercises every symbolic spelling in one
; vertical slice: quote, atom, eq, cons, car, cdr, and cond.
(def canon-law-symbolic-surface
  (lambda ()
    (?:
      ((=? (:п (: 'ліве 'праве)) 'ліве)
       (?:
         ((=? (:р (: 'ліве 'праве)) 'праве)
          (.? 'атом))
         (t '())))
      (t '()))))

; One language-level verdict used by every runtime conformance test.
(def canon-conforms?
  (lambda ()
    (за-умовою
      ((canon-law-empty-list)
       (за-умовою
         ((canon-law-atom-cons (quote x) (quote y))
          (за-умовою
            ((canon-law-car-cons (quote x) (quote y))
             (за-умовою
               ((canon-law-cdr-cons (quote x) (quote y))
                (за-умовою
                  ((canon-law-eq-reflexive-atom (quote x))
                   (за-умовою
                     ((canon-law-cdr-dotted)
                      (за-умовою
                        ((canon-law-cdr-proper)
                         (за-умовою
                           ((canon-law-cdr-improper)
                            (за-умовою
                              ((canon-law-quote-suppresses-evaluation)
                               (за-умовою
                                 ((canon-law-cond-first-true-short-circuit)
                                  (canon-law-symbolic-surface))
                                 (t (quote ()))))
                              (t (quote ()))))
                           (t (quote ()))))
                        (t (quote ()))))
                     (t (quote ()))))
                  (t (quote ()))))
               (t (quote ()))))
            (t (quote ()))))
         (t (quote ()))))
      (t (quote ())))))
