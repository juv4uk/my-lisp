# Docs Drift Sweep — 2026-08-25

**Автор:** Vyasa · **Метод:** grep всіх backtick-посилань на файли у docs/*.md → перевірка існування
**Результат:** 12 мертвих посилань знайдено, класифіковано на 3 категорії

---

## Категорія A: Історичні згадки (НЕ виправляти)

Ці посилання описують минулий стан і є точними для свого контексту:

| Файл | Посилання | Чому не чіпати |
|---|---|---|
| vision.md | crates/my-lisp-cli/src/llm.rs | Свідомо видалено; vision описує ЧОМУ |
| upc-unified-architecture.md | lib/upc.my | UPC заморожено за triage plan |
| upc8-for-my-lisp.md | lib/upc.my | Те саме |
| my-lisp-1-review-of-external-analyses.md | lib/upc.my | Історичний огляд |

## Категорія B: Cross-repo посилання (додати префікс)

Посилання на файли в ІНШИХ репозиторіях — не мертві, просто потребують уточнення:

| Файл | Посилання | Де насправді |
|---|---|---|
| VIVEKA-FINDINGS-2026-08-24.md | docs/AGENT-DIRECT-MESSAGING.md | ecosystem/docs/ |
| agent-doctrine.md | docs/how-to-work-with-sarvam.md | ecosystem/docs/ |
| VIVEKA-MY-LISP-CML-ANALYSIS | docs/heterogeneous-backends.md | cml/docs/ |
| audyt-ostannikh-komitiv | docs/manus-review-conclusions.md | shiva-sutras/docs/ |

## Категорія C: Справжній дрейф (виправити)

| Файл | Посилання | Проблема |
|---|---|---|
| lsp-m3-eval-diagnostics-design.md | crates/my-lisp-lsp/src/diagnostics_eval.rs | Ніколи не було створено; дизайн не реалізований |
| my-lisp-1-thorough-audit-2026-08-19.md | crates/my-lisp/src/semantic/transliteration.rs | Semantic crate реорганізовано |
| sanskrit-p6-alias-enumeration.md | crates/my-lisp/src/semantic/atoms.rs | Semantic crate реорганізовано |

---

## Рекомендація
Категорія C: додати примітку "(file since reorganized)" або оновити шлях.
Категорія B: додати префікс репозиторію до посилання.
Категорія A: лишити як є — історична точність важливіша за актуальність.
