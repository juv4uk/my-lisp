; Sanskrit Cyrillic Keyboard — AutoHotkey MVP
; ===========================================
; Base: Ukrainian layout (untouched)
; Modifier: Win key toggles modes
; Dead keys: RETRO, LONG, PAL (toggled via Win+R/L/P)
;
; Usage:
;   Win+R  → toggle RETRO mode (dot below: retroflex)
;   Win+L  → toggle LONG mode (macron: long vowels)
;   Win+P  → toggle PAL mode (palatal: acute/tilde)
;   Win+A  → toggle ASP mode (aspirated digraphs)
;   Win+V  → toggle VOC mode (vocalic r/l)
;   Win+S  → toggle VIS mode (visarga)
;
; Gestures (when mode active):
;   RETRO: т→т̣  д→д̣  н→н̣  ш→ш̣  р→р̣
;   LONG:  а→а̄  і→ı̄  у→ӯ  е→е̄  о→о̄
;   PAL:   ш→ш́  н→н̃
;   ASP:   к→кг  ґ→ґг  д→дг  б→бг  п→пг
;   VOC:   р→р̣  л→л̣
;   VIS:   х→х̣
;
; Compose (always active, no mode needed):
;   Ctrl+Alt+j+n → джн̃
;   Ctrl+Alt+s+t → шт̣

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

; Win+A → ASP (aspirated digraphs)
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
; RETRO MODE GESTURES (dot below for retroflex)
; ============================================================================
#If RetroMode
    ; т → т̣ (U+0442 + U+0323)
    t::Send {U+0442}{U+0323}
    
    ; д → д̣ (U+0434 + U+0323)
    d::Send {U+0434}{U+0323}
    
    ; н → н̣ (U+043D + U+0323)
    n::Send {U+043D}{U+0323}
    
    ; ш → ш̣ (U+0448 + U+0323)
    SC013::Send {U+0448}{U+0323}  ; ш key (SC013 on standard UA)
    
    ; р → р̣ (U+0440 + U+0323)
    r::Send {U+0440}{U+0323}
    
    ; с → с̣ (retroflex s)
    s::Send {U+0441}{U+0323}
#If

; ============================================================================
; LONG MODE GESTURES (macron for long vowels)
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
; PAL MODE (palatal: acute on ш, tilde on н)
; ============================================================================
#If PalMode
    ; ш → ш́ (U+0448 + U+0301)
    SC013::Send {U+0448}{U+0301}
    
    ; н → н̃ (U+043D + U+0303)
    n::Send {U+043D}{U+0303}
    
    ; с → с́ (palatal s)
    s::Send {U+0441}{U+0301}
    
    ; з → з́ (palatal z)
    z::Send {U+0437}{U+0301}
#If

; ============================================================================
; ASP MODE (aspirated digraphs per VedaBase)
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
; VOC MODE (vocalic r/l)
; ============================================================================
#If VocMode
    ; р → р̣
    r::Send {U+0440}{U+0323}
    
    ; л → л̣
    l::Send {U+043B}{U+0323}
#If

; ============================================================================
; VIS MODE (visarga on х)
; ============================================================================
#If VisMode
    ; х → х̣
    h::Send {U+0445}{U+0323}
#If

; ============================================================================
; ALWAYS-ACTIVE COMPOSE SEQUENCES (Ctrl+Alt)
; ============================================================================
; Ctrl+Alt+j+n → джн̃ (jñ)
^!j::
    SendEvent {Ctrl up}{Alt up}джн̃
    return

; Ctrl+Alt+s+t → шт̣ (ṣṭ)
^!s::
    SendEvent {Ctrl up}{Alt up}шт̣
    return

; Ctrl+Alt+d+h → дг (dh)
^!d::
    SendEvent {Ctrl up}{Alt up}дг
    return

; Ctrl+Alt+k+h → кг (kh)
^!k::
    SendEvent {Ctrl up}{Alt up}кг
    return

; Ctrl+Alt+g+h → ґг (gh)
^!g::
    SendEvent {Ctrl up}{Alt up}ґг
    return

; Ctrl+Alt+b+h → бг (bh)
^!b::
    SendEvent {Ctrl up}{Alt up}бг
    return

; Ctrl+Alt+p+h → пг (ph)
^!p::
    SendEvent {Ctrl up}{Alt up}пг
    return

; Ctrl+Alt+t+h → тг (th)
^!t::
    SendEvent {Ctrl up}{Alt up}тг
    return

; Ctrl+Alt+j+n → джн̃ (jñ)
^!j::
    SendEvent {Ctrl up}{Alt up}джн̃
    return

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

