"""
PVC-16: 16-Bit Unboxed Phonetic Vector Representation for My-Lisp Core Runtime
=============================================================================

Epistemic Layer: Layer 6 (Engineering & Runtime Model)
Status: Prototype / Proposed Language Core Extension
Reference: language-contract.my, ADR-002

Memory Layout (16-bit Unboxed Bitfield):
  Bit [0]:     Vowel Flag (FLAG_VOWEL) -> 1 = Vowel (ac), 0 = Consonant (hal)
  Bits [5:1]:  Sthāna (Place of Articulation)
                 00000: None
                 00001: Kaṇṭhya (Velar / Guttural: a, k, kh, g, gh, ṅ, h)
                 00010: Tālavya (Palatal: i, c, ch, j, jh, ñ, y, ś)
                 00011: Mūrdhanya (Retroflex: ṛ, ṭ, ṭh, ḍ, ḍh, ṇ, r, ṣ)
                 00100: Dantya (Dental: ḷ, t, th, d, dh, n, l, s)
                 00101: Oṣṭhya (Labial: u, p, ph, b, bh, m, v)
  Bits [9:6]:  Prayatna (Manner of Articulation)
                 Bit 6: Spṛṣṭa (Stop / Plosive: k, c, ṭ, t, p, ...)
                 Bit 7: Mahāprāṇa (Aspirated: kh, gh, ch, jh, ...)
                 Bit 8: Ghoṣa (Voiced: g, gh, j, jh, d, dh, b, bh, nasals, semivowels, vowels)
                 Bit 9: Anunāsika (Nasal: ṅ, ñ, ṇ, n, m)
  Bits [13:10]: Length / Svara
                 0001: Hrasva (Short)
                 0010: Dīrgha (Long)
                 0011: Pluta (Prolated)
  Bits [15:14]: Modifiers
                 Bit 14: Palatalized (Ukrainian [ь] / Palatalized consonant)
                 Bit 15: Diphthong (Sandhyakṣara: e, ai, o, au)
"""

from dataclasses import dataclass
from typing import Optional, Dict, Any, Union

# Bit 0: Vowel Flag
FLAG_VOWEL: int = 1 << 0

# Bits [5:1]: Sthāna (Place of Articulation)
STHANA_SHIFT: int = 1
STHANA_MASK: int = 0x1F << STHANA_SHIFT  # 0x003E

STHANA_NONE: int = 0 << STHANA_SHIFT
STHANA_KANTHYA: int = 1 << STHANA_SHIFT    # Velar / Guttural (a, k, kh, g, gh, nG, h)
STHANA_TALAVYA: int = 2 << STHANA_SHIFT    # Palatal (i, c, ch, j, jh, nY, y, S)
STHANA_MURDHANYA: int = 3 << STHANA_SHIFT  # Retroflex (f, T, W, D, Q, R, r, z)
STHANA_DANTYA: int = 4 << STHANA_SHIFT     # Dental (x, t, th, d, dh, n, l, s)
STHANA_OSHTHYA: int = 5 << STHANA_SHIFT    # Labial (u, p, ph, b, bh, m, v)

STHANA_MAP = {
    "none": STHANA_NONE,
    "kanthya": STHANA_KANTHYA,
    "talavya": STHANA_TALAVYA,
    "murdhanya": STHANA_MURDHANYA,
    "dantya": STHANA_DANTYA,
    "oshthya": STHANA_OSHTHYA,
    "velar": STHANA_KANTHYA,
    "palatal": STHANA_TALAVYA,
    "retroflex": STHANA_MURDHANYA,
    "dental": STHANA_DANTYA,
    "labial": STHANA_OSHTHYA,
}

# Bits [9:6]: Prayatna (Manner of Articulation)
PRAYATNA_SPRSTA: int = 1 << 6      # Stop (k, c, T, t, p...)
PRAYATNA_MAHAPRANA: int = 1 << 7   # Aspirate (kh, gh, ch, jh...)
PRAYATNA_GHOSHA: int = 1 << 8      # Voiced (g, gh, j, jh, d, dh, b, bh, nasals, semivowels, vowels)
PRAYATNA_ANUNASIKA: int = 1 << 9   # Nasal (nG, nY, N, n, m)
PRAYATNA_MASK: int = 0xF << 6      # 0x03C0

