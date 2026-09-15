; #150 — one-way authority boundary for machine/backend data.
;
; Lisp semantics may be projected into machine representation.  Machine facts
; may describe CPU capabilities, instruction forms, registers, encodings and
; ABI details, but none of those facts may allocate or rename a language-level
; semantic identity or peer surface.
;
; This file is Lisp-owned policy data.  Host tooling may read it and enforce
; mechanics around it, but the direction and prohibitions are stated here.

(machine-authority-boundary/1
  (semantic-authority lib/surface/semantic-registry.lisp)
  (machine-root lib/machine)
  (public-api-excluded-root lib/machine)
  (machine-public-api-admission explicit-ratification-only)

  (lowering-direction semantic-to-machine)
  (reverse-authority machine-to-semantic forbidden)

  (semantic-id-from-isa forbidden)
  (semantic-id-from-cpu-profile forbidden)
  (peer-surface-from-machine forbidden)

  (diagnostic machine-authority-boundary-violation))
