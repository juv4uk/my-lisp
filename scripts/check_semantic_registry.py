#!/usr/bin/env python3
"""Check the small language-neutral S-expression registry from ADR-007."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "lib" / "surface" / "semantic-registry.wsm"
STATUSES = {"stable", "candidate", "missing", "compatibility-only"}
ID = re.compile(r"^[0-9]{4,}$")


def tokens(source: str) -> list[str]:
    source = "\n".join(line.split(";", 1)[0] for line in source.splitlines())
    return re.findall(r"\(|\)|[^\s()]+", source)


def parse(items: list[str]):
    position = 0

    def one():
        nonlocal position
        if position >= len(items):
            raise ValueError("unexpected end of registry")
        token = items[position]
        position += 1
        if token != "(":
            if token == ")":
                raise ValueError("unexpected )")
            return token
        result = []
        while position < len(items) and items[position] != ")":
            result.append(one())
        if position >= len(items):
            raise ValueError("unclosed list")
        position += 1
        return result

    root = one()
    if position != len(items):
        raise ValueError("more than one top-level form")
    return root


def check(root) -> tuple[int, set[str]]:
    if not isinstance(root, list) or not root or root[0] != "sr/1":
        raise ValueError("registry must start with (sr/1 ...)")

    seen_ids: set[str] = set()
    languages: set[str] = set()

    for entry in root[1:]:
        if not isinstance(entry, list) or len(entry) < 2:
            raise ValueError(f"malformed semantic entry: {entry!r}")
        identity = entry[0]
        if not isinstance(identity, str) or not ID.fullmatch(identity):
            raise ValueError(
                f"semantic identity must contain digits only (minimum four): {identity!r}"
            )
        if identity in seen_ids:
            raise ValueError(f"duplicate semantic identity: {identity}")
        seen_ids.add(identity)

        entry_languages: set[str] = set()
        for surface in entry[1:]:
            if not isinstance(surface, list) or len(surface) != 3:
                raise ValueError(f"{identity}: surface must be (lang name status)")
            language, name, status = surface
            if language in entry_languages:
                raise ValueError(f"{identity}: duplicate surface {language}")
            entry_languages.add(language)
            languages.add(language)

            if status not in STATUSES:
                raise ValueError(f"{identity}/{language}: unknown status {status}")
            if status == "missing" and name != "—":
                raise ValueError(f"{identity}/{language}: missing surface must use —")
            if status != "missing" and name == "—":
                raise ValueError(f"{identity}/{language}: {status} surface needs a name")

    if not seen_ids:
        raise ValueError("registry has no semantic identities")
    return len(seen_ids), languages


def main() -> int:
    try:
        root = parse(tokens(REGISTRY.read_text(encoding="utf-8")))
        count, languages = check(root)
    except (OSError, ValueError) as error:
        print(f"semantic registry error: {error}")
        return 1

    print(f"semantic registry: {count} identities")
    print("surfaces: " + " ".join(sorted(languages)))
    print("meaning-first shape: CONFIRMED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
