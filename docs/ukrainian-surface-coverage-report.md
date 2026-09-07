# Ukrainian + Sanskrit Surface Coverage Report
# Звіт про покриття української та санскритської поверхні

**Date:** 2026-09-07
**Source:** `lib/surface/uk-sa-coverage.wsm` (schema uk-sa-coverage/1)
**Issue:** #1 — Ukrainian Surface Coverage

---

## USC (Ukrainian Surface Coverage) по шарах

USC = stable mappings / eligible public names

| Layer | Eligible | UK stable | UK candidate | UK missing | UK compat | UK USC | SA stable | SA candidate | SA missing | SA compat | SA USC |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Canon 0+7 | 7 | 7 | 0 | 0 | 0 | **100%** | 7 | 0 | 0 | 0 | **100%** |
| Necessary forms | 2 | 2 | 0 | 0 | 0 | **100%** | 0 | 0 | 2 | 0 | 0% |
| Language macros | 1 | 0 | 0 | 1 | 0 | 0% | 0 | 0 | 1 | 0 | 0% |
| Compatibility | 1 | 0 | 0 | 0 | 1 | — | 0 | 0 | 0 | 1 | — |
| Arithmetic | 13 | 0 | 13 | 0 | 0 | 0% | 0 | 11 | 2 | 0 | 0% |
| Comparisons | 5 | 0 | 5 | 0 | 0 | 0% | 0 | 5 | 0 | 0 | 0% |
| Predicates | 6 | 0 | 6 | 0 | 0 | 0% | 0 | 5 | 1 | 0 | 0% |
| Lists | 17 | 0 | 13 | 0 | 4 | 0% | 0 | 13 | 4 | 4 | 0% |
| Higher-order | 3 | 0 | 3 | 0 | 0 | 0% | 0 | 3 | 0 | 0 | 0% |
| Strings | 13 | 0 | 13 | 0 | 0 | 0% | 0 | 7 | 6 | 0 | 0% |
| I/O | 7 | 0 | 7 | 0 | 0 | 0% | 0 | 0 | 7 | 0 | 0% |
| Vectors | 5 | 0 | 5 | 0 | 0 | 0% | 0 | 0 | 5 | 0 | 0% |
| Buffers | 6 | 0 | 0 | 0 | 6 | — | 0 | 0 | 0 | 6 | — |
| Time | 4 | 0 | 2 | 0 | 2 | 0% | 0 | 0 | 4 | 0 | 0% |
| Other | 10 | 0 | 6 | 3 | 1 | 0% | 0 | 3 | 7 | 1 | 0% |
| **Overall** | **89** | **9** | **62** | **4** | **14** | **79.8%** | **7** | **38** | **30** | **14** | **50.6%** |

Примітка: USC% рахує (stable + candidate) / eligible, бо candidate — це вже вибране ім'я, що очікує ратифікації. Compatibility-only не збільшує словник користувача і виключається зі знаменника.

---

## Прогресія Batch 1

```
До Batch 1:   UK 9/75 = 12.0%    SA 7/75 = 9.3%
Після Batch 1: UK 71/75 = 94.7%   SA 45/75 = 60.0%
Загалом (з compat): UK 71/89 = 79.8%   SA 45/89 = 50.6%
```

Batch 1 додав: 62 UK candidate + 38 SA candidate для arithmetic, comparisons, predicates, lists, strings, higher-order.

---

## Ключові рішення

### Три шари рівності (критичне розділення)

| Предикат | UK | SA | Семантика |
|:---|:---|:---|:---|
| `eq` | `тотожне?` | `abheda` | ідентичність об'єктів (Canon) |
| `=` | `рівне?` | `sama?` | числова рівність |
| `equal?` | `однакові?` | `tulya?` | структурна рівність |

### Колізійна стратегія

