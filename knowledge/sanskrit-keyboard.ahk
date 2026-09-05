; Sanskrit Cyrillic Keyboard — AutoHotkey MVP (AltGr only)
; ========================================================
; Architecture: Ukrainian base layout + phonological gesture layer on AltGr
; Base: Ukrainian layout (untouched, CapsLock untouched, Win untouched)
; Modifier: Right Alt (AltGr) for ALL Sanskrit operations
; Principle: Ukrainian VedaBase-UA surface = authority
;            Latin/IAST compose sequences REMOVED
;            CapsLock untouched, Win untouched
;
; ONE-SHOT GESTURES (AltGr + letter, press and release):
;   Retroflex (dot below):
;     AltGr + т → т̣
;     AltGr + д → д̣
;     AltGr + н → н̣
;     AltGr + ш → ш̣
;     AltGr + р → р̣
;     AltGr + с → с̣
;
;   Long vowels (macron):
;     AltGr + а → а̄
;     AltGr + і → і̄
;     AltGr + у → ӯ
;     AltGr + е → е̄
;     AltGr + о → о̄
;
;   Palatal (acute/tilde):
;     AltGr + ш → ш́
;     AltGr + н → н̃
;     AltGr + с → с́
;     AltGr + з → з́
;
;   Aspirated digraphs (VedaBase-UA convention):
;     AltGr + к → кг
;     AltGr + ґ → ґг
;     AltGr + д → дг
;     AltGr + б → бг
;     AltGr + п → пг
;     AltGr + т → тг
;
;   Vocalic (dot below on р/л):
;     AltGr + р → р̣
;     AltGr + л → л̣
;
;   Visarga:
;     AltGr + х → х̣
;
; STICKY MODES (AltGr + digit, toggle):
;   AltGr + 1 → RETRO mode (sticky retroflex)
;   AltGr + 2 → LONG mode (sticky long vowels)
;   AltGr + 3 → PAL mode (sticky palatal)
;   AltGr + 4 → ASP mode (sticky aspirated digraphs)
;   AltGr + 5 → VOC mode (sticky vocalic)
;   AltGr + 6 → VIS mode (sticky visarga)
;
; CONTROLS:
;   AltGr + 0       → show active modes
;   AltGr + Esc     → clear all sticky modes
;   Ctrl+Alt+Pause  → suspend/resume Sanskrit layer
;   Ctrl+Alt+Home   → reload script
;
; CapsLock: UNTOUCHED (preserves Windows uppercase semantics)
; Win key: UNTOUCHED (system reserved)

#NoEnv
#SingleInstance Force
#Warn
SendMode Input
SetWorkingDir %A_ScriptDir%

; ============================================================================
; GLOBAL STATE (sticky modes)
; ============================================================================
global RetroMode := false
global LongMode := false
global PalMode := false
global AspMode := false
global VocMode := false
global VisMode := false

; ============================================================================
; ONE-SHOT GESTURES (Right Alt / AltGr + letter)
; ============================================================================

; --- RETROFLEX (dot below) ---
>!t::Send {U+0442}{U+0323}  ; т → т̣
>!d::Send {U+0434}{U+0323}  ; д̣
>!n::Send {U+043D}{U+0323}  ; н̣
>!s::Send {U+0448}{U+0323}  ; ш̣ (ш key)
>!r::Send {U+0440}{U+0323}  ; р̣
>!s::Send {U+0441}{U+0323}  ; с̣ (s key = с on UA)

; --- LONG VOWELS (macron) ---
>!a::Send {U+0430}{U+0304}  ; а̄
>!i::Send {U+0456}{U+0304}  ; і̄
>!u::Send {U+0443}{U+0304}  ; ӯ
>!e::Send {U+0435}{U+0304}  ; е̄
>!o::Send {U+043E}{U+0304}  ; о̄

