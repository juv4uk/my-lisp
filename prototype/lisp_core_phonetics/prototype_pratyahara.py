"""
64-Bit Pratyāhāra Bitmask Engine for My-Lisp Core Runtime
=========================================================

Epistemic Layer: Layer 6 (Engineering & Runtime Model)
Status: Prototype / Proposed Language Core Extension
Reference: hypotheses/shabda/status.yaml#H2, ADR-002

Mathematical Principles:
- Canonical sounds span exactly 42 unique codes (0x00 to 0x29).
- Any pratyāhāra S ⊆ {0, ..., 41} is encoded as a single 64-bit unsigned integer mask:
      Mask(S) = ∑_{c ∈ S} 2^c
- Membership test is O(1) in a single CPU/FPGA clock cycle:
      is_member(sound_code, mask) = ((1 << sound_code) & mask) != 0
- Set algebra operations are single bitwise ALU instructions:
      S₁ ∩ S₂  ->  mask1 & mask2
      S₁ ∪ S₂  ->  mask1 | mask2
      S₁ \\ S₂  ->  mask1 & ~mask2
      S₁ ⊆ S₂  ->  (mask1 & ~mask2) == 0
"""

from typing import Dict, List, Set, Tuple, Union, Optional

# 42 Canonical Sounds of the Śiva Sūtras in structural sequence order
CANONICAL_SOUNDS: List[str] = [
    # Sutra 1-4: Vowels (ac) -> bits 0..8
    "a", "i", "u", "f", "x", "e", "o", "E", "O",
    # Sutra 5-6: Semivowels + h -> bits 9..13
    "h", "y", "v", "r", "l",
    # Sutra 7: Nasals (ña-ma-ṅa-ṇa-na) -> bits 14..18
    "Y", "m", "N", "R", "n",
    # Sutra 8-9: Voiced Aspirated Stops -> bits 19..23
    "J", "B", "G", "Q", "D",
    # Sutra 10: Voiced Unaspirated Stops -> bits 24..28
    "j", "b", "g", "q", "d",
    # Sutra 11: Voiceless Aspirated Stops + Voiceless Palatal/Retroflex/Dental -> bits 29..36
    "K", "P", "C", "W", "T", "c", "w", "t",
    # Sutra 12: Voiceless Velar/Labial Stops -> bits 37..38
    "k", "p",
    # Sutra 13: Sibilants (Sar) -> bits 39..41
    "S", "z", "s",
]

assert len(CANONICAL_SOUNDS) == 42, f"Expected 42 unique canonical sounds, got {len(CANONICAL_SOUNDS)}"

# Mappings
SOUND_TO_CODE: Dict[str, int] = {s: i for i, s in enumerate(CANONICAL_SOUNDS)}
CODE_TO_SOUND: Dict[int, str] = {i: s for i, s in enumerate(CANONICAL_SOUNDS)}

# 42 Classical Pāṇinian Pratyāhāras
PRATYAHARA_DEFINITIONS: Dict[str, List[str]] = {
    "ac":   ["a", "i", "u", "f", "x", "e", "o", "E", "O"],
    "ak":   ["a", "i", "u", "f", "x"],
    "ik":   ["i", "u", "f", "x"],
    "uk":   ["u", "f", "x"],
    "eN":   ["e", "o"],
    "ec":   ["e", "o", "E", "O"],
    "Ec":   ["E", "O"],
    "al":   CANONICAL_SOUNDS,  # All 42 sounds
    "hal":  CANONICAL_SOUNDS[9:],  # 33 consonants
    "val":  CANONICAL_SOUNDS[10:], # All consonants except h (32)
    "ral":  CANONICAL_SOUNDS[12:], # From r onwards (30)
    "Jal":  ["J", "B", "G", "Q", "D", "j", "b", "g", "q", "d",
             "K", "P", "C", "W", "T", "c", "w", "t", "k", "p", "S", "z", "s", "h"],
    "Sal":  ["S", "z", "s", "h"],
    "Sar":  ["S", "z", "s"],
    "yar":  CANONICAL_SOUNDS[10:42],
    "yay":  ["y", "v", "r", "l", "Y", "m", "N", "R", "n", "J", "B", "G", "Q", "D",
             "j", "b", "g", "q", "d", "K", "P", "C", "W", "T", "c", "w", "t", "k", "p"],
    "yaR":  ["y", "v", "r", "l"],
    "yam":  ["y", "v", "r", "l", "Y", "m", "N", "R", "n"],
    "yaY":  ["y", "v", "r", "l", "Y", "m", "N", "R", "n", "J", "B"],
    "vaw":  ["v", "r"],
    "may":  ["m", "N", "R", "n", "J", "B", "G", "Q", "D", "j", "b", "g", "q", "d",
             "K", "P", "C", "W", "T", "c", "w", "t", "k", "p"],
    "am":   ["a", "i", "u", "f", "x", "e", "o", "E", "O", "h", "y", "v", "r", "l",
             "Y", "m", "N", "R", "n"],
    "aw":   ["a", "i", "u", "f", "x", "e", "o", "E", "O", "h", "y", "v", "r"],
    "iR":   ["i", "u", "f", "x", "e", "o", "E", "O", "h", "y", "v", "r", "l"],
    "aR":   ["a", "i", "u"],
    "eR":   ["e", "o"],
    "nam":  ["N", "R", "n"],
    "JaS":  ["J", "B", "G", "Q", "D", "j", "b", "g", "q", "d"],
    "jaS":  ["j", "b", "g", "q", "d"],
    "baS":  ["b", "g", "q", "d"],
    "Jaz":  ["J", "B", "G", "Q", "D"],
    "Baz":  ["B", "G", "Q", "D"],
    "Jay":  ["J", "B", "G", "Q", "D", "j", "b", "g", "q", "d", "K", "P", "C", "W",
             "T", "c", "w", "t", "k", "p"],
    "Kay":  ["K", "P", "C", "W", "T", "c", "w", "t", "k", "p"],
    "xay":  ["K", "P", "C", "W", "T", "c", "w", "t"],
    "car":  ["c", "w", "t", "k", "p", "S", "z", "s"],
    "cav":  ["c", "w", "t"],
    "caw":  ["c", "w"],
    "Kar":  ["K", "P", "C", "W", "T", "c", "w", "t", "k", "p", "S", "z", "s"],
    "Jar":  ["J", "B", "G", "Q", "D", "j", "b", "g", "q", "d", "K", "P", "C", "W",
             "T", "c", "w", "t", "k", "p", "S", "z", "s"],
    "haS":  ["h", "y", "v", "r", "l", "Y", "m", "N", "R", "n", "J", "B", "G", "Q",
             "D", "j", "b", "g", "q", "d"],
}

