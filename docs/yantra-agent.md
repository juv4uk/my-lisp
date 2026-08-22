# my-lisp-yantra — the smallest coding agent in .my

> A Chebupelka-style agent whose control logic lives entirely in my-lisp.
> The host boundary is one generic primitive; everything else is `.my`.

## Architecture

`lib/yantra.my` implements the whole agent: messages, agent state,
tool-call representation, dispatch, turn loop, hard MAX_TURNS, and
completion validation. No agent logic exists in Rust.

Host capabilities (all generic, none agent-specific):

| Capability | Where | Note |
|---|---|---|
| bash tool | existing `process-run` | allowlist-gated (`--allow-process=bash,curl`) |
| HTTP POST | existing `process-run` + curl | transport-local; swap for a native primitive later if ever justified |
| JSON decode | new `json-parse` primitive | provably not expressible in `.my`: `\uXXXX` escapes need int→char construction, which no primitive provides |
| JSON encode | pure `.my` (`json-encode*`) | built on `string-append`/`number->string`; note the kernel's `string-append` is binary *and* a special form, so `yantra.my` derives an n-ary `strcat` |

## Message shapes

```lisp
((role . "user") (content . "..."))
;; assistant carrying tool calls:
((role . "assistant") (content . "")
 (tool-calls (((id . "call_1") (name . "bash")
               (arguments . "{\"cmd\": \"pwd\"}")))))
;; tool result — correlated by construction:
((role . "tool") (tool-call-id . "call_1") (content . "/home/x\n"))
```

## The key rule

A textual claim that a command was executed is NOT evidence of
execution. `valid-final?` refuses to finish any turn whose text claims
execution ("ran", "executed", "виконав", …) unless the conversation
already contains at least one real tool-result message. Such replies get
a system nudge and the loop continues instead of completing.

## Turn loop

`run-agent complete system-prompt user-prompt` →
`agent-loop` threads immutable state; stops on completion or at
`max-turns` (6), returning a tagged result:

```lisp
((status . completed) (answer . "...") (turn . 1) (messages ...))
((status . max-turns-reached) (turn . 6) (messages ...))
```

## Live wiring

`ollama-complete` targets `http://127.0.0.1:11434/v1/chat/completions`
with model `qwen3:4b` through the OpenAI-compatible tools API. Tests do
NOT require a running server: they inject scripted `complete` functions,
while the bash tool itself executes for real.

```bash
printf '%s\n' '(load "lib/yantra.my")' \
  '(print (result-status (run-agent ollama-complete "You are yantra." "What is 2+2?")))' \
  | ./target/debug/my-lisp --allow-process=bash,curl
```

## Tests

`crates/my-lisp/tests/yantra.rs`: pure question finishes without a tool;
filesystem question invokes real bash; textual-claim-only reply can never
finish (MAX_TURNS); tool results correlated by `tool_call_id`; hard
MAX_TURNS limit; JSON encode/parse round trip.
