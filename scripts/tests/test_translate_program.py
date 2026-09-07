#!/usr/bin/env python3
import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "translate-program.py"
SPEC = importlib.util.spec_from_file_location("translate_program", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class TranslateProgramTests(unittest.TestCase):
    def test_all_six_directions_are_available(self):
        for source in MODULE.LANGUAGE_COLUMN:
            for target in MODULE.LANGUAGE_COLUMN:
                if source != target:
                    self.assertTrue(MODULE.translation_map(source, target))

    def test_program_round_trip_preserves_source(self):
        english = "(car (cons 'cat (quote ())))\n"
        ukrainian = MODULE.translate_program(
            english, MODULE.translation_map("en", "uk")
        )
        self.assertEqual(ukrainian, "(перше (сполучити 'cat (як-є ())))\n")
        restored = MODULE.translate_program(
            ukrainian, MODULE.translation_map("uk", "en")
        )
        self.assertEqual(restored, english)

    def test_comments_strings_and_unknown_names_are_preserved(self):
        source = '(print "car atom") ; car atom\n(my-function об\'єкт)\n'
        result = MODULE.translate_program(
            source, MODULE.translation_map("en", "uk")
        )
        self.assertEqual(
            result,
            '(друкувати "car atom") ; car atom\n(my-function об\'єкт)\n',
        )

    def test_ukrainian_to_sanskrit_uses_the_shared_table(self):
        source = "(перше (сполучити 'кіт 'пес))"
        result = MODULE.translate_program(
            source, MODULE.translation_map("uk", "sa")
        )
        self.assertEqual(result, "(ādi (saṃyuj 'кіт 'пес))")


if __name__ == "__main__":
    unittest.main()
