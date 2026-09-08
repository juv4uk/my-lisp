#!/usr/bin/env python3
"""Перекладає my-lisp між людськими поверхнями через numeric identities.

Джерело словника — `lib/surface/semantic-registry.wsm`. Жодна людська мова не
є мостом до іншої. `sym` — спільна немовна нотація: такі токени не
"перекладаються з англійської", а зберігаються дослівно.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REGISTRY = REPO_ROOT / "lib" / "surface" / "semantic-registry.wsm"
ENTRY = re.compile(r"^\s*\(([0-9]{4,})\s+(.*)\)\s*$")
SURFACE = re.compile(
    r"\(([A-Za-z][A-Za-z0-9-]*)\s+([^\s()]+)\s+"
    r"(stable|candidate|missing|compatibility-only)\)"
)
NON_HUMAN = {"sym"}


def registry_rows() -> list[dict[str, tuple[str, str]]]:
    rows = []
    for line in REGISTRY.read_text(encoding="utf-8").splitlines():
        match = ENTRY.match(line)
        if not match:
            continue
        _identity, body = match.groups()
        surfaces = {
            language: (name, status)
            for language, name, status in SURFACE.findall(body)
        }
        rows.append(surfaces)
    if not rows:
        raise ValueError("numeric semantic registry has no entries")
    return rows


def human_languages(rows: list[dict[str, tuple[str, str]]]) -> set[str]:
    return {
        language
        for row in rows
        for language in row
        if language not in NON_HUMAN
    }


def translation_map(source_language: str, target_language: str) -> dict[str, str]:
    translations: dict[str, str] = {}
    for row in registry_rows():
        source = row.get(source_language)
        target = row.get(target_language)
        if source is None or target is None:
            continue
        source_name, source_status = source
        target_name, target_status = target
        if (
            source_name == "—"
            or target_name == "—"
            or source_status in {"missing", "compatibility-only"}
            or target_status in {"missing", "compatibility-only"}
        ):
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
    """Rewrite symbols while preserving layout, comments, strings and shared symbols."""
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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="source program, or - for stdin")
    parser.add_argument("--from", dest="source_language", required=True)
    parser.add_argument("--to", dest="target_language", required=True)
    parser.add_argument("-o", "--output", type=Path, help="write here instead of stdout")
    return parser.parse_args()


def main() -> int:
    args = arguments()
    try:
        rows = registry_rows()
        languages = human_languages(rows)
        unknown = {args.source_language, args.target_language} - languages
        if unknown:
            raise ValueError("unknown human surface(s): " + ", ".join(sorted(unknown)))
        if args.source_language == args.target_language:
            raise ValueError("source and target surfaces must differ")
        translations = translation_map(args.source_language, args.target_language)
    except (OSError, ValueError) as error:
        print(f"translation registry error: {error}", file=sys.stderr)
        return 2

    source = (
        sys.stdin.read()
        if str(args.input) == "-"
        else args.input.read_text(encoding="utf-8")
    )
    translated = translate_program(source, translations)
    if args.output:
        args.output.write_text(translated, encoding="utf-8")
    else:
        sys.stdout.write(translated)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
