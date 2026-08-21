#!/usr/bin/env python3
"""
Unit Test Suite for My-Lisp Core Phonetics Runtime & Semantics
==============================================================

Tests unboxed 16-bit Phonetic Vector Code (PVC-16), 64-bit Pratyāhāra Bitmask Engine,
Lisp built-in primitives (`pvc-make`, `savarna?`, `prat-member?`, `sandhi-voice`, `palatalize`),
and reader macros `#pvc(...)` and `#prat(...)`.
"""

import unittest
from prototype_pvc16 import (
    PhoneticVector, pvc_make, get_phoneme, REGISTRY,
    STHANA_KANTHYA, STHANA_TALAVYA, STHANA_DANTYA, STHANA_OSHTHYA,
    PRAYATNA_SPRSTA, PRAYATNA_GHOSHA, MOD_PALATALIZED,
)
from prototype_pratyahara import (
    prat_member, prat_mask, prat_intersect, prat_union, prat_diff, prat_subset,
    prat_sounds, sound_code, CANONICAL_SOUNDS, PRATYAHARA_MASKS,
)
from prototype_lisp_runtime import (
    create_global_env, eval_string, parse, evaluate, Symbol, Keyword,
)


class TestLispPhonetics(unittest.TestCase):

    def setUp(self):
        self.env = create_global_env()

    # ------------------------------------------------------------------------
    # 1. Phonetic Vector Construction & Field Inspection
    # ------------------------------------------------------------------------
    def test_pvc_make_keywords(self):
        """Test (pvc-make :vowel t :sthana 1 :prayatna 256 :length 1 :modifier 0)"""
        code = '(pvc-make :vowel t :sthana 1 :prayatna 256 :length 1 :modifier 0)'
        p = eval_string(code, self.env)
        self.assertIsInstance(p, PhoneticVector)
        self.assertTrue(p.is_vowel)
        self.assertEqual(p.sthana, 1)
        self.assertEqual(p.length, 1)
        self.assertTrue(p.is_voiced)

    def test_pvc_make_positional(self):
        """Test positional (pvc-make vowel sthana prayatna length modifier)"""
        code = '(pvc-make t 2 256 2 0)'
        p = eval_string(code, self.env)
        self.assertIsInstance(p, PhoneticVector)
        self.assertTrue(p.is_vowel)
        self.assertEqual(p.sthana, 2)
        self.assertEqual(p.length, 2)

    def test_pvc_field_accessors(self):
        """Test Lisp accessor primitives: pvc-vowel?, pvc-sthana, pvc-prayatna, pvc-voiced?"""
        eval_string('(def a (pvc-from-sym "a"))', self.env)
        eval_string('(def k (pvc-from-sym "k"))', self.env)

        self.assertTrue(eval_string('(pvc-vowel? a)', self.env))
        self.assertFalse(eval_string('(pvc-vowel? k)', self.env))
        self.assertTrue(eval_string('(pvc-consonant? k)', self.env))
        self.assertEqual(eval_string('(pvc-sthana a)', self.env), 1)
        self.assertEqual(eval_string('(pvc-sthana k)', self.env), 1)
        self.assertTrue(eval_string('(pvc-voiced? a)', self.env))
        self.assertFalse(eval_string('(pvc-voiced? k)', self.env))
        self.assertTrue(eval_string('(pvc-sprsta? k)', self.env))

    # ------------------------------------------------------------------------
    # 2. Sūtra 1.1.9 Savarṇa Homogeneity Check
    # ------------------------------------------------------------------------
    def test_savarna_homogeneity_vowels(self):
        """Sūtra 1.1.9: a and ā, i and ī are savarṇa; a and i are not."""
        self.assertTrue(eval_string('(savarna? (pvc-from-sym "a") (pvc-from-sym "A"))', self.env))
        self.assertTrue(eval_string('(savarna? (pvc-from-sym "i") (pvc-from-sym "I"))', self.env))
        self.assertTrue(eval_string('(savarna? (pvc-from-sym "u") (pvc-from-sym "U"))', self.env))
        self.assertFalse(eval_string('(savarna? (pvc-from-sym "a") (pvc-from-sym "i"))', self.env))
        self.assertFalse(eval_string('(savarna? (pvc-from-sym "i") (pvc-from-sym "u"))', self.env))

    def test_savarna_homogeneity_consonants(self):
        """Sūtra 1.1.9: consonants in same varga (k, kh, g, gh, ṅ) are savarṇa."""
        self.assertTrue(eval_string('(savarna? (pvc-from-sym "k") (pvc-from-sym "K"))', self.env))
        self.assertTrue(eval_string('(savarna? (pvc-from-sym "k") (pvc-from-sym "g"))', self.env))
        self.assertTrue(eval_string('(savarna? (pvc-from-sym "t") (pvc-from-sym "T"))', self.env))
        self.assertTrue(eval_string('(savarna? (pvc-from-sym "t") (pvc-from-sym "d"))', self.env))
        self.assertTrue(eval_string('(savarna? (pvc-from-sym "p") (pvc-from-sym "b"))', self.env))
        
        # Cross-varga are NOT savarṇa
        self.assertFalse(eval_string('(savarna? (pvc-from-sym "k") (pvc-from-sym "t"))', self.env))
        self.assertFalse(eval_string('(savarna? (pvc-from-sym "c") (pvc-from-sym "p"))', self.env))
        
        # Consonant vs Vowel in same place are NOT savarṇa (different primary prayatna)
        self.assertFalse(eval_string('(savarna? (pvc-from-sym "k") (pvc-from-sym "a"))', self.env))

    # ------------------------------------------------------------------------
    # 3. 64-Bit Pratyāhāra Membership Check
    # ------------------------------------------------------------------------
    def test_prat_member_primitive(self):
        """Test (prat-member? sound-code mask-64) in single clock cycle."""
        # Vowels (ac)
        self.assertTrue(eval_string('(prat-member? (quote a) (quote ac))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote i) (quote ac))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote u) (quote ac))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote e) (quote ac))', self.env))
        self.assertFalse(eval_string('(prat-member? (quote k) (quote ac))', self.env))

        # Consonants (hal)
        self.assertTrue(eval_string('(prat-member? (quote k) (quote hal))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote t) (quote hal))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote s) (quote hal))', self.env))
        self.assertFalse(eval_string('(prat-member? (quote a) (quote hal))', self.env))

        # Specialized pratyāhāras: ik (i, u, ṛ, ḷ), ec (e, o, ai, au), Sar (ś, ṣ, s)
        self.assertTrue(eval_string('(prat-member? (quote i) (quote ik))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote u) (quote ik))', self.env))
        self.assertFalse(eval_string('(prat-member? (quote a) (quote ik))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote e) (quote ec))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote O) (quote ec))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote S) (quote Sar))', self.env))
        self.assertTrue(eval_string('(prat-member? (quote s) (quote Sar))', self.env))
        self.assertFalse(eval_string('(prat-member? (quote k) (quote Sar))', self.env))

    # ------------------------------------------------------------------------
    # 4. Pratyāhāra Set Algebra
    # ------------------------------------------------------------------------
    def test_pratyahara_set_algebra(self):
        """Test bitwise set operations: intersect, union, diff, subset?"""
        # ik ⊆ ac
        self.assertTrue(eval_string('(prat-subset? (prat-mask (quote ik)) (prat-mask (quote ac)))', self.env))
        self.assertFalse(eval_string('(prat-subset? (prat-mask (quote ac)) (prat-mask (quote ik)))', self.env))

        # ac ∩ ik == ik
        self.assertEqual(
            eval_string('(prat-intersect (prat-mask (quote ac)) (prat-mask (quote ik)))', self.env),
            eval_string('(prat-mask (quote ik))', self.env),
        )

        # ac ∪ hal == al
        self.assertEqual(
            eval_string('(prat-union (prat-mask (quote ac)) (prat-mask (quote hal)))', self.env),
            eval_string('(prat-mask (quote al))', self.env),
        )

        # al \ ac == hal
        self.assertEqual(
            eval_string('(prat-diff (prat-mask (quote al)) (prat-mask (quote ac)))', self.env),
            eval_string('(prat-mask (quote hal))', self.env),
        )

    # ------------------------------------------------------------------------
    # 5. Instant Bitwise Transformations (Sandhi Voicing & Palatalization)
    # ------------------------------------------------------------------------
    def test_sandhi_voicing_and_palatalization(self):
        """Test (sandhi-voice sound) and (palatalize sound)."""
        # Voicing: k -> g
        k = eval_string('(pvc-from-sym "k")', self.env)
        voiced_k = eval_string('(sandhi-voice (pvc-from-sym "k"))', self.env)
        g = eval_string('(pvc-from-sym "g")', self.env)
        self.assertTrue(voiced_k.is_voiced)
        self.assertEqual(voiced_k.code, g.code)

        # Devoicing: g -> k
        devoiced_g = eval_string('(sandhi-devoice (pvc-from-sym "g"))', self.env)
        self.assertFalse(devoiced_g.is_voiced)
        self.assertEqual(devoiced_g.code, k.code)

        # Palatalization: t -> t'
        t = eval_string('(pvc-from-sym "t")', self.env)
        t_pal = eval_string('(palatalize (pvc-from-sym "t"))', self.env)
        self.assertTrue(t_pal.is_palatalized)
        self.assertEqual(t_pal.sthana, STHANA_DANTYA >> 1)
        self.assertTrue(t_pal.is_sprsta)

    # ------------------------------------------------------------------------
    # 6. S-expression Reader Macros: #pvc(...) and #prat(...)
    # ------------------------------------------------------------------------
    def test_reader_macro_pvc(self):
        """Test #pvc reader macro expansion."""
        res1 = eval_string('(savarna? #pvc("a") #pvc("A"))', self.env)
        self.assertTrue(res1)

        res2 = eval_string('(savarna? #pvc(k) #pvc(K))', self.env)
        self.assertTrue(res2)

        res3 = eval_string('(savarna? #pvc(k) #pvc(t))', self.env)
        self.assertFalse(res3)

        res4 = eval_string('(pvc-vowel? #pvc(:vowel t :sthana 1 :prayatna 256 :length 1 :modifier 0))', self.env)
        self.assertTrue(res4)

    def test_reader_macro_prat(self):
        """Test #prat reader macro expansion to 64-bit integer constant."""
        mask_ac = eval_string('#prat(ac)', self.env)
        self.assertEqual(mask_ac, 0x00000000000001FF)

        res1 = eval_string('(prat-member? (quote a) #prat(ac))', self.env)
        self.assertTrue(res1)

        res2 = eval_string('(prat-member? (quote k) #prat(ac))', self.env)
        self.assertFalse(res2)

        res3 = eval_string('(prat-member? (quote k) #prat(hal))', self.env)
        self.assertTrue(res3)


if __name__ == '__main__':
    unittest.main()
