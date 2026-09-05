; Sanskrit Cyrillic Keyboard — AutoHotkey MVP (AltGr prefix scheme)
; =================================================================
; Architecture: Ukrainian base layout + phonological gesture layer on AltGr
; Base: Ukrainian layout (untouched, CapsLock untouched, Win untouched)
; Modifier: Right Alt (AltGr) for ALL Sanskrit operations
; Principle: Ukrainian VedaBase-UA surface = authority
;            No ambiguous AltGr+letter bindings
;            CapsLock untouched, Win untouched
;
; SCHEME: AltGr + [1..6] as ONE-SHOT OPERATION PREFIX
;   AltGr + 1  → RETRO  (next key = retroflex)
;   AltGr + 2  → LONG   (next key = long vowel)
;   AltGr + 3  → PAL    (next key = palatal)
;   AltGr + 4  → ASP    (next key = aspirated digraph)
;   AltGr + 5  → VOC    (next key = vocalic r/l)
;   AltGr + 6  → VIS    (next key = visarga)
;
;   AltGr + 0  → show active modes
;   AltGr + Esc → clear all pending operations
;
; SCHEME EXAMPLES:
;   AltGr+1, т  → RETRO(t)   → т̣
;   AltGr+4, д  → ASP(d)     → дг
;   AltGr+3, н  → PAL_NASAL(n) → н̃
;   AltGr+1, т  → RETRO(t)   → т̣
;   AltGr+2, а  → LONG(a)    → а̄
;   AltGr+5, р  → VOC(r)     → р̣
;   AltGr+6, х  → VIS(x)     → х̣
;
; CONJUNCTS:
;   RETRO(ш) + RETRO(т) → шт̣
;   PAL(дж) + PAL_NASAL(н) → джн̃
;
; CONTROLS:
;   AltGr + 0  → show active modes
;   AltGr + Esc → clear pending operation
;   Ctrl+Alt+Pause → suspend/resume Sanskrit layer
;   Ctrl+Alt+Home → reload script
;
; CapsLock: UNTOUCHED
; Win: UNTOUCHED
; Right Alt (AltGr) = Sanskrit modifier ONLY

#NoEnv
#SingleInstance Force
#Warn
SendMode Input
SetWorkingDir %A_ScriptDir%

; ============================================================================
; GLOBAL STATE
; ============================================================================
global PendingOp := ""
global PendingOpName := ""

; ============================================================================
; OPERATION PREFIX KEYS (AltGr + digit)
; ============================================================================

; AltGr + 1 → RETRO (retroflex dot below)
>!1::
    PendingOp := "RETRO"
    PendingOpName := "RETRO"
    ShowOpTip("RETRO")
    return

; AltGr + 2 → LONG (macron)
>!2::
    PendingOp := "LONG"
    PendingOpName := "LONG"
    ShowOpTip("LONG")
    return

; AltGr + 3 → PAL (palatal)
>!3::
    PendingOp := "PAL"
    PendingOpName := "PAL"
    ShowOpTip("PAL")
    return

; AltGr + 4 → ASP (aspiration digraph)
>!4::
    PendingOp := "ASP"
    PendingOpName := "ASP"
    ShowOpTip("ASP")
    return

; AltGr + 5 → VOC (vocalic r/l)
>!5::
    PendingOp := "VOC"
    PendingOpName := "VOC"
    ShowOpTip("VOC")
    return

; AltGr + 6 → VIS (visarga)
>!6::
    PendingOp := "VIS"
    PendingOpName := "VIS"
    ShowOpTip("VIS")
    return

; ============================================================================
; CLEAR / STATUS
; ============================================================================

; AltGr + 0 → show pending operation
>!0::
    if (PendingOp = "")
        ToolTip No pending Sanskrit operation, A_CaretX + 20, A_CaretY - 20
    else
        ToolTip Pending Sanskrit operation: %PendingOpName%, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -3000
    return

; AltGr + Esc → clear pending operation
>!Esc::
    PendingOp := ""
    PendingOpName := ""
    ToolTip Sanskrit operation cleared, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -1500
    return

