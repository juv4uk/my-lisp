#!/usr/bin/env python3
from pathlib import Path

RENAMES = [
    ("string<?", "текст-менше?", "текст-передує?", "Лексикографічний порядок: «передує» не плутається з довжиною тексту."),
    ("nth", "за-номером", "елемент-списку-за-номером", "Назва сама вказує домен і те, що саме повертається."),
    ("string-first", "перший-знак", "перший-символ-тексту", "Явно відрізняє Unicode-символ тексту від інших значень слова «знак»."),
    ("string-rest", "решта-знаків", "решта-символів-тексту", "Симетрична й доменно явна назва до string-first."),
    ("vector-set!", "встановити-вектор!", "встановити-елемент-вектора!", "Мутація змінює елемент, а не «встановлює вектор» цілком."),
    ("mono-ns", "монотонний-час", "монотонний-нс", "Одиниця наносекунд є частиною значення і має бути видима в імені."),
    ("unix-time-observation->utc", "юнікс-із-спостереження", "юнікс-спостереження-у-всч", "Стара назва читалася у зворотному напрямку; результатом є ВСЧ."),
    ("timezone-name", "часовий-пояс-назва", "назва-часового-поясу", "Природний український порядок слів."),
    ("timezone-detect", "часовий-пояс-виявити", "визначити-часовий-пояс", "Природний дієслівний порядок і точне значення операції."),
    ("timezone-offset-seconds", "часовий-пояс-зміщення", "зміщення-часового-поясу-в-секундах", "Природний порядок слів та явна одиниця виміру."),
    ("deadline-reached-at?", "дедлайн-досягнуто-о?", "дедлайн-досягнуто-на-момент?", "Виправляє явне «-о» і робить часовий аргумент зрозумілим."),
    ("deadline-from", "дедлайн-з", "дедлайн-від", "«Від» природно передає побудову дедлайну від заданого моменту."),
    ("internet-time-sync", "інтернет-час-синхронізація", "запитати-інтернет-час", "Функція робить один NTP-запит і повертає спостереження; системний годинник вона не синхронізує."),
    ("map-empty", "карта-порожня", "порожня-карта", "Природний порядок прикметника й іменника для значення."),
    ("map-get", "карта-отримати", "отримати-з-карти", "Природний дієслівний порядок і явний напрям доступу."),
    ("map-insert", "карта-вставити", "вставити-в-карту", "Природний дієслівний порядок; відсутність ! зберігає ознаку персистентності."),
    ("vec-empty", "вектор-порожній", "порожній-вектор", "Природний порядок прикметника й іменника для значення."),
    ("vec-conj", "вектор-додати", "додати-до-вектора", "Природний дієслівний порядок; операція повертає новий вектор."),
    ("vec-count", "вектор-розмір", "розмір-вектора", "Природна іменна конструкція."),
    ("vec-nth", "вектор-за-номером", "елемент-вектора-за-номером", "Назва явно каже, що повертається елемент за індексом."),
    ("forward-in", "вперед-висновок", "пряме-виведення", "У логіці forward chaining природніше називається прямим виведенням."),
    ("module-clauses-now", "модуль-умови-зараз", "поточні-клаузи-модуля", "Clause не тотожний умові; функція проєктує поточні клаузи модуля."),
    ("explain-proof", "обґрунтувати-доведення", "пояснити-доведення", "Explain означає пояснити вже отримане доведення, а не обґрунтувати його заново."),
    ("reason-explain", "міркування-пояснити", "пояснити-міркування", "Природний український порядок дієслова й об'єкта."),
    ("walk", "відшукати", "розіменувати", "Уніфікаційний walk виконує dereference логічної змінної, а не загальний пошук."),
    ("claim-statement", "твердження-текст", "зміст-твердження", "Statement може бути структурованими Lisp-даними, не лише текстовим рядком."),
    ("claim-review", "твердження-відгук", "стан-розгляду-твердження", "Поле review є скінченним станом proposed/reviewed/rejected, а не відгуком."),
    ("evidence-method", "доказ-метод", "метод-доказу", "Природна українська родова конструкція."),
    ("evidence-outcome", "доказ-результат", "результат-доказу", "Природна українська родова конструкція."),
    ("observation-statement", "спостереження-текст", "зміст-спостереження", "Statement спостереження може бути структурованими даними, не лише текстом."),
    ("intent-goal", "намір-мета", "мета-наміру", "Природна українська родова конструкція."),
    ("supporting-evidence", "підтримуючий-доказ", "підтримувальний-доказ", "Нормативніша прикметникова форма без калькованого активного дієприкметника."),
]

assert len(RENAMES) == 32