; --- PALATAL (acute/tilde) ---
>!s::Send {U+0448}{U+0301}  ; ш́ (ш key)
>!n::Send {U+043D}{U+0303}  ; н̃
>!s::Send {U+0441}{U+0301}  ; с́
>!z::Send {U+0437}{U+0301}  ; з́

; --- ASPIRATED DIGRAPHS (VedaBase-UA convention) ---
>!k::Send кг
>!g::Send ґг
>!d::Send дг
>!b::Send бг
>!p::Send пг
>!t::Send тг

; --- VOCALIC (dot below on р/л) ---
>!r::Send {U+0440}{U+0323}  ; р̣
>!l::Send {U+043B}{U+0323}  ; л̣

; --- VISARGA ---
>!h::Send {U+0445}{U+0323}  ; х̣ (h key = х on UA)

; ============================================================================
; STICKY MODES (AltGr + digit, toggle)
; ============================================================================
global RetroMode := false
global LongMode := false
global PalMode := false
global AspMode := false
global VocMode := false
global VisMode := false

; AltGr + 1 → RETRO mode
>!1::
    RetroMode := !RetroMode
    ShowModeTip("RETRO", RetroMode)
    return

; AltGr + 2 → LONG mode
>!2::
    LongMode := !LongMode
    ShowModeTip("LONG", LongMode)
    return

; AltGr + 3 → PAL mode
>!3::
    PalMode := !PalMode
    ShowModeTip("PAL", PalMode)
    return

; AltGr + 4 → ASP mode
>!4::
    AspMode := !AspMode
    ShowModeTip("ASP", AspMode)
    return

; AltGr + 5 → VOC mode
>!5::
    VocMode := !VocMode
    ShowModeTip("VOC", VocMode)
    return

; AltGr + 6 → VIS mode
>!6::
    VisMode := !VisMode
    ShowModeTip("VIS", VisMode)
    return

; ============================================================================
; STICKY MODE GESTURES (when mode active)
; ============================================================================
#If RetroMode
    t::Send {U+0442}{U+0323}
    d::Send {U+0434}{U+0323}
    n::Send {U+043D}{U+0323}
    SC013::Send {U+0448}{U+0323}
    r::Send {U+0440}{U+0323}
    s::Send {U+0441}{U+0323}
#If

#If LongMode
    a::Send {U+0430}{U+0304}
    i::Send {U+0456}{U+0304}
    u::Send {U+0443}{U+0304}
    e::Send {U+0435}{U+0304}
    o::Send {U+043E}{U+0304}
#If

#If PalMode
    SC013::Send {U+0448}{U+0301}
    n::Send {U+043D}{U+0303}
    s::Send {U+0441}{U+0301}
    z::Send {U+0437}{U+0301}
#If

#If AspMode
    k::Send кг
    g::Send ґг
    d::Send дг
    b::Send бг
    p::Send пг
    t::Send тг
#If

#If VocMode
    r::Send {U+0440}{U+0323}
    l::Send {U+043B}{U+0323}
#If

; ============================================================================
; HELPER: Show mode tooltip
; ============================================================================
ShowModeTip(mode, state) {
    ToolTip % mode " mode: " (state ? "ON" : "OFF"), A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -1500
    return

RemoveToolTip:
    ToolTip
    return
}

; ============================================================================
; CONTROLS (AltGr + key)
; ============================================================================

; AltGr + 0 → show active modes
>!0::
    modes := ""
    modes .= (RetroMode ? "RETRO " : "")
    modes .= (LongMode ? "LONG " : "")
    modes .= (PalMode ? "PAL " : "")
    modes .= (AspMode ? "ASP " : "")
    modes .= (VocMode ? "VOC " : "")
    modes .= (VisMode ? "VIS " : "")
    if (modes = "")
        modes := "(none)"
    ToolTip Active Sanskrit modes: %modes%, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -3000
    return

; AltGr + Esc → clear all modes
>!Esc::
    RetroMode := false
    LongMode := false
    PalMode := false
    AspMode := false
    VocMode := false
    VisMode := false
    ToolTip All Sanskrit modes OFF, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -1500
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

