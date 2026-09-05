; Sanskrit Cyrillic Keyboard — AutoHotkey MVP (Corrected)
; ========================================================
; Architecture: Ukrainian base layout + phonological gesture layer
; Base: Ukrainian layout (untouched)
; Gesture layer: Win-key toggled modes (dead-key equivalents)
; Principle: Ukrainian VedaBase-UA surface = authority
;            Latin/IAST compose sequences REMOVED
;
; Modes (Win + key toggle):
;   Win+R → RETRO (retroflex: dot below)
;   Win+L → LONG  (vowel length: macron)
;   Win+P → PAL   (palatal: acute/tilde)
;   Win+A → ASP   (aspiration: digraphs per VedaBase)
;   Win+V → VOC   (vocalic r/l: dot below)
;   Win+S → VIS   (visarga: dot below on х)
;
; Gestures (when mode active):
;   RETRO: т→т̣ д→д̣ н→н̣ ш→ш̣ р→р̣ с→с̣
;   LONG:  а→а̄ і→ı̄ у→ӯ е→е̄ о→о̄
;   PAL:   ш→ш́ н→н̃ с→с́ з→з́
;   ASP:   к→кг ґ→ґг д→дг б→бг п→пг т→тг
;   VOC:   р→р̣ л→л̣
;   VIS:   х→х̣
;
; Conjuncts: via mode combinations (not Latin compose)
;   RETRO+PAL: шт̣
;   PAL+NASAL: джн̃
;
; Controls:
;   Win+Space → show active modes
;   Win+Esc   → disable all modes
;   Win+Ctrl+S → suspend/resume
;   Win+Ctrl+R → reload script

#NoEnv
#SingleInstance Force
#Warn
SendMode Input
SetWorkingDir %A_ScriptDir%

; ============================================================================
; GLOBAL STATE
; ============================================================================
global RetroMode := false
global LongMode := false
global PalMode := false
global AspMode := false
global VocMode := false
global VisMode := false

; ============================================================================
; MODE TOGGLES (Win + key)
; ============================================================================

; Win+R → RETRO
~LWin & r::
    RetroMode := !RetroMode
    ShowModeTip("RETRO", RetroMode)
    return

; Win+L → LONG
~LWin & l::
    LongMode := !LongMode
    ShowModeTip("LONG", LongMode)
    return

; Win+P → PAL (palatal)
~LWin & p::
    PalMode := !PalMode
    ShowModeTip("PAL", PalMode)
    return

; Win+A → ASP (aspiration)
~LWin & a::
    AspMode := !AspMode
    ShowModeTip("ASP", AspMode)
    return

; Win+V → VOC (vocalic r/l)
~LWin & v::
    VocMode := !VocMode
    ShowModeTip("VOC", VocMode)
    return

; Win+S → VIS (visarga)
~LWin & s::
    VisMode := !VisMode
    ShowModeTip("VIS", VisMode)
    return

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
; RETRO MODE (dot below for retroflex)
; ============================================================================
#If RetroMode
    ; т → т̣ (U+0442 + U+0323)
    t::Send {U+0442}{U+0323}
    
    ; д → д̣ (U+0434 + U+0323)
    d::Send {U+0434}{U+0323}
    
    ; н → н̣ (U+043D + U+0323)
    n::Send {U+043D}{U+0323}
    
    ; ш → ш̣ (U+0448 + U+0323)
    SC013::Send {U+0448}{U+0323}
    
    ; р → р̣ (U+0440 + U+0323)
    r::Send {U+0440}{U+0323}
    
    ; с → с̣ (retroflex s)
    s::Send {U+0441}{U+0323}
#If

; ============================================================================
; LONG MODE (macron for long vowels)
; ============================================================================
#If LongMode
    ; а → а̄ (U+0430 + U+0304)
    a::Send {U+0430}{U+0304}
    
    ; і → і̄ (U+0456 + U+0304)
    i::Send {U+0456}{U+0304}
    
    ; у → ӯ (U+0443 + U+0304)
    u::Send {U+0443}{U+0304}
    
    ; е → е̄ (U+0435 + U+0304)
    e::Send {U+0435}{U+0304}
    
    ; о → о̄ (U+043E + U+0304)
    o::Send {U+043E}{U+0304}
#If

; ============================================================================
; PAL MODE (palatal: acute on ш/с/з, tilde on н)
; ============================================================================
#If PalMode
    ; ш → ш́ (U+0448 + U+0301)
    SC013::Send {U+0448}{U+0301}
    
    ; н → н̃ (U+043D + U+0303)
    n::Send {U+043D}{U+0303}
    
    ; с → с́ (U+0441 + U+0301)
    s::Send {U+0441}{U+0301}
    
    ; з → з́ (U+0437 + U+0301)
    z::Send {U+0437}{U+0301}
#If

; ============================================================================
; ASP MODE (aspirated digraphs per VedaBase-UA)
; ============================================================================
#If AspMode
    ; к → кг
    k::Send кг
    
    ; ґ → ґг
    g::Send ґг
    
    ; д → дг
    d::Send дг
    
    ; б → бг
    b::Send бг
    
    ; п → пг
    p::Send пг
    
    ; т → тг
    t::Send тг
#If

; ============================================================================
; VOC MODE (vocalic r/l: dot below)
; ============================================================================
#If VocMode
    ; р → р̣ (U+0440 + U+0323)
    r::Send {U+0440}{U+0323}
    
    ; л → л̣ (U+043B + U+0323)
    l::Send {U+043B}{U+0323}
#If

; ============================================================================
; VIS MODE (visarga on х)
; ============================================================================
#If VisMode
    ; х → х̣ (U+0445 + U+0323)
    h::Send {U+0445}{U+0323}
#If

; ============================================================================
; CONJUNCT SEQUENCES (mode combinations, NOT Latin compose)
; ============================================================================
; RETRO + PAL → шт̣ (SA_TTA_CONJ)
; Enter: RETRO mode, then PalMode, type ш then т
; Or: enable both modes, type ш then т

; PAL + NASAL → джн̃ (SA_JNYA)
; Enter: PalMode, type дж then NASAL+н
; Implementation: use existing PalMode+n for н̃, precede with дж

; ============================================================================
; STATUS DISPLAY (Win+Space shows current modes)
; ============================================================================
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

; ============================================================================
; TOGGLE ALL OFF (Win+Esc)
; ============================================================================
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

; ============================================================================
; RELOAD SCRIPT (Win+Ctrl+R)
; ============================================================================
#LWin & ^r::
    Reload
    return

; ============================================================================
; SUSPEND (Win+Ctrl+S)
; ============================================================================
#LWin & ^s::
    Suspend
    ToolTip Sanskrit layer % (A_IsSuspended ? "SUSPENDED" : "ACTIVE"), A_CaretX + 20, A_CaretY - 20
    SetTimer RemoveToolTip, -1500
    return