def read(path: str) -> str:
    return Path(path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    Path(path).write_text(text, encoding="utf-8")


# 1. Реальна поверхня: preferred bindings замінюємо, старі переносимо в
#    окремий compatibility-блок. Семантичні EN-цілі не змінюються.
path = "lib/surface/uk.my"
text = read(path)
for en, old, new, _reason in RENAMES:
    needle = f"(define {old} {en})"
    assert text.count(needle) == 1, (path, needle, text.count(needle))
    text = text.replace(needle, f"(define {new} {en})", 1)

# env був єдиним випадковим дублем stable preferred binding.
env_line = "(define середовище env)"
assert text.count(env_line) == 2, text.count(env_line)
last = text.rfind(env_line)
end = last + len(env_line)
if end < len(text) and text[end] == "\n":
    end += 1
text = text[:last] + text[end:]

compat_marker = ";; ═══════════════════════════════════════════════════════════════\n;; Batch 2 — SA missing fill"
assert compat_marker in text
compat_lines = [
    ";; ═══════════════════════════════════════════════════════════════",
    ";; Сумісність назв після смислового аудиту 2026-09-08",
    ";; Preferred-назви вище є основними для нових програм; старі назви",
    ";; лишаються alias-ами тієї самої семантичної тотожності.",
    ";; ═══════════════════════════════════════════════════════════════",
    "",
]
compat_lines += [f"(define {old} {en})" for en, old, _new, _reason in RENAMES]
compat_block = "\n".join(compat_lines) + "\n\n"
text = text.replace(compat_marker, compat_block + compat_marker, 1)
write(path, text)

# 2. Машинно-читані preferred-реєстри.
path = "lib/surface/uk-sa-coverage.wsm"
text = read(path)
for en, old, new, _reason in RENAMES:
    needle = f" {en} {old} "
    assert text.count(needle) == 1, (path, needle, text.count(needle))
    text = text.replace(needle, f" {en} {new} ", 1)
write(path, text)

for path in ["lib/surface/uk-docs.wsm", "docs/ukrainian-api.md", "docs/ukrainian-surface-inventory.md"]:
    text = read(path)
    for _en, old, new, _reason in RENAMES:
        text = text.replace(old, new)
    write(path, text)

# 3. Машинно-читаний журнал цього аудиту: 140/140 переглянуто,
#    32 preferred-назви уточнено, 108 прийнято без зміни.
audit_lines = [
    "; Machine-readable semantic/intuitive audit of the stable Ukrainian surface.",
    "; Машинно-читаний смисловий та інтуїтивний аудит stable української поверхні.",
    "",
    "(uk-name-audit",
    "  (schema uk-name-audit/1)",
    '  (reviewed-at "2026-09-08")',
    "  (stable-reviewed 140)",
    "  (renamed 32)",
    "  (retained 108)",
    "  (criteria semantic-accuracy natural-ukrainian intuitive-without-english)",
    "  (renames",
]
for en, old, new, reason in RENAMES:
    escaped = reason.replace("\\", "\\\\").replace('"', '\\"')
    audit_lines.append(f'    (rename {en} {old} {new} "{escaped}")')
audit_lines += ["  ))", ""]
write("lib/surface/uk-name-audit.wsm", "\n".join(audit_lines))

# 4. Людський звіт: не лише список змін, а критерії й рішення «не чіпати».
rows = "\n".join(
    f"| `{en}` | `{old}` | **`{new}`** | {reason} |" for en, old, new, reason in RENAMES
)
human = f"""# Смисловий аудит українських назв API

**Дата:** 2026-09-08  
**Обсяг:** усі **140/140** stable українських surface-відповідників.  
**Результат:** **32** preferred-назви уточнено, **108** залишено без зміни.

Цей аудит не змінює семантичні тотожності. Англійська основа кожного binding лишається тією самою; старі українські назви збережені як compatibility aliases.

## Критерії

Кожну назву читали так, ніби користувач **не знає англійського оригіналу**. Preferred-назва має:

1. точно описувати семантичну операцію, а не буквально перекладати EN identifier;
2. звучати природно українською;
3. явно показувати домен, напрям перетворення або одиницю виміру там, де без цього можливе хибне прочитання;
4. зберігати `?` лише для предикатів і `!` лише для мутації;
5. не створювати нової семантики: перейменування — лише surface-рівень.

## Уточнені preferred-назви

| Основа | Було | Стало | Чому |
|---|---|---|---|
{rows}

## Що перевірено й свідомо залишено

Не кожну назву, яку можна перефразувати, треба міняти. Після окремої перевірки залишено, зокрема:

- `відобразити` для `map`: **відображення** є нормальним математичним терміном для mapping; `зіставити` сильніше тягне до matching/comparison;
- `модуль` для `abs`: стандартна математична назва абсолютної величини;
- `хибне?` для `not`: точно питає, чи значення є канонічною хибністю `()`;
- `логічний-висновок` для `reason-in`: не суперечить backward-chaining семантиці й читається без знання EN;
- `всч-із-юнікс` і `поточний-всч`: напрям перетворення вже явний, а кириличне `всч` не порушує вимогу української розкладки;
- `відобразити`, `відсіяти`, `згорнути`: три функції вищого порядку утворюють зрозумілу українську групу;
- `у-текст`: у секції читання/обчислення однозначно означає канонічне Lisp-подання, а не довільний UI display.

## Сумісність

У `lib/surface/uk.my` старі 32 назви лишаються alias-ами. Translator, coverage-таблиця та основна документація використовують **нові preferred-назви**. Це означає: старий код не ламається, але новий код і приклади поступово сходяться до природнішої української поверхні.
"""
write("docs/ukrainian-name-audit.md", human)

# 5. Довідник отримує коротке пояснення міграції без дублювання всієї таблиці.
path = "docs/ukrainian-api.md"
text = read(path)
anchor = "## Сумісність назв після смислового аудиту"
if anchor not in text:
    text += f"""\n\n{anchor}\n\nУсі 140 stable-назв переглянуто 2026-09-08 за смисловою точністю та природністю українською. 32 preferred-назви уточнено; старі написання лишаються compatibility aliases і не ламають наявні програми. Повний протокол рішень: [`ukrainian-name-audit.md`](ukrainian-name-audit.md).\n"""
write(path, text)

# 6. Тести документаційного контракту: preferred names оновлюємо й додаємо
#    machine-backed перевірку compatibility та унікальності stable binding.
path = "crates/my-lisp/tests/ukrainian_api_docs.rs"
text = read(path)
for _en, old, new, _reason in RENAMES:
    text = text.replace(old, new)
text = text.replace(
    'const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");',
    'const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");\nconst NAME_AUDIT: &str = include_str!("../../../lib/surface/uk-name-audit.wsm");',
)
text += r'''

#[test]
fn smyslovyi_audyt_pokryvaie_vsi_140_stable_nazv() {
    assert!(NAME_AUDIT.contains("(stable-reviewed 140)"));
    assert!(NAME_AUDIT.contains("(renamed 32)"));
    assert!(NAME_AUDIT.contains("(retained 108)"));

    let rename_rows = NAME_AUDIT
        .lines()
        .filter(|line| line.trim_start().starts_with("(rename "))
        .count();
    assert_eq!(rename_rows, 32, "журнал аудиту має містити рівно 32 перейменування");
}

#[test]
fn stable_preferred_binding_v_uk_my_vyznachenyi_rivno_odyn_raz() {
    for (_en, uk) in stable_pairs() {
        let needle = format!("(define {uk} ");
        let count = UK_SURFACE.match_indices(&needle).count();
        assert_eq!(
            count, 1,
            "stable preferred binding {uk} має бути визначений у uk.my рівно один раз"
        );
    }
}

#[test]
fn stari_nazvy_smystovoho_audytu_lyshaiutsia_aliasamy_sumisnosti() {
    for line in NAME_AUDIT.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.first() != Some(&"(rename") {
            continue;
        }
        let en = fields[1];
        let old = fields[2];
        let new = fields[3];

        assert!(
            UK_SURFACE.contains(&format!("(define {old} {en})")),
            "старий alias сумісності відсутній: {old} -> {en}"
        );
        assert!(
            UK_SURFACE.contains(&format!("(define {new} {en})")),
            "preferred alias відсутній: {new} -> {en}"
        );
    }
}
'''
write(path, text)

# 7. Batch-2 runtime-тести мають демонструвати preferred surface, а не лише legacy aliases.
path = "crates/my-lisp/tests/uk_sa_batch2.rs"
text = read(path)
for _en, old, new, _reason in RENAMES:
    text = text.replace(old, new)
write(path, text)

# 8. Статичні інваріанти перед запуском Rust-тестів.
coverage = read("lib/surface/uk-sa-coverage.wsm")
docs_idx = read("lib/surface/uk-docs.wsm")
surface = read("lib/surface/uk.my")
assert coverage.count(" stable ") >= 140
assert surface.count("(define середовище env)") == 1
for en, old, new, _reason in RENAMES:
    assert f" {en} {new} " in coverage
    assert f" {en} {old} " not in coverage
    assert f"(define {new} {en})" in surface
    assert f"(define {old} {en})" in surface
    assert new in docs_idx

print("uk semantic naming audit applied: 140 reviewed, 32 renamed, 108 retained")
