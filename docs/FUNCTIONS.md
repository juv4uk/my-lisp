# FUNCTION REFERENCE — my-lisp (regenerated)

**Згенеровано:** 2026-09-02 · база `8b4529f` · Claude Sonnet 5 (Ecosystem Lead)
**Джерело:** live `(env)` через `scripts/gen-functions.my` (contract 2.1: builtins живуть у середовищі) + статичний прохід `lib/*.my`, що тепер збирає і `(def NAME ...)`, і `(defmacro NAME ...)` (попередня регенерація захоплювала лише `def`, тому пропускала кожен `defmacro` — `let`/`let*`/`and`/`or`/`->`/`->>` в `core.my`, `defmodule`/`advise`/`tell-knowledge`/... в `knowledge.my`/`world.my`, `assert-fact!`/`run-tms!`/... в `forward.my`).
**Всього:** 32 builtin'ів + 482 бібліотечних функцій у 17 файлах.

> Оновлення: перезапусти `scripts/gen-functions.my` для live-секції; lib-секції — статичні до наступної регенерації.

## 1. Builtin'и ядра (live env)

`*`, `+`, `-`, `/`, `<`, `=`, `>`, `abs`, `atom`, `car`, `cdr`, `cons`, `env`, `eq`, `f32-buffer`, `i32-buffer`, `internet-time-sync`, `make-vector`, `max`, `max-list`, `min`, `min-list`, `mono-ms`, `mono-ns`, `numeric-buffer-length`, `numeric-buffer-map`, `numeric-buffer-ref`, `numeric-buffer-type`, `numeric-buffer?`, `string-slice`, `timezone-detect`, `utc-now`, `vector`, `vector-length`, `vector-ref`, `vector-set!`

> `<`, `=`, `>` return the canonical WSM truth value (`t`/`()`), not a hidden `Value::Bool`, as of `8b4529f` — see [`docs/language-core.md`](language-core.md).

### clips-import.my (40)

`clips-all-asserts?`, `clips-assert-conclusions`, `clips-assert-form?`, `clips-clauses-for-conclusions`, `clips-cons-each-onto`, `clips-convert-template`, `clips-convert-template-list`, `clips-convert-vars`, `clips-deffacts->clauses`, `clips-defrule->clauses`, `clips-deftemplate-form?`, `clips-deftemplate-name`, `clips-deftemplate-slots`, `clips-drop-printouts`, `clips-fact-clause`, `clips-facts->clauses`, `clips-facts->clauses-onto`, `clips-form->clauses`, `clips-import`, `clips-import-file`, `clips-import-forms`, `clips-import-forms-onto`, `clips-lookup-slot-value`, `clips-positional-args`, `clips-printout-form?`, `clips-rule-preamble-form?`, `clips-slot-name`, `clips-slot-names`, `clips-slot-value-of`, `clips-split-at-arrow`, `clips-string-after-last-double-colon`, `clips-string-empty?`, `clips-string-starts-with-double-colon?`, `clips-strip-module-prefix`, `clips-strip-rule-preamble`, `clips-symbol-starts-with-?`, `clips-template-slot-order`, `clips-templates-from-forms`, `clips-var-term`, `clips-var?`

### content-store.my (6)

`content-store-contains?`, `content-store-get`, `content-store-put`, `content-store-put-world`, `content-store-size`, `empty-content-store`

### core.my (52)

`->`, `->>`, `<=`, `>=`, `and`, `append`, `assoc`, `caar`, `cadddr`, `cadr`, `cddr`, `digit->string`, `equal?`, `fifth`, `filter`, `filter-onto`, `fourth`, `gensym`, `identity`, `isqrt`, `isqrt-step`, `largest-chunk`, `length`, `length-onto`, `let`, `let*`, `list`, `map`, `map-onto`, `member?`, `mod`, `nondecreasing-from?`, `nonincreasing-from?`, `not`, `nth`, `number->string`, `number->string-onto`, `or`, `pair`, `quotient`, `reduce`, `reverse`, `reverse-onto`, `second`, `sqrt`, `sqrt-iter`, `string-contains?`, `string-empty?`, `string-length`, `string-prefix?`, `symbol?`, `third`

### epistemic.my (25)

`claim-review`, `claim-statement`, `claim?`, `epistemic--all-required-present?`, `epistemic--claim-ref?`, `evidence-claim-ref`, `evidence-method`, `evidence-outcome`, `evidence-source-ref`, `evidence?`, `intent-capabilities-satisfied?`, `intent-goal`, `intent-produces`, `intent-requires`, `intent-stop-on`, `intent?`, `make-claim`, `make-evidence`, `make-intent`, `make-observation`, `observation-source`, `observation-statement`, `observation?`, `source-ref?`, `supporting-evidence`

