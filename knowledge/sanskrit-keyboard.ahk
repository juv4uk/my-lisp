; Sanskrit Cyrillic Keyboard — AutoHotkey v2
; ===========================================
; Architecture: Ukrainian base layout + direct AltGr extensions
; Base: Ukrainian layout (untouched, CapsLock untouched, Win untouched)
; Modifier: Right Alt (AltGr) = primary Sanskrit extension
;           Shift+AltGr = secondary (aspirated) extension
; Principle: Ukrainian VedaBase-UA surface = authority
;            No modes, no pending state, no digit prefixes
;            CapsLock untouched, Win untouched

#Requires AutoHotkey v2.0

; ============================================================================
; PRIMARY EXTENSIONS (Right Alt / AltGr + letter)
; ============================================================================

; --- RETROFLEX (dot below) ---
*>!t::Send "{U+0442}{U+0323}"   ; т → т̣
*>!d::Send "{U+0434}{U+0323}"   ; д̣
*>!r::Send "{U+0440}{U+0323}"   ; р̣

; --- LONG VOWELS (macron) ---
*>!a::Send "{U+0430}{U+0304}"   ; а̄
*>!i::Send "{U+0456}{U+0304}"   ; і̄
*>!u::Send "{U+0443}{U+0304}"   ; ӯ
*>!e::Send "{U+0435}{U+0304}"   ; е̄
*>!o::Send "{U+043E}{U+0304}"   ; о̄

; --- PALATAL NASAL ---
*>!n::Send "{U+043D}{U+0303}"   ; н̃

; --- PALATAL SIBILANT ---
*>!s::Send "{U+0448}{U+0301}"   ; ш́ (ш key)

; --- VISARGA ---
*>!h::Send "{U+0445}{U+0323}"   ; х̣ (h key = х on UA)

; ============================================================================
; SECONDARY (Shift + AltGr + letter) — ASPIRATED DIGRAPHS
; ============================================================================
*+!k::Send "кг"
*+!g::Send "ґг"
*+!d::Send "дг"
*+!b::Send "бг"
*+!p::Send "пг"
*+!t::Send "тг"

; ============================================================================
; CONTROLS
; ============================================================================

; RightAlt + 0 → show help
*>!0::
    MsgBox "Sanskrit keyboard help`n`n"
        . "Primary (AltGr+letter):`n"
        . "  т→т̣  д→д̣  р→р̣  х→х̣`n"
        . "  а→а̄  і→ı̄  у→ӯ  е→е̄  о→о̄`n"
        . "  н→н̃  ш→ш́`n`n"
        . "Secondary (Shift+AltGr):`n"
        . "  к→кг  ґ→ґг  д→дг  б→бг  п→пг  т→тг`n`n"
        . "Conjuncts: т̣т̣ (retroflex), дж+AltGr+н → джн̃"
        , "Sanskrit keyboard help"

; Ctrl+Alt+Pause → suspend/resume
^!Pause::
    Suspend
    ToolTip("Sanskrit layer " (A_IsSuspended ? "SUSPENDED" : "ACTIVE"))
    SetTimer(() => ToolTip(), -1500)
    return

; Ctrl+Alt+Home → reload
^!Home::
    Reload()
    return

