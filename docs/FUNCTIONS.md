# FUNCTION REFERENCE — my-lisp v0.27

**Згенеровано:** 2026-08-22, агент Сакші (ox-alpha).
**Всього:** 31 builtin'ів + 424 бібліотечних функцій.

> ⚠️ Згенеровано статичним аналізом сорсів. Після contract 2.1 це можна робити зсередини REPL через `(env)`.

## 1. Builtin'и ядра

`+`, `-`, `/`, `<`, `=`, `>`, `atom`, `car`, `cdr`, `cond`, `cons`, `def`, `defmacro`, `eq`, `eval`, `json-parse`, `lambda`, `princ`, `print`, `quote`, `read`, `read-all`, `sha256-hex`, `string->symbol`, `string-append`, `string-first`, `string-rest`, `string<?`, `string?`, `symbol->string`, `write-to-string`

## 2. Бібліотека lib/

### clips-import.my (40)
`clips-all-asserts?`, `clips-assert-conclusions`, `clips-assert-form?`, `clips-clauses-for-conclusions`, `clips-cons-each-onto`, `clips-convert-template`, `clips-convert-template-list`, `clips-convert-vars`, `clips-deffacts->clauses`, `clips-defrule->clauses`, `clips-deftemplate-form?`, `clips-deftemplate-name`, `clips-deftemplate-slots`, `clips-drop-printouts`, `clips-fact-clause`, `clips-facts->clauses`, `clips-facts->clauses-onto`, `clips-form->clauses`, `clips-import`, `clips-import-file`, `clips-import-forms`, `clips-import-forms-onto`, `clips-lookup-slot-value`, `clips-positional-args`, `clips-printout-form?`, `clips-rule-preamble-form?`, `clips-slot-name`, `clips-slot-names`, `clips-slot-value-of`, `clips-split-at-arrow`, `clips-string-after-last-double-colon`, `clips-string-empty?`, `clips-string-starts-with-double-colon?`, `clips-strip-module-prefix`, `clips-strip-rule-preamble`, `clips-symbol-starts-with-?`, `clips-template-slot-order`, `clips-templates-from-forms`, `clips-var-term`, `clips-var?`

### content-store.my (6)
`content-store-contains?`, `content-store-get`, `content-store-put`, `content-store-put-world`, `content-store-size`, `empty-content-store`

### core.my (50)
`<=`, `>=`, `abs`, `append`, `assoc`, `caar`, `cadddr`, `cadr`, `cddr`, `digit->string`, `equal?`, `fifth`, `filter`, `filter-onto`, `fourth`, `identity`, `isqrt`, `isqrt-step`, `largest-chunk`, `length`, `length-onto`, `list`, `map`, `map-onto`, `max`, `max-pair`, `member?`, `min`, `min-pair`, `mod`, `nondecreasing-from?`, `nonincreasing-from?`, `not`, `nth`, `number->string`, `number->string-onto`, `pair`, `quotient`, `reduce`, `reverse`, `reverse-onto`, `second`, `sqrt`, `sqrt-iter`, `string-contains?`, `string-empty?`, `string-length`, `string-prefix?`, `symbol?`, `third`

### epistemic.my (25)
`claim-review`, `claim-statement`, `claim?`, `epistemic--all-required-present?`, `epistemic--claim-ref?`, `evidence-claim-ref`, `evidence-method`, `evidence-outcome`, `evidence-source-ref`, `evidence-supports?`, `evidence?`, `intent-capabilities-satisfied?`, `intent-goal`, `intent-produces`, `intent-requires`, `intent-stop-on`, `intent?`, `make-claim`, `make-evidence`, `make-intent`, `make-observation`, `observation-source`, `observation-statement`, `observation?`, `source-ref?`

### forward.my (73)
`*jtms-memory*`, `*justified-memory*`, `*working-memory*`, `add-justification`, `add-new-entries-jtms`, `add-new-justified`, `append-new`, `axiom`, `condition-is-and?`, `condition-is-exists?`, `condition-is-forall?`, `condition-is-not?`, `condition-is-or?`, `condition-is-test?`, `dependents-of`, `drop-unsupported`, `fact-of`, `find-entry`, `fire-rule`, `fire-rule-jtms`, `fire-rule-jtms-multi`, `fire-rule-multi`, `fire-rule-on-facts`, `fire-rule-on-facts-jtms`, `fire-rule-on-facts-tms`, `fire-rule-on-working-memory`, `fire-rule-tms`, `fire-rules-jtms-multi`, `fire-rules-multi`, `fire-rules-on-facts`, `fire-rules-on-facts-jtms`, `fire-rules-on-facts-tms`, `fire-rules-on-working-memory`, `forall-every-candidate-satisfies?`, `jtms-make-state`, `jtms-state-subst`, `jtms-state-used`, `justifications-of`, `justified-member?`, `make-justified`, `map-apply-head`, `map-apply-head-jtms`, `map-fact-of`, `match-and-condition`, `match-condition-against-facts`, `match-conditions`, `match-conditions-jtms`, `match-exists-condition`, `match-forall-condition`, `match-negated-condition`, `match-one-condition`, `match-one-condition-jtms`, `match-or-condition`, `match-or-condition-jtms`, `match-plain-condition-jtms`, `match-test-condition`, `prune-all-entries`, `prune-entry`, `prune-justifications`, `remove-entry-jtms`, `remove-justified`, `retract-fact`, `retract-fact-jtms`, `retract-fact-tms`, `retract-facts-jtms`, `retract-facts-tms`, `run`, `run-jtms`, `run-jtms-multi`, `run-multi`, `run-tms`, `supports-of`, `unsupported-facts`