> `evidence-supports?` was renamed to `supporting-evidence` (`10061fb`, this session): it now returns the matched evidence record itself, not a bare `t`, when the outcome is `supports` — the smallest experiment proving rich information need not collapse to a flag just because it's used in a `cond` test. See [`docs/language-core.md`](language-core.md).

### forward.my (83)

`*jtms-memory*`, `*justified-memory*`, `*working-memory*`, `add-justification`, `add-new-entries-jtms`, `add-new-justified`, `append-new`, `assert-fact!`, `assert-fact-jtms!`, `assert-fact-tms!`, `assert-facts!`, `axiom`, `condition-is-and?`, `condition-is-exists?`, `condition-is-forall?`, `condition-is-not?`, `condition-is-or?`, `condition-is-test?`, `dependents-of`, `drop-unsupported`, `fact-of`, `find-entry`, `fire-rule`, `fire-rule-jtms`, `fire-rule-jtms-multi`, `fire-rule-multi`, `fire-rule-on-facts`, `fire-rule-on-facts-jtms`, `fire-rule-on-facts-tms`, `fire-rule-on-working-memory`, `fire-rule-tms`, `fire-rules-jtms-multi`, `fire-rules-multi`, `fire-rules-on-facts`, `fire-rules-on-facts-jtms`, `fire-rules-on-facts-tms`, `fire-rules-on-working-memory`, `forall-every-candidate-satisfies?`, `jtms-make-state`, `jtms-state-subst`, `jtms-state-used`, `justifications-of`, `justified-member?`, `make-justified`, `map-apply-head`, `map-apply-head-jtms`, `map-fact-of`, `match-and-condition`, `match-condition-against-facts`, `match-conditions`, `match-conditions-jtms`, `match-exists-condition`, `match-forall-condition`, `match-negated-condition`, `match-one-condition`, `match-one-condition-jtms`, `match-or-condition`, `match-or-condition-jtms`, `match-plain-condition-jtms`, `match-test-condition`, `prune-all-entries`, `prune-entry`, `prune-justifications`, `remove-entry-jtms`, `remove-justified`, `retract-fact`, `retract-fact!`, `retract-fact-jtms`, `retract-fact-jtms!`, `retract-fact-tms`, `retract-fact-tms!`, `retract-facts-jtms`, `retract-facts-tms`, `run`, `run-jtms`, `run-jtms!`, `run-jtms-multi`, `run-jtms-multi!`, `run-multi`, `run-tms`, `run-tms!`, `supports-of`, `unsupported-facts`

### knowledge.my (50)

`*knowledge-journal*`, `*knowledge-package-version*`, `*usage-counts*`, `accept-knowledge-exchange`, `advice-all-decision`, `advice-batch-conflict`, `advice-conflict-proof`, `advice-decision`, `advice-negative-head-conflict`, `advise`, `advise-all`, `apply-journal-event`, `check-conflict`, `clauses->tell-events`, `collect-facts-about`, `contains-atom?`, `defmodule`, `describe`, `exchange-knowledge-package`, `forward-in`, `import-knowledge-file`, `import-knowledge-package`, `is-fact?`, `knowledge-clause-valid?`, `knowledge-clauses-valid?`, `knowledge-goal-valid?`, `knowledge-goals-valid?`, `knowledge-package-decision`, `knowledge-package-entries-valid?`, `knowledge-package-field`, `knowledge-proper-list?`, `knowledge-term-valid?`, `knowledge-terms-valid?`, `load-knowledge`, `make-knowledge-package`, `module-clauses-now`, `module-journal-events`, `module-known?`, `opposite-knowledge-head`, `reason-in`, `receive-knowledge-package`, `record-usage!`, `retract-knowledge`, `send-knowledge-package`, `string-through-line`, `tcp-read-frame`, `tcp-read-to-eof`, `tell-knowledge`, `usage-of`, `write-knowledge-package`

### linter.my (13)

`collect-free-vars`, `collect-free-vars-let*`, `collect-free-vars-letrec`, `effectful-primitives`, `get-threshold`, `lint-all`, `lint-check`, `lint-complexity`, `lint-effects`, `lint-globals`, `lint-max2`, `lint-nesting`, `lint-size`

### meta-eval.my (10)

`bind-params`, `env-lookup`, `my-apply`, `my-eval`, `my-eval-body`, `my-eval-cond`, `my-eval-list`, `my-eval-program`, `my-eval-top-form`, `my-macro?`

### narrate.my (8)