; ============================================================================
; OPERATION EXECUTION (next key after prefix)
; ============================================================================
#If PendingOp = "RETRO"
    ; Retroflex: dot below
    t::Send {U+0442}{U+0323}  ; т → т̣
    d::Send {U+0434}{U+0323}  ; д̣
    n::Send {U+043D}{U+0323}  ; н̣
    SC013::Send {U+0448}{U+0323}  ; ш̣
    r::Send {U+0440}{U+0323}  ; р̣
    s::Send {U+0441}{U+0323}  ; с̣
    ClearPending()
#If

#If PendingOp = "LONG"
    ; Long vowels: macron
    a::Send {U+0430}{U+0304}  ; а̄
    i::Send {U+0456}{U+0304}  ; і̄
    u::Send {U+0443}{U+0304}  ; ӯ
    e::Send {U+0435}{U+0304}  ; е̄
    o::Send {U+043E}{U+0304}  ; о̄
    ClearPending()
#If

#If PendingOp = "PAL"
    ; Palatal: acute on ш/с/з, tilde on н
    SC013::Send {U+0448}{U+0301}  ; ш́
    n::Send {U+043D}{U+0303}  ; н̃
    s::Send {U+0441}{U+0301}  ; с́
    z::Send {U+0437}{U+0301}  ; з́
    ClearPending()
#If

#If PendingOp = "ASP"
    ; Aspirated digraphs per VedaBase-UA
    k::Send кг
    g::Send ґг
    d::Send дг
    b::Send бг
    p::Send пг
    t::Send тг
    ClearPending()
#If

#If PendingOp = "VOC"
    ; Vocalic: dot below on р/л
    r::Send {U+0440}{U+0323}  ; р̣
    l::Send {U+043B}{U+0323}  ; л̣
    ClearPending()
#If

#If PendingOp = "VIS"
    h::Send {U+0445}{U+0323}  ; х̣
    ClearPending()
#If

; ============================================================================
; HELPER: Clear pending operation
; ============================================================================
ClearPending() {
    global PendingOp := ""
    global PendingOpName := ""
}

; ============================================================================
; HELPER: Show operation tooltip
; ============================================================================
ShowOpTip(op) {
    ToolTip Sanskrit operation: %op%, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -2000
    return

RemoveToolTip:
    ToolTip
    return

; AltGr + 0 → show pending operation
>!0::
    if (PendingOp = "")
        ToolTip No pending Sanskrit operation, A_CaretX + 20, A_CaretY - 20
    else
        ToolTip Pending Sanskrit operation: %PendingOpName%, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -3000
    return
}

; AltGr + Esc → clear pending operation
>!Esc::
    PendingOp := ""
    PendingOpName := ""
    ToolTip Sanskrit operation cleared, A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -1500
    return

; ============================================================================
; CONJUNCTS (mode combos - enabled via sticky modes for now)
; ============================================================================
; For conjuncts, use sticky modes as before:
; Win+R → RETRO sticky, then ш + т → шт̣
; Win+P → PAL sticky, then дж + AltGr+3,н → джн̃

global RetroMode := false
global LongMode := false
global PalMode := false
global AspMode := false
global VocMode := false
global VisMode := false

; Sticky mode toggles (Win key for series input)
~LWin & r::
    RetroMode := !RetroMode
    ShowModeTip("RETRO", RetroMode)
    return

~LWin & l::
    LongMode := !LongMode
    ShowModeTip("LONG", LongMode)
    return

~LWin & p::
    PalMode := !PalMode
    ShowModeTip("PAL", PalMode)
    return

~LWin & a::
    AspMode := !AspMode
    ShowModeTip("ASP", AspMode)
    return

~LWin & v::
    VocMode := !VocMode
    ShowModeTip("VOC", VocMode)
    return

~LWin & s::
    VisMode := !VisMode
    ShowModeTip("VIS", VisMode)
    return

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
; CONTROLS
; ============================================================================

; Win+Space → show active sticky modes
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

; Win+Esc → clear all sticky modes
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

