; Sanskrit Cyrillic Keyboard — AutoHotkey MVP (AltGr one-shot)
; =============================================================
; Architecture: Ukrainian base layout + phonological gesture layer on AltGr
; Base: Ukrainian layout (untouched, CapsLock untouched)
; Modifier: Right Alt (AltGr) for one-shot phonological gestures
; Principle: Ukrainian VedaBase-UA surface = authority
;            Latin/IAST compose sequences REMOVED
;            CapsLock untouched
;
; One-shot gestures (hold AltGr, press base key, release):
;   AltGr + т → т̣ (retroflex)
;   AltGr + д → д̣
;   AltGr + н → н̣
;   AltGr + ш → ш̣
;   AltGr + р → р̣
;   AltGr + с → с̣
;
;   AltGr + а → а̄ (macron)
;   AltGr + і → і̄
;   AltGr + у → ӯ
;   AltGr + е → е̄
;   AltGr + о → о̄
;
;   AltGr + ш → ш́ (palatal acute)
;   AltGr + н → н̃ (palatal nasal tilde)
;   AltGr + с → с́
;   AltGr + з → з́
;
;   AltGr + к → кг (aspirated digraph)
;   AltGr + ґ → ґг
;   AltGr + д → дг
;   AltGr + б → бг
;   AltGr + п → пг
;   AltGr + т → тг
;
;   AltGr + р → р̣ (vocalic r)
;   AltGr + л → л̣
;
;   AltGr + х → х̣ (visarga)
;
; Conjuncts (mode combos):
;   RETRO then PAL: шт̣
;   PAL then NASAL: джн̃
;
; Toggles (Win key for mode-based input):
;   Win+R → RETRO mode (sticky)
;   Win+L → LONG mode
;   Win+P → PAL mode
;   Win+A → ASP mode (sticky for multiple aspirates)
;   Win+V → VOC mode
;   Win+S → VIS mode
;
; Controls:
;   Win+Space → show active modes
;   Win+Esc → disable all modes
;   Win+Ctrl+S → suspend/resume
;   Win+Ctrl+R → reload script
;
; CapsLock: UNTOUCHED (preserves Windows uppercase semantics)

#NoEnv
#SingleInstance Force
#Warn
SendMode Input
SetWorkingDir %A_ScriptDir%

; ============================================================================
; GLOBAL STATE (for mode-based toggles)
; ============================================================================
global RetroMode := false
global LongMode := false
global PalMode := false
global AspMode := false
global VocMode := false
global VisMode := false

; ============================================================================
; ONE-SHOT GESTURES (AltGr = Right Alt)
; ============================================================================

; --- RETROFLEX (dot below) ---
>!t::Send {U+0442}{U+0323}  ; т → т̣
>!d::Send {U+0434}{U+0323}  ; д̣
>!n::Send {U+043D}{U+0323}  ; н̣
>!s::Send {U+0448}{U+0323}  ; ш̣
>!r::Send {U+0440}{U+0323}  ; р̣
>!s::Send {U+0441}{U+0323}  ; с̣ (using s for с̣)

; --- LONG VOWELS (macron) ---
>!a::Send {U+0430}{U+0304}  ; а̄
>!i::Send {U+0456}{U+0304}  ; і̄
>!u::Send {U+0443}{U+0304}  ; ӯ
>!e::Send {U+0435}{U+0304}  ; е̄
>!o::Send {U+043E}{U+0304}  ; о̄

; --- PALATAL (acute/tilde) ---
>!s::Send {U+0448}{U+0301}  ; ш́ (SC013 = ш key)
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
>!h::Send {U+0445}{U+0323}  ; х̣ (h key = х on UA layout)

; ============================================================================
; MODE TOGGLES (Win + key) — sticky modes for repeated use
; ============================================================================

global RetroMode := false
global LongMode := false
global PalMode := false
global AspMode := false
global VocMode := false
global VisMode := false

; Win+R → RETRO mode (sticky)
~LWin & r::
    RetroMode := !RetroMode
    ShowModeTip("RETRO", RetroMode)
    return

; Win+L → LONG mode
~LWin & l::
    LongMode := !LongMode
    ShowModeTip("LONG", LongMode)
    return

; Win+P → PAL mode
~LWin & p::
    PalMode := !PalMode
    ShowModeTip("PAL", PalMode)
    return

; Win+A → ASP mode (sticky for multiple aspirates)
~LWin & a::
    AspMode := !AspMode
    ShowModeTip("ASP", AspMode)
    return

; Win+V → VOC mode
~LWin & v::
    VocMode := !VocMode
    ShowModeTip("VOC", VocMode)
    return

; Win+S → VIS mode
~LWin & s::
    VisMode := !VisMode
    ShowModeTip("VIS", VisMode)
    return

; ============================================================================
; MODE-BASED GESTURES (when sticky mode active)
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

#If VisMode
    h::Send {U+0445}{U+0323}
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
; CONTROLS
; ============================================================================

; Win+Space → show active modes
#LWin & Space::
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

; Win+Esc → disable all modes
#LWin & Esc::
    RetroMode := false
    LongMode := false
    PalMode := false
    AspMode := false
    VocMode := false
    VisMode := false
    ToolTip All Sanskrit modes OFF, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -1500
    return

; Win+Ctrl+S → suspend/resume
#LWin & ^s::
    Suspend
    ToolTip Sanskrit layer % (A_IsSuspended ? "SUSPENDED" : "ACTIVE"), A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -1500
    return

; Win+Ctrl+R → reload script
#LWin & ^r::
    Reload
    return

