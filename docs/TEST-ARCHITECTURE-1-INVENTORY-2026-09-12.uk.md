# TEST-ARCHITECTURE-1 — Інвентаризація, крок 1 (2026-09-12)

Це переклад англомовного оригіналу
[TEST-ARCHITECTURE-1-INVENTORY-2026-09-12.md](TEST-ARCHITECTURE-1-INVENTORY-2026-09-12.md).

Лише класифікація тестів за критичним оглядом власника. **На цьому кроці жоден тест
не об'єднано і не видалено.** Знахідки живлять кроки 2-5 (стиснення surface-тестів,
уніфікація meta-corpus, винесення policy-тестів, прохід видалення).

Головне питання до кожного тесту: *яку конкретну мутацію ловить цей тест, яку не
ловить жоден дешевший тест?*

## 1. Кластер surface/UK

- `cyrillic_extension_witness.rs` — підтверджено як застарілий historical-policy-witness
  (сам коментар файлу визнає, що це виконуваний доказ вже істинного факту, а не guard
  проти регресії). **Кандидат на видалення**, але лише після того, як міграція на
  `.lisp` (issue #81) просунеться настільки, що generic тест "той самий код, три
  розширення, той самий результат" зможе його замінити.
- **Знайдено живе протиріччя**: `rivnopravnist_mov.rs` та `runtime_peer_operators.rs`
  стверджують, що виконавчий код більше не читає `lib/surface/uk-sa-coverage.wsm`
  (стару EN-таблицю), але `uk_surface_equivalence.rs` — найсильніший canonical
  equivalence-тест і природна ціль для об'єднання ~45 дублюючих тестів з
  `uk_sa_surface.rs`/`uk_sa_batch2.rs` — досі парсить саме цей legacy-файл як джерело
  даних. Треба мігрувати `uk_surface_equivalence.rs` на `semantic-registry.wsm`
  безпосередньо перед об'єднанням інших тестів у нього.
- Число **140** (кількість stable UK surface) захардкожено незалежно щонайменше у
  4 місцях у 2 файлах (`uk_surface_equivalence.rs` ×2, `ukrainian_api_docs.rs` ×2),
  плюс пов'язані числа (`161`, `21`, `136`, `42`, `98`, `30`, `1`) — усі тихо
  застаріють з ростом реєстру. Варто виводити їх з реєстру один раз.
- `uk_sa_surface.rs` (~28 з ~30 тестів) і `uk_sa_batch2.rs` (~15 з ~17) майже повністю
  дублюють registry-driven перевірку `uk_surface_equivalence.rs` — найсильніші
  кандидати на об'єднання/видалення, але деякі назви можуть бути ще не в stable
  реєстрі — потрібна перевірка покриття, а не сліпе видалення.
- `peer_surface_identity.rs`, `runtime_peer_operators.rs` та `rivnopravnist_mov.rs`
  кожен окремо реалізують ту саму перевірку "затінення одного peer-імені не
  переналаштовує інші" з 3 різними захардкоженими списками імен — один
  параметризований тест замінив би всі три.
- Тести keyboard/no-Latin-characters у `uk_surface_equivalence.rs` та
  `ukrainska_programa_pryimannya.rs` — це text-lint, не семантичні тести; варто
  винести з `cargo test` в окремий lint-інструмент.
- `semantic_form_identity.rs`, `uk_surface_inventory.rs`, `translation_boundary.rs`,
  `time_host_surface.rs`, `crates/my-lisp-host/tests/process_surface.rs` — добре
  окреслені, без дублювання, поза межами цього стиснення (деякі — хибні
  спрацювання пошуку через збіг назв, наприклад "translation"/"surface" означають
  там щось інше, не людську мовну поверхню).

## 2. Кластер meta-eval (15 файлів, 883 рядки, ~67 тестів)

- **Щонайменше 9 з 15 файлів порівнюють вихід meta з native замість порівняння
  обох з незалежним `expected` з корпусу**: `meta_eval_advice_taker.rs`,
  `meta_eval_closure_parity.rs`, `meta_eval_error_provenance.rs`,
  `meta_eval_evidence.rs` (більшість з 12 тестів), `meta_eval_later_binding.rs`
  (усі 6), `meta_eval_mutual.rs`, `meta_eval_parity.rs` (сам називає native
  "(oracle)" у власному panic-повідомленні), `meta_eval_semantic_registry.rs`
  (2 з 3). `meta_eval_errors.rs` виглядає так само на перший погляд, але
  насправді коректний (його `via_native` — це заданий вручну літерал, а не
  живе читання native).
- `IN_SCOPE_EXPRS` у `meta_eval_parity.rs` — точний приклад бюрократії
  "скопіювати вручну, потім перевірити": 25 виразів набрано вручну, а другий тест
  повторно парсить `conformance.my` лише щоб підтвердити точність копії.
- **У `tests/fixtures/conformance.my` немає позитивного тегу meta-eval-охоплення
  сьогодні** — існує лише негативний тег `meta-eval-gap`. Потрібно ввести новий тег
  (напр. `meta-eval` зі статусом `required`/`supported`, за зразком вже наявного
  вкладеного alist `wsm-native`), перш ніж будь-який runner зможе фільтрувати
  корпус напряму, за прикладом наявного `scripts/fixtures-for-tier.my`.
- 9 файлів мають майже ідентичний допоміжний код
  `meta_session`/`meta_eval_program`/`native_value` — природні кандидати на
  об'єднання в один corpus-driven runner: `meta_eval_parity.rs`,
  `meta_eval_evidence.rs`, `meta_eval_environment_semantics.rs`,
  `meta_eval_later_binding.rs`, `meta_eval_mutual.rs`,
  `meta_eval_error_kind_parity.rs`, `meta_eval_error_provenance.rs`,
  `meta_eval_error_detail_boundary.rs`, `meta_eval_closure_parity.rs`.
- `meta_eval.rs`, `meta_eval_advice_taker.rs`, `meta_eval_empty_program.rs`
  перевіряють справді окремі сценарії і можуть лишитись окремими файлами
  (потрібне лише виправлення напряму oracle у перших двох).
- `meta_eval_evidence_matrix.rs` двічі шелить у `python3`
  (`check-meta-eval-evidence.py`, `generate-meta-eval-evidence.py --check`) — це не
  семантичний тест, треба винести з `cargo test` повністю.

## 3. Кластер policy/docs/CI

- Не існує ні `xtask`-крейту, ні підкоманди `my-lisp verify` — цільове місце
  доведеться створювати, ймовірно обгортаючи/замінюючи наявні
  `scripts/check-meta-eval-evidence.py`, `scripts/generate-meta-eval-evidence.py`,
  `scripts/semantic-ownership.py`.
- `documentation_contract.rs`'s `public_docs_share_current_project_identity_and_extension`
  — живий приклад саме тієї проблеми, яку побоюється власник: тест зараз проходить,
  хоча [README.md:287](../README.md) досі каже, що канонічне розширення — `.wsm`, а
  [docs/language-core.md:11](language-core.md) каже — `.lisp`. Тест перевіряє лише
  "чи згадані всі три токени", а не яке з них справді канонічне, тож не може
  зловити це протиріччя. README.md треба виправити незалежно від того, де
  опиниться цей тест.
- `meta_eval_evidence_matrix.rs`, `semantic_ownership.rs` — чисті python3-shell-out
  перевірки дрейфу документації, винести з `cargo test`.
- `typed_buffer_proposal.rs` та два тести в `swarm_deprecation.rs` — реальні,
  корисні інваріанти, але вони перевіряють governance/контрактні метадані, а не
  runtime-поведінку; перенести в майбутній canon/contract validator, а не видаляти.
- `ukrainian_api_docs.rs` (7 з 11), `swarm_deprecation.rs` (2 з 4),
  `meta_eval_error_detail_boundary.rs` (1 з 3) — кожен змішує кілька чистих
  markdown/doc-text перевірок серед інакше легітимних behavior-тестів; варто
  розділити файл, а не переносити/видаляти цілком.
- `program_surface_translation.rs` справді тестує поведінку (paritet трьох
  поверхонь у виводі перекладача), попри залежність від `python3` — залишити,
  але недокументована залежність від середовища — це предмет для зміцнення, не
  для перенесення.

## Запропонований порядок кроків 2-5

1. Виправити твердження README.md про канонічне розширення (`.wsm`→`.lisp`) —
   невелика зміна, розблоковує виправлення `documentation_contract.rs` і усуває
   реальне живе протиріччя незалежно від рефакторингу тестів.
2. Мігрувати `uk_surface_equivalence.rs` з `uk-sa-coverage.wsm` на
   `semantic-registry.wsm` напряму (виправляє живе протиріччя), потім об'єднати
   дублюючі тести з `uk_sa_surface.rs`/`uk_sa_batch2.rs`, перевіривши що жодна
   назва не втратить покриття. Видалити `cyrillic_extension_witness.rs` після
   появи generic multi-extension witness.
3. Ввести тег `meta-eval` у корпус `conformance.my`, побудувати один
   data-driven runner, що його читає, мігрувати 9 файлів-дублікатів на нього,
   виправивши напрям oracle (і native, і meta перевіряються проти `expected`,
   ніколи одне проти одного) як частину того самого переносу.
4. Створити `xtask`/`verify` дім для docs-drift/policy перевірок, перенести
   виявлені чисті doc/python3-shell-out тести туди, розділити змішані файли.
5. Прохід видалення лише після завершення кроків 2-4, з явною позначкою для
   кожного видаленого тесту, який тест, що залишився, ловить ту саму мутацію.