| Конфлікт | Рішення |
|:---|:---|
| `додати` (+) vs `приєднати` (append) | Розділені за контекстом: числа vs списки |
| `остача` (mod) vs `решта` (cdr) | Різні слова; `залишок` відхилено через колізію з `решта` |
| `менше?` (<) vs `найменше` (min) | Предикат `?` vs суперлатив |
| `більше?` (>) vs `найбільше` (max) | Предикат `?` vs суперлатив |
| `avasiṣṭa` (mod) vs `śeṣa` (cdr) | Різні терміни; `śeṣa` зайнято Canon |
| `saṅkalana` (append) vs `saṃyuj` (cons) | Різні корені: *kal* vs *yuj* |

### Санскритські терміни — епістемічний статус

| Статус | Кількість | Опис |
|:---|:---:|:---|
| **stable** (Canon) | 7 | Зафіксовані в calibration doc, перевірені проти першоджерел |
| **candidate** (standard math) | 15 | Стандартні математичні терміни (yoga, viyoga, guṇana, haraṇa, bhāga, mūla, sama, hīna, adhika, viloma, śūnya, śreṇī, pramāṇa, dvandva, saṃbandha) |
| **candidate** (Pāṇinian) | 8 | Терміни з Pāṇinian граматики (nāman, śabda, dvitīya, tṛtīya, caturtha, pañcama, krama (Canon), svarūpa (Canon)) |
| **candidate** (hypothetical) | 15 | Наші гіпотези — не мають прямого традиційного аналога (āvartana, kalpana, saṅgraha, sambaddha, pūrva, cheda, rūpa, svabhāva, sakalamūla, alpatara, brhattara, avasiṣṭa, na-adhika, na-hīna, śabdasaṃyoga) |
| **missing** | 30 | Потребують дослідження (I/O, vectors, conversions, buffers, time) |
| **compatibility-only** | 14 | Не перекладаються (FFI, algorithm names, c*r compositions) |

---

## Наступні кроки

1. **Ратифікація Batch 1** — перевірити кандидати через тести еквівалентності (task 7-8 з issue)
2. **Batch 2: I/O** — print, princ, read, eval, write-to-string (потрібні санскритські відповідники)
3. **Batch 3: Vectors** — vector, make-vector, vector-ref, vector-set!
4. **Batch 4: Conversions** — number->string, symbol->string, codepoint->string
5. **Batch 5: Missing** — gensym, env, defmacro
6. **CI drift detection** — новий public name не може з'явитись без класифікації
7. **Ukrainian-only acceptance program** — програма що використовує тільки укр. словник

---

## Міжсубстратне порівняння (AGENTS.md вимога)

| Концепт | UK surface | SA surface | Rust analog | Метафора |
|:---|:---|:---|:---|:---|
| addition | додати | yoga | `+` / `add` | UK = "прибавити", SA = "з'єднання" |
| subtraction | відняти | viyoga | `-` / `sub` | UK = "відібрати", SA = "роз'єднання" |
| multiplication | помножити | guṇana | `*` / `mul` | UK = "збільшити кратність", SA = "якісне множення" |
| division | поділити | haraṇa | `/` / `div` | UK = "розділити", SA = "брати частинами" |
| map | відобразити | āvartana | `iter().map()` | UK = математичне map, SA = "повторне обертання" |
| filter | відсіяти | kalpana | `iter().filter()` | UK = метафора сита, SA = "відбір" |
| reduce | згорнути | saṅgraha | `fold` / `reduce` | UK = складання, SA = "охоплення" |

UK метафори тяжіють до **практичних дій** (сито, складання, відібрати), SA — до **онтологічних понять** (з'єднання, роз'єднання, охоплення). Це відображає різницю між мовами: українська — практично-орієнтована, санскрит — філософськи-орієнтований.

---

*Звіт згенеровано з `lib/surface/uk-sa-coverage.wsm`. Всі candidate імена потребують ратифікації через тести еквівалентності.*