### knowledge.my (39)
`*knowledge-journal*`, `*knowledge-package-version*`, `*usage-counts*`, `advice-all-decision`, `advice-batch-conflict`, `advice-conflict-proof`, `advice-decision`, `advice-negative-head-conflict`, `apply-journal-event`, `check-conflict`, `clauses->tell-events`, `collect-facts-about`, `contains-atom?`, `describe`, `exchange-knowledge-package`, `forward-in`, `is-fact?`, `knowledge-clause-valid?`, `knowledge-clauses-valid?`, `knowledge-goal-valid?`, `knowledge-goals-valid?`, `knowledge-package-decision`, `knowledge-package-entries-valid?`, `knowledge-package-field`, `knowledge-proper-list?`, `knowledge-term-valid?`, `knowledge-terms-valid?`, `make-knowledge-package`, `module-clauses-now`, `module-journal-events`, `module-known?`, `opposite-knowledge-head`, `reason-in`, `send-knowledge-package`, `string-through-line`, `tcp-read-frame`, `tcp-read-to-eof`, `usage-of`, `write-knowledge-package`

### linter.my (13)
`collect-free-vars`, `collect-free-vars-let*`, `collect-free-vars-letrec`, `effectful-primitives`, `get-threshold`, `lint-all`, `lint-check`, `lint-complexity`, `lint-effects`, `lint-globals`, `lint-nesting`, `lint-size`, `max`

### meta-eval.my (10)
`bind-params`, `env-lookup`, `my-apply`, `my-eval`, `my-eval-body`, `my-eval-cond`, `my-eval-list`, `my-eval-program`, `my-eval-top-form`, `my-macro?`

### narrate.my (8)
`narrate-answer`, `narrate-derivation`, `narrate-fact`, `narrate-provenance`, `provenance-derived-from`, `provenance-goal`, `provenance-rule`, `provenance-source`

### persistent-map.my (17)
`balance`, `balance-factor`, `height-of`, `make-balanced-node`, `map->list`, `map-contains?`, `map-empty`, `map-get`, `map-insert`, `max2`, `node-height`, `node-key`, `node-left`, `node-right`, `node-value`, `rotate-left`, `rotate-right`

### reason.my (20)
`add-usage`, `count-usage`, `count-usage-list`, `explain-proof`, `explain-proof-list`, `explain-proof-node`, `map-goal-results`, `map-proofs`, `merge-usage`, `print-indent`, `prove-goal`, `prove-goal-state`, `prove-goals`, `prove-rule`, `provenance`, `provenance-list`, `reason`, `reason-explain`, `rename-vars`, `source-of`

### result-status.my (7)
`make-blocked`, `make-disputed`, `make-partial`, `make-unknown`, `result-payload`, `result-status`, `result-tagged?`

### understand.my (5)
`strip-article`, `understand`, `understand-is`, `understand-relation`, `understand-universal`

### unify.my (15)
`apply-subst`, `apply-subst-walked`, `extend-subst`, `failed-subst?`, `logic-var`, `lookup-subst`, `occurs-check`, `thread-conjunction`, `thread-conjunction-branches`, `unify`, `unify-var`, `unify-walked`, `var?`, `walk`, `walk-resolved`

### world.my (38)
`advice-all-decision-in-world`, `advice-decision-in-world`, `advise-all-world`, `advise-world`, `empty-world`, `forward-in-world`, `import-knowledge-package-world`, `knowledge-content-address`, `legacy-world-transition-expansion`, `make-world`, `make-world-knowledge-package`, `reason-in-world`, `world-address-content`, `world-apply-event`, `world-at-depth`, `world-at-depth-from`, `world-branch-diff`, `world-clauses`, `world-climb-to-depth`, `world-common-ancestor`, `world-common-ancestor-aligned`, `world-content-address`, `world-depth`, `world-diff`, `world-journal`, `world-journal-prefix`, `world-metadata`, `world-module-events`, `world-module-known?`, `world-no-common-ancestor?`, `world-not-ancestor?`, `world-parent`, `world-record`, `world-remove-first`, `world-retract`, `world-tell`, `world-tell-all`, `world?`

### yantra.my (58)
`agent-loop`, `alist-ref`, `all-covered?`, `append-tool-results`, `bash-tool-schema`, `build-request-body`, `claim-markers`, `claims-execution?`, `collect-trailing-tools`, `count-with-role`, `dispatch-tool`, `encode-message`, `encode-tool-call`, `ends-with-owned-tool-results?`, `execute-bash`, `execute-tool-call`, `extract-assistant-message`, `has-tool-result?`, `http-post-json`, `id-in-list?`, `invalid-completion-nudge`, `json->message`, `json->tool-call`, `json-encode`, `json-encode-array`, `json-encode-array-items`, `json-encode-key`, `json-encode-object`, `json-encode-object-entries`, `json-encode-string`, `json-encode-value`, `json-escape`, `json-escape-char`, `json-escape-onto`, `json-message-content`, `json-object?`, `markers-contained?`, `max-turns`, `msg-call-ids`, `msg-content`, `msg-role`, `msg-tool-call-id`, `msg-tool-calls`, `number->string-nonneg`, `ollama-complete`, `ollama-model`, `ollama-url`, `result-answer`, `result-messages`, `result-status`, `result-turn`, `run-agent`, `strcat`, `strcat-onto`, `tc-arguments`, `tc-id`, `tc-name`, `valid-final?`