# Bits [13:10]: Length / Svara
LEN_SHIFT: int = 10
LEN_HRASVA: int = 1 << LEN_SHIFT   # Short
LEN_DIRGHA: int = 2 << LEN_SHIFT   # Long
LEN_PLUTA: int = 3 << LEN_SHIFT    # Prolated
LEN_MASK: int = 0xF << LEN_SHIFT   # 0x3C00

# Bits [15:14]: Modifiers
MOD_PALATALIZED: int = 1 << 14     # Ukrainian [ь] / Palatalized consonant
MOD_DIPHTHONG: int = 1 << 15       # Diphthong
MOD_MASK: int = 0x3 << 14          # 0xC000


@dataclass(frozen=True)
class PhoneticVector:
    """16-bit unboxed phonetic vector structure."""
    code: int
    symbol: Optional[str] = None

    def __post_init__(self):
        object.__setattr__(self, "code", int(self.code) & 0xFFFF)

    @property
    def is_vowel(self) -> bool:
        """True if vowel (ac), False if consonant (hal)."""
        return bool(self.code & FLAG_VOWEL)

    @property
    def is_consonant(self) -> bool:
        """True if consonant (hal)."""
        return not self.is_vowel

    @property
    def sthana(self) -> int:
        """Place of articulation index (1..5)."""
        return (self.code & STHANA_MASK) >> STHANA_SHIFT

    @property
    def sthana_name(self) -> str:
        s = self.sthana
        return {
            1: "kanthya",
            2: "talavya",
            3: "murdhanya",
            4: "dantya",
            5: "oshthya",
        }.get(s, "none")

    @property
    def prayatna(self) -> int:
        """Raw prayatna bits (manner of articulation)."""
        return (self.code & PRAYATNA_MASK) >> 6

    @property
    def is_sprsta(self) -> bool:
        """True if stop consonant (spṛṣṭa)."""
        return bool(self.code & PRAYATNA_SPRSTA)

    @property
    def is_aspirate(self) -> bool:
        """True if aspirated (mahāprāṇa)."""
        return bool(self.code & PRAYATNA_MAHAPRANA)

    @property
    def is_voiced(self) -> bool:
        """True if voiced (ghoṣa)."""
        return bool(self.code & PRAYATNA_GHOSHA)

    @property
    def is_nasal(self) -> bool:
        """True if nasal (anunāsika)."""
        return bool(self.code & PRAYATNA_ANUNASIKA)

    @property
    def length(self) -> int:
        """Vowel length (1 = hrasva, 2 = dīrgha, 3 = pluta)."""
        return (self.code & LEN_MASK) >> LEN_SHIFT

    @property
    def is_palatalized(self) -> bool:
        """True if palatalized modifier bit is set."""
        return bool(self.code & MOD_PALATALIZED)

    @property
    def is_diphthong(self) -> bool:
        """True if diphthong modifier bit is set."""
        return bool(self.code & MOD_DIPHTHONG)

    def is_savarna_with(self, other: "PhoneticVector") -> bool:
        """
        Sūtra 1.1.9: tulyāsyaprayatnaṁ savarṇam.
        Single-cycle hardware-synthesizable homogeneity check.
        Two sounds are savarṇa if they share:
          1. Identical place of articulation (sthāna): (a & 0x003E) == (b & 0x003E) != 0
          2. Identical primary prayatna & vowel category: (a & 0x0041) == (b & 0x0041)
        """
        # Place equality (non-zero)
        sthana_a = self.code & STHANA_MASK
        sthana_b = other.code & STHANA_MASK
        if sthana_a == 0 or sthana_a != sthana_b:
            return False

        # Primary internal effort equality (spṛṣṭa bit and vowel flag)
        primary_a = self.code & (PRAYATNA_SPRSTA | FLAG_VOWEL)
        primary_b = other.code & (PRAYATNA_SPRSTA | FLAG_VOWEL)
        return primary_a == primary_b

    def with_voicing(self, voiced: bool = True) -> "PhoneticVector":
        """Single-cycle bitwise voicing transformation (Ghoṣa sandhi)."""
        new_code = (self.code | PRAYATNA_GHOSHA) if voiced else (self.code & ~PRAYATNA_GHOSHA)
        return PhoneticVector(new_code, self.symbol)

    def with_palatalization(self, palatalized: bool = True) -> "PhoneticVector":
        """Single-cycle bitwise palatalization transformation (Ukrainian ь)."""
        new_code = (self.code | MOD_PALATALIZED) if palatalized else (self.code & ~MOD_PALATALIZED)
        return PhoneticVector(new_code, self.symbol)

    def __int__(self) -> int:
        return self.code

    def __repr__(self) -> str:
        sym = f"'{self.symbol}'" if self.symbol else f"0x{self.code:04X}"
        return f"#pvc({sym} :0x{self.code:04X})"


