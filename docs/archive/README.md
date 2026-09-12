# Архів документації — не нормативно

Українською: усе під `docs/archive/` — **історичний матеріал, не специфікація**. Жоден документ тут не визначає поточну поведінку мови, контракт чи архітектурне рішення. У разі будь-якого конфлікту між файлом тут і чинним джерелом істини (Canon, `lib/surface/semantic-registry.wsm`, `tests/fixtures/conformance.my`, активні ADR, `CURRENT.md`) — **чинне джерело істини завжди перемагає беззастережно**. Архів існує, щоб зберегти доказову історію (хто що досліджував, коли і чому), а не щоб бути другою специфікацією.

This directory holds superseded design docs, completed one-off plans, exploratory research notes, and historical reviews/audits. **Nothing here is normative.** On any conflict between a file here and the current source of truth (Canon, `lib/surface/semantic-registry.wsm`, `tests/fixtures/conformance.my`, active ADRs, `CURRENT.md`), the current source of truth wins unconditionally — no exceptions, no "but this doc said."

Start at [`/CURRENT.md`](../../CURRENT.md) for the one active entry point. This README exists so an agent that stumbles into this directory understands immediately why nothing here should drive an implementation decision.

## Subdirectories

- **`superseded/`** — a document whose content was explicitly replaced by a newer, currently-active document. Look for a `Superseded-by:` line at the top of each file naming its replacement.
- **`completed-plans/`** — a plan, roadmap, or vertical-slice design whose work is done and merged; kept as the record of what was actually decided and why, not as an open task.
- **`experiments/`** — a proof-of-concept, spike, or research probe that was run, produced a result, and is not (yet, or ever) part of the shipped architecture.
- **`historical/`** — dated status reports, external reviews, audits, and point-in-time reactions from a specific session or reviewer. Valuable as evidence of what was known/believed at that time; never a current requirement.

## What moving a file here does NOT mean

- It does not mean the file is wrong — many archived documents were entirely correct for their moment and remain useful evidence.
- It does not mean the underlying question is closed — a superseded design doc's open question may still be open; check the current active plan, not the archive, for that.
- It does not mean the file is deleted — every move here is history-preserving (`git mv`), so `git log --follow` on any archived file still shows its full history.

## For agents

If you are about to cite a document under `docs/archive/` as the reason for an implementation decision, stop — that is exactly the failure mode this directory exists to prevent. Read `CURRENT.md` first, find the active document that actually governs the area you're working in, and cite that instead. An archived document may inform your understanding of *how we got here*; it may never be the *reason* for what you do next.