def sounds_to_mask(sounds: List[str]) -> int:
    """Compile list of sound symbols to a 64-bit bitmask."""
    mask = 0
    for s in sounds:
        if s in SOUND_TO_CODE:
            mask |= (1 << SOUND_TO_CODE[s])
    return mask

# Precomputed 64-bit masks
PRATYAHARA_MASKS: Dict[str, int] = {
    name: sounds_to_mask(sounds) for name, sounds in PRATYAHARA_DEFINITIONS.items()
}


def sound_code(sound: Union[str, int]) -> int:
    """Get integer sound code (0..41) from SLP1 symbol or integer."""
    if isinstance(sound, int):
        if 0 <= sound < 42:
            return sound
        raise ValueError(f"Sound code {sound} out of range (0..41)")
    if sound in SOUND_TO_CODE:
        return SOUND_TO_CODE[sound]
    raise KeyError(f"Unknown canonical sound: {sound}")


def prat_mask(name_or_sounds: Union[str, List[str], int]) -> int:
    """Retrieve or compute 64-bit mask for pratyāhāra name, sound list, or mask."""
    if isinstance(name_or_sounds, int):
        return name_or_sounds & 0xFFFFFFFFFFFFFFFF
    if isinstance(name_or_sounds, str):
        if name_or_sounds in PRATYAHARA_MASKS:
            return PRATYAHARA_MASKS[name_or_sounds]
        raise KeyError(f"Unknown pratyāhāra: {name_or_sounds}")
    if isinstance(name_or_sounds, list):
        return sounds_to_mask(name_or_sounds)
    raise TypeError(f"Invalid pratyāhāra specifier: {name_or_sounds}")


def prat_member(sound: Union[str, int], mask_or_name: Union[int, str]) -> bool:
    """
    O(1) Single-Cycle Pratyāhāra Membership Test:
    ((1 << sound_code) & mask_64) != 0
    """
    try:
        c = sound_code(sound)
        m = prat_mask(mask_or_name) if isinstance(mask_or_name, str) else mask_or_name
        return bool((1 << c) & m)
    except (KeyError, ValueError):
        return False


def prat_intersect(m1: int, m2: int) -> int:
    """Set intersection: S₁ ∩ S₂."""
    return (m1 & m2) & 0xFFFFFFFFFFFFFFFF


def prat_union(m1: int, m2: int) -> int:
    """Set union: S₁ ∪ S₂."""
    return (m1 | m2) & 0xFFFFFFFFFFFFFFFF


def prat_diff(m1: int, m2: int) -> int:
    """Set difference: S₁ \\ S₂."""
    return (m1 & ~m2) & 0xFFFFFFFFFFFFFFFF


def prat_subset(m1: int, m2: int) -> bool:
    """Subset test: True if S₁ ⊆ S₂."""
    return ((m1 & ~m2) & 0xFFFFFFFFFFFFFFFF) == 0


def prat_sounds(mask: int) -> List[str]:
    """Decompile a 64-bit mask into a list of SLP1 sound symbols."""
    return [CODE_TO_SOUND[bit] for bit in range(42) if (mask & (1 << bit))]
