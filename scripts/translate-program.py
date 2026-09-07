#!/usr/bin/env python3
"""Translate my-lisp program surface names between English, Ukrainian, Sanskrit.

Перекладає програмні імена поверхні my-lisp між англійською, українською та
санскритом. Форматування, коментарі, рядки, числа й невідомі користувацькі
символи зберігаються дослівно. Джерелом словника є
lib/surface/uk-sa-coverage.wsm; окремого словника в цьому скрипті немає.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
COVERAGE = REPO_ROOT / "lib" / "surface" / "uk-sa-coverage.wsm"
LANGUAGE_COLUMN = {"en": 2, "uk": 3, "sa": 4}


def surface_rows(source: str) -> list[tuple[str, str, str]]:
    rows = []
    for line in source.splitlines():
        fields = line.split()
        if fields[:1] != ["(entry"] or len(fields) < 7:
            continue
        rows.append((fields[2], fields[3], fields[4]))
    return rows


def translation_map(source_language: str, target_language: str) -> dict[str, str]:
    source_column = LANGUAGE_COLUMN[source_language] - 2
    target_column = LANGUAGE_COLUMN[target_language] - 2
    translations: dict[str, str] = {}
    for row in surface_rows(COVERAGE.read_text(encoding="utf-8")):
        source_name = row[source_column]
        target_name = row[target_column]
        if source_name == "—" or target_name == "—":
            continue
        previous = translations.get(source_name)
        if previous is not None and previous != target_name:
            raise ValueError(
                f"ambiguous {source_language} surface {source_name!r}: "
                f"{previous!r} or {target_name!r}"
            )
        translations[source_name] = target_name
    return translations


def translate_program(source: str, translations: dict[str, str]) -> str:
    """Rewrite symbols while preserving layout, comments, and string data."""
    output: list[str] = []
    index = 0
    length = len(source)

    while index < length:
        character = source[index]
        if character == ";":
            end = source.find("\n", index)
            if end == -1:
                output.append(source[index:])
                break
            output.append(source[index : end + 1])
            index = end + 1
        elif character == '"':
            start = index
            index += 1
            while index < length:
                if source[index] == "\\":
                    index += 2
                elif source[index] == '"':
                    index += 1
                    break
                else:
                    index += 1
            output.append(source[start:index])
        elif character.isspace() or character in "()":
            output.append(character)
            index += 1
        elif character == "'":
            # Initial apostrophe is reader syntax and not part of the symbol
            # following it. An apostrophe inside об'єкт remains in that token.
            output.append(character)
            index += 1
        else:
            start = index
            while index < length:
                current = source[index]
                if current.isspace() or current in "();\"":
                    break
                index += 1
            token = source[start:index]
            output.append(translations.get(token, token))

    return "".join(output)


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Translate a my-lisp program between en, uk, and sa surfaces."
    )
    parser.add_argument("input", type=Path, help="source program, or - for stdin")
    parser.add_argument("--from", dest="source_language", choices=LANGUAGE_COLUMN, required=True)
    parser.add_argument("--to", dest="target_language", choices=LANGUAGE_COLUMN, required=True)
    parser.add_argument("-o", "--output", type=Path, help="write here instead of stdout")
    return parser.parse_args()


def main() -> int:
    args = arguments()
    if args.source_language == args.target_language:
        print("source and target surfaces must differ", file=sys.stderr)
        return 2
    source = (
        sys.stdin.read()
        if str(args.input) == "-"
        else args.input.read_text(encoding="utf-8")
    )
    try:
        translations = translation_map(args.source_language, args.target_language)
    except (OSError, ValueError) as error:
        print(f"translation table error: {error}", file=sys.stderr)
        return 2
    translated = translate_program(source, translations)
    if args.output:
        args.output.write_text(translated, encoding="utf-8")
    else:
        sys.stdout.write(translated)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