def pvc_make(
    vowel: bool = False,
    sthana: Union[int, str] = 0,
    prayatna: int = 0,
    length: int = 0,
    modifier: int = 0,
    symbol: Optional[str] = None,
) -> PhoneticVector:
    """Constructor for 16-bit PhoneticVector from discrete feature fields."""
    code = 0
    if vowel:
        code |= FLAG_VOWEL

    if isinstance(sthana, str):
        sthana_val = STHANA_MAP.get(sthana.lower(), 0)
        code |= sthana_val
    elif isinstance(sthana, int):
        code |= (sthana & 0x1F) << STHANA_SHIFT

    if prayatna:
        if prayatna < 16:
            code |= (prayatna & 0xF) << 6
        else:
            code |= (prayatna & PRAYATNA_MASK)

    if length:
        if length < 16:
            code |= (length & 0xF) << LEN_SHIFT
        else:
            code |= (length & LEN_MASK)

    if modifier:
        if modifier < 4:
            code |= (modifier & 0x3) << 14
        else:
            code |= (modifier & MOD_MASK)

    return PhoneticVector(code=code, symbol=symbol)


# Canonical Sanskrit and Ukrainian Phoneme Registry in PVC-16
REGISTRY: Dict[str, PhoneticVector] = {
    # Vowels (ac)
    "a": PhoneticVector(FLAG_VOWEL | STHANA_KANTHYA | PRAYATNA_GHOSHA | LEN_HRASVA, "a"),
    "A": PhoneticVector(FLAG_VOWEL | STHANA_KANTHYA | PRAYATNA_GHOSHA | LEN_DIRGHA, "A"),
    "i": PhoneticVector(FLAG_VOWEL | STHANA_TALAVYA | PRAYATNA_GHOSHA | LEN_HRASVA, "i"),
    "I": PhoneticVector(FLAG_VOWEL | STHANA_TALAVYA | PRAYATNA_GHOSHA | LEN_DIRGHA, "I"),
    "u": PhoneticVector(FLAG_VOWEL | STHANA_OSHTHYA | PRAYATNA_GHOSHA | LEN_HRASVA, "u"),
    "U": PhoneticVector(FLAG_VOWEL | STHANA_OSHTHYA | PRAYATNA_GHOSHA | LEN_DIRGHA, "U"),
    "f": PhoneticVector(FLAG_VOWEL | STHANA_MURDHANYA | PRAYATNA_GHOSHA | LEN_HRASVA, "f"),  # ṛ
    "F": PhoneticVector(FLAG_VOWEL | STHANA_MURDHANYA | PRAYATNA_GHOSHA | LEN_DIRGHA, "F"),  # ṝ
    "x": PhoneticVector(FLAG_VOWEL | STHANA_DANTYA | PRAYATNA_GHOSHA | LEN_HRASVA, "x"),     # ḷ
    "X": PhoneticVector(FLAG_VOWEL | STHANA_DANTYA | PRAYATNA_GHOSHA | LEN_DIRGHA, "X"),     # ḹ
    "e": PhoneticVector(FLAG_VOWEL | STHANA_TALAVYA | PRAYATNA_GHOSHA | LEN_DIRGHA | MOD_DIPHTHONG, "e"),
    "o": PhoneticVector(FLAG_VOWEL | STHANA_OSHTHYA | PRAYATNA_GHOSHA | LEN_DIRGHA | MOD_DIPHTHONG, "o"),
    "E": PhoneticVector(FLAG_VOWEL | STHANA_TALAVYA | PRAYATNA_GHOSHA | LEN_DIRGHA | MOD_DIPHTHONG, "E"),  # ai
    "O": PhoneticVector(FLAG_VOWEL | STHANA_OSHTHYA | PRAYATNA_GHOSHA | LEN_DIRGHA | MOD_DIPHTHONG, "O"),  # au

    # Consonants - Kavarga (Guttural / Velar)
    "k": PhoneticVector(STHANA_KANTHYA | PRAYATNA_SPRSTA, "k"),
    "K": PhoneticVector(STHANA_KANTHYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA, "K"),
    "g": PhoneticVector(STHANA_KANTHYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA, "g"),
    "G": PhoneticVector(STHANA_KANTHYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA | PRAYATNA_GHOSHA, "G"),
    "N": PhoneticVector(STHANA_KANTHYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA | PRAYATNA_ANUNASIKA, "N"),

    # Consonants - Cavarga (Palatal)
    "c": PhoneticVector(STHANA_TALAVYA | PRAYATNA_SPRSTA, "c"),
    "C": PhoneticVector(STHANA_TALAVYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA, "C"),
    "j": PhoneticVector(STHANA_TALAVYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA, "j"),
    "J": PhoneticVector(STHANA_TALAVYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA | PRAYATNA_GHOSHA, "J"),
    "Y": PhoneticVector(STHANA_TALAVYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA | PRAYATNA_ANUNASIKA, "Y"),

    # Consonants - Ṭavarga (Retroflex)
    "w": PhoneticVector(STHANA_MURDHANYA | PRAYATNA_SPRSTA, "w"),
    "W": PhoneticVector(STHANA_MURDHANYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA, "W"),
    "q": PhoneticVector(STHANA_MURDHANYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA, "q"),
    "Q": PhoneticVector(STHANA_MURDHANYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA | PRAYATNA_GHOSHA, "Q"),
    "R": PhoneticVector(STHANA_MURDHANYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA | PRAYATNA_ANUNASIKA, "R"),

    # Consonants - Tavarga (Dental)
    "t": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA, "t"),
    "T": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA, "T"),
    "d": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA, "d"),
    "D": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA | PRAYATNA_GHOSHA, "D"),
    "n": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA | PRAYATNA_ANUNASIKA, "n"),

    # Consonants - Pavarga (Labial)
    "p": PhoneticVector(STHANA_OSHTHYA | PRAYATNA_SPRSTA, "p"),
    "P": PhoneticVector(STHANA_OSHTHYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA, "P"),
    "b": PhoneticVector(STHANA_OSHTHYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA, "b"),
    "B": PhoneticVector(STHANA_OSHTHYA | PRAYATNA_SPRSTA | PRAYATNA_MAHAPRANA | PRAYATNA_GHOSHA, "B"),
    "m": PhoneticVector(STHANA_OSHTHYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA | PRAYATNA_ANUNASIKA, "m"),

    # Semivowels (Antastha)
    "y": PhoneticVector(STHANA_TALAVYA | PRAYATNA_GHOSHA, "y"),
    "r": PhoneticVector(STHANA_MURDHANYA | PRAYATNA_GHOSHA, "r"),
    "l": PhoneticVector(STHANA_DANTYA | PRAYATNA_GHOSHA, "l"),
    "v": PhoneticVector(STHANA_OSHTHYA | PRAYATNA_GHOSHA, "v"),

    # Sibilants & Aspirate (Ūṣman)
    "S": PhoneticVector(STHANA_TALAVYA, "S"),   # ś
    "z": PhoneticVector(STHANA_MURDHANYA, "z"), # ṣ
    "s": PhoneticVector(STHANA_DANTYA, "s"),    # s
    "h": PhoneticVector(STHANA_KANTHYA | PRAYATNA_GHOSHA, "h"),

    # Ukrainian Extensions
    "т'": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA | MOD_PALATALIZED, "т'"),
    "д'": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA | MOD_PALATALIZED, "д'"),
    "н'": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA | PRAYATNA_GHOSHA | PRAYATNA_ANUNASIKA | MOD_PALATALIZED, "н'"),
    "л'": PhoneticVector(STHANA_DANTYA | PRAYATNA_GHOSHA | MOD_PALATALIZED, "л'"),
    "с'": PhoneticVector(STHANA_DANTYA | MOD_PALATALIZED, "с'"),
    "з'": PhoneticVector(STHANA_DANTYA | PRAYATNA_GHOSHA | MOD_PALATALIZED, "з'"),
    "ц'": PhoneticVector(STHANA_DANTYA | PRAYATNA_SPRSTA | MOD_PALATALIZED, "ц'"),
}


def get_phoneme(sym: str) -> Optional[PhoneticVector]:
    """Lookup phoneme vector by SLP1 or Ukrainian symbol."""
    return REGISTRY.get(sym)