`narrate-answer`, `narrate-derivation`, `narrate-fact`, `narrate-provenance`, `provenance-derived-from`, `provenance-goal`, `provenance-rule`, `provenance-source`

### persistent-map.my (17)

`balance`, `balance-factor`, `height-of`, `make-balanced-node`, `map->list`, `map-contains?`, `map-empty`, `map-get`, `map-insert`, `max2`, `node-height`, `node-key`, `node-left`, `node-right`, `node-value`, `rotate-left`, `rotate-right`

### persistent-vector.my (23)

`vbalance`, `vbalance-factor`, `vec->list`, `vec-conj`, `vec-count`, `vec-empty`, `vec-from-list`, `vec-from-list-onto`, `vec-nth`, `vec-tree`, `vheight-of`, `vmake-balanced-node`, `vmax2`, `vnode-height`, `vnode-index`, `vnode-left`, `vnode-right`, `vnode-value`, `vrotate-left`, `vrotate-right`, `vtree->list`, `vtree-get`, `vtree-insert`

> Missing from the previous regeneration entirely — a persistent, AVL-balanced vector counterpart to `persistent-map.my`, added since the last pass.

### reason.my (20)

`add-usage`, `count-usage`, `count-usage-list`, `explain-proof`, `explain-proof-list`, `explain-proof-node`, `map-goal-results`, `map-proofs`, `merge-usage`, `print-indent`, `prove-goal`, `prove-goal-state`, `prove-goals`, `prove-rule`, `provenance`, `provenance-list`, `reason`, `reason-explain`, `rename-vars`, `source-of`

### result-status.my (7)

`make-blocked`, `make-disputed`, `make-partial`, `make-unknown`, `result-payload`, `result-status`, `result-tagged?`

### understand.my (8)

`strip-article`, `understand`, `understand-is`, `understand-query`, `understand-query-is`, `understand-query-relation`, `understand-relation`, `understand-universal`

### unify.my (15)

`apply-subst`, `apply-subst-walked`, `extend-subst`, `failed-subst?`, `logic-var`, `lookup-subst`, `occurs-check`, `thread-conjunction`, `thread-conjunction-branches`, `unify`, `unify-var`, `unify-walked`, `var?`, `walk`, `walk-resolved`

### world.my (44)

`advice-all-decision-in-world`, `advice-decision-in-world`, `advise`, `advise-all`, `advise-all-world`, `advise-world`, `defmodule`, `empty-world`, `forward-in-world`, `import-knowledge-package`, `import-knowledge-package-world`, `knowledge-content-address`, `legacy-world-transition-expansion`, `make-world`, `make-world-knowledge-package`, `reason-in-world`, `retract-knowledge`, `tell-knowledge`, `world-address-content`, `world-apply-event`, `world-at-depth`, `world-at-depth-from`, `world-branch-diff`, `world-clauses`, `world-climb-to-depth`, `world-common-ancestor`, `world-common-ancestor-aligned`, `world-content-address`, `world-depth`, `world-diff`, `world-journal`, `world-journal-prefix`, `world-metadata`, `world-module-events`, `world-module-known?`, `world-no-common-ancestor?`, `world-not-ancestor?`, `world-parent`, `world-record`, `world-remove-first`, `world-retract`, `world-tell`, `world-tell-all`, `world?`

### yantra.my (61)

`agent-loop`, `alist-ref`, `all-covered?`, `append-tool-results`, `bash-tool-schema`, `build-request-body`, `claim-markers`, `claims-execution?`, `collect-trailing-tools`, `count-with-role`, `dispatch-tool`, `encode-message`, `encode-tool-call`, `ends-with-owned-tool-results?`, `execute-bash`, `execute-tool-call`, `extract-assistant-message`, `has-tool-result?`, `http-post-json`, `http-transport-body`, `http-transport-exit`, `http-transport-stderr`, `id-in-list?`, `invalid-completion-nudge`, `json->message`, `json->tool-call`, `json-encode`, `json-encode-array`, `json-encode-array-items`, `json-encode-key`, `json-encode-object`, `json-encode-object-entries`, `json-encode-string`, `json-encode-value`, `json-escape`, `json-escape-char`, `json-escape-onto`, `json-message-content`, `json-object?`, `markers-contained?`, `max-turns`, `msg-call-ids`, `msg-content`, `msg-role`, `msg-tool-call-id`, `msg-tool-calls`, `ollama-complete`, `ollama-model`, `ollama-url`, `result-answer`, `result-epistemic-status`, `result-messages`, `result-status`, `result-turn`, `run-agent`, `strcat`, `strcat-onto`, `tc-arguments`, `tc-id`, `tc-name`, `valid-final?`
