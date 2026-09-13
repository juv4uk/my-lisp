;; Peer-identity acceptance program.
;; Приймальна програма тотожності мовних поверхонь.
;;
;; Proves that a Ukrainian peer spelling and its English counterpart
;; share exact language-level identity under `eq`, not merely equal
;; behavior through a re-implementation or translation shim -- the
;; distinction docs/PLAN-FULL-LANGUAGE-PARITY.md's invariant 5 draws
;; ("Жодного cross-language alias як реалізації"). The runtime
;; representation used to realize that identity is deliberately not part
;; of this language contract.
;;
;; This is a different concern from tests/fixtures/conformance.lisp's
;; single-session semantic facts: it inherently compares bindings across
;; two loaded surface files, which conformance.lisp's uniform core.lisp-only
;; execution model does not accommodate.
;;
;; Required: core.lisp, then uk.lisp loaded on top (same as the CLI's own
;; --surface=uk prerequisite chain).
;; Run: load core library + uk.lisp, then evaluate this file.
;;
;; Added 2026-09-11 alongside the abs/min/max/min-list/max-list
;; migration (docs/VERTICAL-SLICE-1-ABS-MIN-MAX-2026-09-11.md) as the
;; first entries; extend this file, not conformance.lisp, for future
;; peer-identity witnesses.

(визначити перевірити-все
  (функція ()
    (за-умовою
      ((хибне? (тотожне? модуль abs)) (як-є помилка-модуль-abs))
      ((хибне? (тотожне? найменше min)) (як-є помилка-найменше-min))
      ((хибне? (тотожне? найбільше max)) (як-є помилка-найбільше-max))
      ((хибне? (тотожне? найменше-у-списку min-list)) (як-є помилка-найменше-у-списку-min-list))
      ((хибне? (тотожне? найбільше-у-списку max-list)) (як-є помилка-найбільше-у-списку-max-list))
      ((тотожне? 1 1) (як-є успіх)))))

;; Run
(перевірити-все)
