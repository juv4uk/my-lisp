# My-Lisp Core Phonetics Runtime & Semantics Prototype

**Epistemic Layer:** Layer 6 (Engineering / Runtime Architecture & Semantics)  
**Status:** Experimental Prototype & Architecture Proposal  
**Target Repository:** `my-lisp` (Core Runtime & S-Expression Knowledge Base)  

---

## 1. Architectural Overview

This prototype integrates high-performance **unboxed phonetic vector representations** and **single-cycle phonological evaluation primitives** into the My-Lisp runtime:

1. **Unboxed 16-Bit Phonetic Vector Code (PVC-16):** Compact bitfield layout embedding place of articulation (*Sthāna*), manner of articulation (*Prayatna*), vowel status, length (*Svara*), and cross-linguistic modifiers (Ukrainian palatalization).
2. **Sūtra 1.1.9 Savarṇa Homogeneity Engine:** Fast $O(1)$ equality check verifying *tulyāsyaprayatnaṁ savarṇam* via combinational bitwise logic: `((p1 & 0x003E) == (p2 & 0x003E) && (p1 & 0x003E) != 0) && ((p1 & 0x0041) == (p2 & 0x0041))`.
3. **64-Bit Pratyāhāra Bitmask Engine:** 42 canonical sounds encoded into 64-bit integer masks, providing single-cycle class membership tests `((1 << sound_code) & mask) != 0` and bitwise set algebra.
4. **Instant Bitwise Transformations:** Direct bit-flipping primitives for Sandhi voicing (`sandhi-voice`) and palatalization (`palatalize`).
5. **Reader Macro Extensions:** First-class `#pvc(...)` and `#prat(...)` reader syntax for compile-time constant expansion.
6. **S-Expression Knowledge Base:** Declarative `.my` knowledge base integrating phonetic facts, Sūtras, and inference rules into the immutable World model.

---

## 2. Memory Layout & NaN-Boxing Integration

In the My-Lisp NaN-boxing runtime (`crates/my-lisp/src/layout.rs`):
```text
Bit 63:       Sign bit = 0
Bits 62..52:  Quiet NaN Exponent (0x7FF)
Bits 51..32:  Upper Pointer / Extra Payload (0x00000)
Bits 31..28:  Type Tag = 0xC (TAG_PHONETIC_VECTOR = 12)
Bits 27..16:  Reserved / Segment ID metadata
Bits 15..0:   16-Bit Unboxed PVC-16 Vector Payload
```

### PVC-16 Bit Allocation Table:
| Bits | Name | Description / Values |
|---|---|---|
| `[0]` | `FLAG_VOWEL` | `1` = Vowel (*ac*), `0` = Consonant (*hal*) |
| `[5:1]` | `STHANA` | Place of Articulation: `1`=Kaṇṭhya (Velar), `2`=Tālavya (Palatal), `3`=Mūrdhanya (Retroflex), `4`=Dantya (Dental), `5`=Oṣṭhya (Labial) |
| `[6]` | `PRAYATNA_SPRSTA` | Stop / Plosive consonant |
| `[7]` | `PRAYATNA_MAHAPRANA` | Aspirated consonant |
| `[8]` | `PRAYATNA_GHOSHA` | Voiced sound |
| `[9]` | `PRAYATNA_ANUNASIKA`| Nasal sound |
| `[13:10]`| `LENGTH` | `1`=Hrasva (Short), `2`=Dīrgha (Long), `3`=Pluta (Prolated) |
| `[14]` | `MOD_PALATALIZED` | Ukrainian [ь] / Palatalized consonant modifier |
| `[15]` | `MOD_DIPHTHONG` | Diphthong (Sandhyakṣara: *e, ai, o, au*) |

---

## 3. Lisp Built-in Primitives

### 3.1 Phonetic Vector Construction & Inspection
- `(pvc-make :vowel bool :sthana int :prayatna int :length int :modifier int)`
- `(pvc-vowel? p)` $\to$ `t` / `()`
- `(pvc-sthana p)` $\to$ integer `1..5`
- `(pvc-sthana-name p)` $\to$ symbol `'kanthya`, `'talavya`, etc.
- `(pvc-prayatna p)` $\to$ integer bitfield
- `(pvc-voiced? p)` $\to$ `t` / `()`
- `(pvc-sprsta? p)` $\to$ `t` / `()`
- `(pvc-palatalized? p)` $\to$ `t` / `()`

### 3.2 Sūtra 1.1.9 Savarṇa Homogeneity
- `(savarna? p1 p2)` $\to$ `t` if $p_1$ and $p_2$ share identical place and primary effort.

### 3.3 64-Bit Pratyāhāra Membership & Set Algebra
- `(prat-member? sound-code mask-64)` $\to$ `t` / `()` (single clock cycle)
- `(prat-mask 'ac)` $\to$ `#x00000000000001FF`
- `(prat-intersect m1 m2)` $\to$ bitwise AND
- `(prat-union m1 m2)` $\to$ bitwise OR
- `(prat-diff m1 m2)` $\to$ bitwise difference ($m_1 \land \neg m_2$)
- `(prat-subset? m1 m2)` $\to$ subset inclusion predicate

### 3.4 Instant Bitwise Transformations
- `(sandhi-voice sound)` $\to$ sets bit 8 (`0x0100`), converting e.g. `k` to `g`
- `(sandhi-devoice sound)` $\to$ clears bit 8, converting e.g. `g` to `k`
- `(palatalize sound)` $\to$ sets bit 14 (`0x4000`), converting e.g. `t` to `t'`
- `(depalatalize sound)` $\to$ clears bit 14

---

## 4. S-Expression Reader Macros

The My-Lisp parser expands reader macros at parse time:
```lisp
;; Unboxed PVC-16 literal
(savarna? #pvc("a") #pvc("A"))        ; -> t
(savarna? #pvc(k) #pvc(K))            ; -> t
(savarna? #pvc(k) #pvc(t))            ; -> ()

;; Direct 64-bit pratyāhāra constant
(prat-member? (quote a) #prat(ac))    ; -> t
(prat-member? (quote k) #prat(ac))    ; -> ()
(prat-member? (quote k) #prat(hal))   ; -> t
```

---

## 5. Test Suite Verification

Run the unit test suite:
```bash
python3 prototype_test_lisp_phonetics.py
```
Test suite validates all 7 categories: construction, feature accessors, Sūtra 1.1.9 savarṇa checks, 64-bit pratyāhāra membership, set algebra, Sandhi bitwise transforms, and reader macros.
