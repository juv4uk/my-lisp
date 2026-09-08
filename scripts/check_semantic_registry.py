#!/usr/bin/env python3
"""Перевіряє єдину numeric-only authority мовних поверхонь.

Semantic ID — тільки цифри. UK/EN/SA є рівноправними людськими поверхнями,
`sym` — окрема спільна немовна нотація. Майбутня людська мова додається ще
однією `(xx назва стан)` формою без зміни схеми.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "lib" / "surface" / "semantic-registry.wsm"
STATUSES = {"stable", "candidate", "missing", "compatibility-only"}
ID = re.compile(r"^[0-9]{4,}$")
FIRST_WAVE = {"uk", "en", "sa"}
NON_HUMAN = {"sym"}


def tokens(source: str) -> list[str]:
    source = "\n".join(line.split(";", 1)[0] for line in source.splitlines())
    return re.findall(r"\(|\)|[^\s()]+", source)


def parse(items: list[str]):
    position = 0

    def one():
        nonlocal position
        if position >= len(items):
            raise ValueError("реєстр обірвався посеред форми")
        token = items[position]
        position += 1
        if token != "(":
            if token == ")":
                raise ValueError("неочікувана закривальна дужка")
            return token
        result = []
        while position < len(items) and items[position] != ")":
            result.append(one())
        if position >= len(items):
            raise ValueError("незакритий список")
        position += 1
        return result

    root = one()
    if position != len(items):
        raise ValueError("реєстр повинен мати рівно одну верхньорівневу форму")
    return root


def check(root) -> tuple[int, set[str], int]:
    if not isinstance(root, list) or not root or root[0] != "sr/1":
        raise ValueError("реєстр повинен починатися з (sr/1 ...)")

    seen_ids: set[str] = set()
    all_surfaces: set[str] = set()
    symbolic_count = 0

    for entry in root[1:]:
        if not isinstance(entry, list) or len(entry) < 2:
            raise ValueError(f"некоректний semantic-запис: {entry!r}")
        identity = entry[0]
        if not isinstance(identity, str) or not ID.fullmatch(identity):
            raise ValueError(
                f"semantic identity повинна містити тільки цифри (мінімум чотири): {identity!r}"
            )
        if identity in seen_ids:
            raise ValueError(f"дубль semantic identity: {identity}")
        seen_ids.add(identity)

        entry_surfaces: set[str] = set()
        for surface in entry[1:]:
            if not isinstance(surface, list) or len(surface) != 3:
                raise ValueError(f"{identity}: surface має форму (мова назва стан)")
            language, name, status = surface
            if not all(isinstance(value, str) for value in (language, name, status)):
                raise ValueError(f"{identity}: surface-поля мають бути атомами")
            if language in entry_surfaces:
                raise ValueError(f"{identity}: дубль surface {language}")
            entry_surfaces.add(language)
            all_surfaces.add(language)

            if status not in STATUSES:
                raise ValueError(f"{identity}/{language}: невідомий стан {status}")
            if status == "missing" and name != "—":
                raise ValueError(f"{identity}/{language}: missing мусить використовувати —")
            if status not in {"missing", "compatibility-only"} and name == "—":
                raise ValueError(f"{identity}/{language}: {status} потребує назви")
            if name == identity:
                raise ValueError(f"{identity}/{language}: surface name не може підміняти numeric ID")

            # Пунктуаційна нотація не належить жодній людській мові.
            if (
                language not in NON_HUMAN
                and name != "—"
                and not any(character.isalpha() for character in name)
            ):
                raise ValueError(
                    f"{identity}/{language}: символічне написання {name!r} мусить жити під sym"
                )

        missing_first_wave = FIRST_WAVE - entry_surfaces
        if missing_first_wave:
            raise ValueError(
                f"{identity}: UK/EN/SA мають бути явними; відсутні: "
                + ", ".join(sorted(missing_first_wave))
            )
        symbolic_count += int("sym" in entry_surfaces)

    if not seen_ids:
        raise ValueError("реєстр не містить semantic identities")
    return len(seen_ids), all_surfaces, symbolic_count


def main() -> int:
    try:
        root = parse(tokens(REGISTRY.read_text(encoding="utf-8")))
        count, surfaces, symbolic_count = check(root)
    except (OSError, ValueError) as error:
        print(f"semantic registry error: {error}")
        return 1

    print(f"semantic registry: {count} identities")
    print("surfaces: " + " ".join(sorted(surfaces)))
    print(f"shared symbolic identities: {symbolic_count}")
    print("numeric-only authority: CONFIRMED")
    print("meaning-first shape: CONFIRMED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
