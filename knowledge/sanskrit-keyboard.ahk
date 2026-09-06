; Sanskrit Cyrillic Keyboard — AutoHotkey MVP (AltGr direct)
; =============================================================
; Architecture: Ukrainian base layout + direct AltGr extensions
; Base: Ukrainian layout (untouched, CapsLock untouched, Win untouched)
; Modifier: Right Alt (AltGr) = primary Sanskrit extension
;           Shift+AltGr = secondary (aspirated) extension
; Principle: Ukrainian VedaBase-UA surface = authority
;            No modes, no pending state, no digit prefixes
;            CapsLock untouched, Win untouched
;
; PRIMARY EXTENSIONS (AltGr + letter):
;   Retroflex (dot below):
;     AltGr + т → т̣
;     AltGr + д → д̣
;     AltGr + р → р̣
;     AltGr + х → х̣
;
;   Long vowels (macron):
;     AltGr + а → а̄
;     AltGr + і → і̄
;     AltGr + у → ӯ
;     AltGr + е → е̄
;     AltGr + о → о̄
;
;   Palatal nasal:
;     AltGr + н → н̃
;
;   Palatal sibilant:
;     AltGr + ш → ш́
;
;   Visarga:
;     AltGr + х → х̣
;
; SECONDARY (Shift + AltGr + letter) — Aspirated digraphs:
;   Shift+AltGr+к → кг
;   Shift+AltGr+ґ → ґг
;   Shift+AltGr+д → дг
;   Shift+AltGr+б → бг
;   Shift+AltGr+п → пг
;   Shift+AltGr+т → тг
;
; CONJUNCTS (explicit sequences):
;   Retroflex + Retroflex: шт̣ (type т̣ then т̣, or use compose)
;   Palatal nasal conjunct: джн̃ (type дж then AltGr+н)
;
; CONTROLS:
;   AltGr + 0     → show help
;   AltGr + Esc   → no-op (reserved)
;   Ctrl+Alt+Pause → suspend/resume Sanskrit layer
;   Ctrl+Alt+Home  → reload script
;
; CapsLock: UNTOUCHED
; Win: UNTOUCHED

#NoEnv
#SingleInstance Force
#Warn
SendMode Input
SetWorkingDir %A_ScriptDir%

; ============================================================================
; PRIMARY EXTENSIONS (Right Alt / AltGr + letter)
; ============================================================================

; --- RETROFLEX (dot below) ---
>!t::Send {U+0442}{U+0323}  ; т → т̣
>!d::Send {U+0434}{U+0323}  ; д̣
>!r::Send {U+0440}{U+0323}  ; р̣

; --- LONG VOWELS (macron) ---
>!a::Send {U+0430}{U+0304}  ; а̄
>!i::Send {U+0456}{U+0304}  ; і̄
>!u::Send {U+0443}{U+0304}  ; ӯ
>!e::Send {U+0435}{U+0304}  ; е̄
>!o::Send {U+043E}{U+0304}  ; о̄

; --- PALATAL NASAL ---
>!n::Send {U+043D}{U+0303}  ; н̃

; --- PALATAL SIBILANT ---
>!s::Send {U+0448}{U+0301}  ; ш́ (ш key)

; --- VISARGA ---
>!h::Send {U+0445}{U+0323}  ; х̣ (h key = х on UA)

; ============================================================================
; SECONDARY (Shift + AltGr + letter) — ASPIRATED DIGRAPHS
; ============================================================================
>+!k::Send кг
>+!g::Send ґг
>+!d::Send дг
>+!b::Send бг
>+!p::Send пг
>+!t::Send тг

; ============================================================================
; CONTROLS
; ============================================================================

; AltGr + 0 → show help
>!0::
    ToolTip Sanskrit keyboard help:`n`n`n`nPrimary (AltGr+letter):`n  т→т̣  д→д̣  р→р̣  х→х̣`n  а→а̄  і→ı̄  у→ӯ`n  ш→ш́  н→н̃`n`nSecondary (Shift+AltGr):`n  к→кг  ґ→ґг  д→дг  б→бг  п→пг  т→тг`n`nConjuncts: т̣т̣, дж+н̃`n`nAltGr+0 = this help, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -5000
    return

; ============================================================================
; HELPER
; ============================================================================
RemoveToolTip:
    ToolTip
    return

; Ctrl+Alt+Pause → suspend/resume
^!Pause::
    Suspend
    ToolTip Sanskrit layer % (A_IsSuspended ? "SUSPENDED" : "ACTIVE"), A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -1500
    return

; Ctrl+Alt+Home → reload
^!Home::
    Reload
    return

