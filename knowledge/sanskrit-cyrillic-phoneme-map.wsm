(sound-unit-registry
  (note "Candidate registry — not finalized standard. Glyphs and gestures unresolved pending extended Cyrillic audit.")
  (status-vocabulary
    (observed "external script value witnessed")
    (candidate "current proposed mapping")
    (unresolved "no mapping selected")
    (ready "passed Cyrillic audit + round-trip tests"))

  ;; ============================================================
  ;; VOWELS
  ;; ============================================================
  (sound-unit
    (id SA_A)
    (features (place open) (manner vowel) (voice voiced) (length short))
    (surface (devanagari "अ") (iast "a") (cyrillic "а"))
    (gesture (base "а"))
    (status candidate))

  (sound-unit
    (id SA_A_LONG)
    (features (place open) (manner vowel) (voice voiced) (length long))
    (surface (devanagari "आ") (iast "ā") (cyrillic "а̄"))
    (gesture (dead-long "а"))
    (status candidate))

  (sound-unit
    (id SA_I)
    (features (place close) (manner vowel) (voice voiced) (length short))
    (surface (devanagari "इ") (iast "i") (cyrillic "и"))
    (gesture (base "и"))
    (status candidate))

  (sound-unit
    (id SA_I_LONG)
    (features (place close) (manner vowel) (voice voiced) (length long))
    (surface (devanagari "ई") (iast "ī") (cyrillic "ӣ"))
    (gesture (dead-long "и"))
    (status candidate))

  (sound-unit
    (id SA_U)
    (features (place close) (manner vowel) (voice voiced) (length short) (rounding rounded))
    (surface (devanagari "उ") (iast "u") (cyrillic "у"))
    (gesture (base "у"))
    (status candidate))

  (sound-unit
    (id SA_U_LONG)
    (features (place close) (manner vowel) (voice voiced) (length long) (rounding rounded))
    (surface (devanagari "ऊ") (iast "ū") (cyrillic "ӯ"))
    (gesture (dead-long "у"))
    (status candidate))

  (sound-unit
    (id SA_VOCALIC_R)
    (features (manner vocalic) (place retroflex) (length short))
    (surface (devanagari "ऋ") (iast "ṛ") (cyrillic unresolved))
    (gesture (dead-vocalic "р"))
    (status unresolved))

  (sound-unit
    (id SA_VOCALIC_R_LONG)
    (features (manner vocalic) (place retroflex) (length long))
    (surface (devanagari "ॠ") (iast "ṝ") (cyrillic unresolved))
    (gesture (dead-long (dead-vocalic "р")))
    (status unresolved))

  (sound-unit
    (id SA_VOCALIC_L)
    (features (manner vocalic) (place dental) (length short))
    (surface (devanagari "ऌ") (iast "ḷ") (cyrillic unresolved))
    (gesture (dead-vocalic "л"))
    (status unresolved))

  (sound-unit
    (id SA_VOCALIC_L_LONG)
    (features (manner vocalic) (place dental) (length long))
    (surface (devanagari "ॡ") (iast "ḹ") (cyrillic unresolved))
    (gesture (dead-long (dead-vocalic "л")))
    (status unresolved))

  ;; ============================================================
  ;; CONSONANTS — VARGA MATRIX
  ;; ============================================================
  ;; Row: KA (velar)
  (sound-unit
    (id SA_KA)
    (features (place velar) (manner stop) (voice voiceless) (aspiration none))
    (surface (devanagari "क") (iast "k") (cyrillic "к"))
    (gesture (base "к"))
    (status candidate))

  (sound-unit
    (id SA_KHA)
    (features (place velar) (manner stop) (voice voiceless) (aspiration aspirated))
    (surface (devanagari "ख") (iast "kh") (cyrillic candidate))
    (gesture (dead-asp "к"))
    (status candidate))

  (sound-unit
    (id SA_GA)
    (features (place velar) (manner stop) (voice voiced) (aspiration none))
    (surface (devanagari "ग") (iast "g") (cyrillic "ґ"))
    (gesture (base "ґ"))
    (status candidate))

  (sound-unit
    (id SA_GHA)
    (features (place velar) (manner stop) (voice voiced) (aspiration aspirated))
    (surface (devanagari "घ") (iast "gh") (cyrillic candidate))
    (gesture (dead-asp "ґ"))
    (status candidate))

  (sound-unit
    (id SA_NGA)
    (features (place velar) (manner nasal) (voice voiced))
    (surface (devanagari "ङ") (iast "ṅ") (cyrillic "ӈ"))
    (gesture (dead-vel "н"))
    (status candidate))

  ;; Row: CA (palatal)
  (sound-unit
    (id SA_CA)
    (features (place palatal) (manner stop) (voice voiceless) (aspiration none))
    (surface (devanagari "च") (iast "c") (cyrillic "ч"))
    (gesture (base "ч"))
    (status candidate))

  (sound-unit
    (id SA_CHA)
    (features (place palatal) (manner stop) (voice voiceless) (aspiration aspirated))
    (surface (devanagari "छ") (iast "ch") (cyrillic candidate))
    (gesture (dead-asp "ч"))
    (status candidate))

  (sound-unit
    (id SA_JA)
    (features (place palatal) (manner stop) (voice voiced) (aspiration none))
    (surface (devanagari "ज") (iast "j") (cyrillic "дж"))
    (gesture (base "дж"))
    (status candidate))

  (sound-unit
    (id SA_JHA)
    (features (place palatal) (manner stop) (voice voiced) (aspiration aspirated))
    (surface (devanagari "झ") (iast "jh") (cyrillic candidate))
    (gesture (dead-asp "дж"))
    (status candidate))

  (sound-unit
    (id SA_NYA)
    (features (place palatal) (manner nasal) (voice voiced))
    (surface (devanagari "ञ") (iast "ñ") (cyrillic "њ"))
    (gesture (dead-pal "н"))
    (status candidate))

  ;; Row: ṬA (retroflex)
  (sound-unit
    (id SA_TTA)
    (features (place retroflex) (manner stop) (voice voiceless) (aspiration none))
    (surface (devanagari "ट") (iast "ṭ") (cyrillic unresolved))
    (gesture (dead-retro "т"))
    (status unresolved))

  (sound-unit
    (id SA_TTHA)
    (features (place retroflex) (manner stop) (voice voiceless) (aspiration aspirated))
    (surface (devanagari "ठ") (iast "ṭh") (cyrillic unresolved))
    (gesture (dead-retro (dead-asp "т")))
    (status unresolved))

  (sound-unit
    (id SA_DDA)
    (features (place retroflex) (manner stop) (voice voiced) (aspiration none))
    (surface (devanagari "ड") (iast "ḍ") (cyrillic unresolved))
    (gesture (dead-retro "д"))
    (status unresolved))

  (sound-unit
    (id SA_DDHA)
    (features (place retroflex) (manner stop) (voice voiced) (aspiration aspirated))
    (surface (devanagari "ढ") (iast "ḍh") (cyrillic unresolved))
    (gesture (dead-retro (dead-asp "д")))
    (status unresolved))

  (sound-unit
    (id SA_NNA)
    (features (place retroflex) (manner nasal) (voice voiced))
    (surface (devanagari "ण") (iast "ṇ") (cyrillic unresolved))
    (gesture (dead-retro "н"))
    (status unresolved))

  ;; Row: TA (dental)
  (sound-unit
    (id SA_TA)
    (features (place dental) (manner stop) (voice voiceless) (aspiration none))
    (surface (devanagari "त") (iast "t") (cyrillic "т"))
    (gesture (base "т"))
    (status candidate))

  (sound-unit
    (id SA_THA)
    (features (place dental) (manner stop) (voice voiceless) (aspiration aspirated))
    (surface (devanagari "थ") (iast "th") (cyrillic candidate))
    (gesture (dead-asp "т"))
    (status candidate))

  (sound-unit
    (id SA_DA)
    (features (place dental) (manner stop) (voice voiced) (aspiration none))
    (surface (devanagari "द") (iast "d") (cyrillic "д"))
    (gesture (base "д"))
    (status candidate))

  (sound-unit
    (id SA_DHA)
    (features (place dental) (manner stop) (voice voiced) (aspiration aspirated))
    (surface (devanagari "ध") (iast "dh") (cyrillic candidate))
    (gesture (dead-asp "д"))
    (status candidate))

  (sound-unit
    (id SA_NA)
    (features (place dental) (manner nasal) (voice voiced))
    (surface (devanagari "न") (iast "n") (cyrillic "н"))
    (gesture (base "н"))
    (status candidate))

  ;; Row: PA (labial)
  (sound-unit
    (id SA_PA)
    (features (place labial) (manner stop) (voice voiceless) (aspiration none))
    (surface (devanagari "प") (iast "p") (cyrillic "п"))
    (gesture (base "п"))
    (status candidate))

  (sound-unit
    (id SA_PHA)
    (features (place labial) (manner stop) (voice voiceless) (aspiration aspirated))
    (surface (devanagari "फ") (iast "ph") (cyrillic candidate))
    (gesture (dead-asp "п"))
    (status candidate))

  (sound-unit
    (id SA_BA)
    (features (place labial) (manner stop) (voice voiced) (aspiration none))
    (surface (devanagari "ब") (iast "b") (cyrillic "б"))
    (gesture (base "б"))
    (status candidate))

  (sound-unit
    (id SA_BHA)
    (features (place labial) (manner stop) (voice voiced) (aspiration aspirated))
    (surface (devanagari "भ") (iast "bh") (cyrillic candidate))
    (gesture (dead-asp "б"))
    (status candidate))

  (sound-unit
    (id SA_MA)
    (features (place labial) (manner nasal) (voice voiced))
    (surface (devanagari "म") (iast "m") (cyrillic "м"))
    (gesture (base "м"))
    (status candidate))

  ;; ============================================================
  ;; SEMIVOWELS / LIQUIDS / FRICATIVES
  ;; ============================================================
  (sound-unit
    (id SA_YA)
    (features (place palatal) (manner approximant) (voice voiced))
    (surface (devanagari "य") (iast "y") (cyrillic "й"))
    (gesture (base "й"))
    (status candidate))

  (sound-unit
    (id SA_RA)
    (features (place alveolar) (manner trill) (voice voiced))
    (surface (devanagari "र") (iast "r") (cyrillic "р"))
    (gesture (base "р"))
    (status candidate))

  (sound-unit
    (id SA_LA)
    (features (place alveolar) (manner lateral) (voice voiced))
    (surface (devanagari "ल") (iast "l") (cyrillic "л"))
    (gesture (base "л"))
    (status candidate))

  (sound-unit
    (id SA_VA)
    (features (place labial) (manner approximant) (voice voiced))
    (surface (devanagari "व") (iast "v") (cyrillic "в"))
    (gesture (base "в"))
    (status candidate))

  (sound-unit
    (id SA_SHA)
    (features (place palatal) (manner fricative) (voice voiceless))
    (surface (devanagari "श") (iast "ś") (cyrillic candidate))
    (gesture (base "ш"))
    (status candidate))

  (sound-unit
    (id SA_SSA)
    (features (place retroflex) (manner fricative) (voice voiceless))
    (surface (devanagari "ष") (iast "ṣ") (cyrillic unresolved))
    (gesture (dead-retro "ш"))
    (status unresolved))

  (sound-unit
    (id SA_SA)
    (features (place dental) (manner fricative) (voice voiceless))
    (surface (devanagari "स") (iast "s") (cyrillic "с"))
    (gesture (base "с"))
    (status candidate))

  (sound-unit
    (id SA_HA)
    (features (place glottal) (manner fricative) (voice voiced))
    (surface (devanagari "ह") (iast "h") (cyrillic "г"))
    (gesture (base "г"))
    (status candidate))

  ;; ============================================================
  ;; SPECIAL UNITS
  ;; ============================================================
  (sound-unit
    (id SA_VISARGA)
    (features (manner visarga) (contextual true))
    (surface (devanagari "ः") (iast "ḥ") (cyrillic unresolved))
    (gesture (dead-visarga))
    (status unresolved))

  (sound-unit
    (id SA_ANUSVARA)
    (features (manner anusvara) (contextual true))
    (surface (devanagari "ं") (iast unresolved) (cyrillic unresolved))
    (gesture (dead-anusvara))
    (status unresolved))

  (sound-unit
    (id SA_CHANDRA_BINDU)
    (features (manner nasalization) (contextual true))
    (surface (devanagari "ँ") (iast unresolved) (cyrillic unresolved))
    (gesture (dead-candrabindu))
    (status unresolved))

  ;; ============================================================
  ;; DEAD KEY DEFINITIONS (gesture layer)
  ;; ============================================================
  (dead-key
    (id dead-long)
    (name "LONG")
    (description "Vowel lengthening"))

  (dead-key
    (id dead-asp)
    (name "ASP")
    (description "Aspiration"))

  (dead-key
    (id dead-retro)
    (name "RETRO")
    (description "Retroflex articulation"))

  (dead-key
    (id dead-pal)
    (name "PAL")
    (description "Palatal articulation"))

  (dead-key
    (id dead-vel)
    (name "VEL")
    (description "Velar nasal"))

  (dead-key
    (id dead-visarga)
    (name "VISARGA")
    (description "Visarga"))

  (dead-key
    (id dead-anusvara)
    (name "ANUSVARA")
    (description "Anusvara"))

  (dead-key
    (id dead-candrabindu)
    (name "CANDRABINDU")
    (description "Candrabindu"))
)
