# Function table (generated projection)

**Authority:** `lib/surface/semantic-registry.lisp` — projection only, not a second source of truth.

**Machine projection:** `lib/machine/intel-core-i5-6400.lisp` — physical execution paths only; it does not create language meaning.

Regenerate: `cargo run -p my-lisp-cli --bin my-lisp -- scripts/generate-function-table.lisp`

| ID | uk | ukr | ukr status | English | Sanskrit | primary | Intel Core i5-6400 / Skylake |
|----|----|-----|------------|---------|----------|---------|------------------------------|
| `00000001` | як-є | як-є | stable | quote | svarūpa | stable | — |
| `00000010` | атом? | атом? | stable | atom | aṇu | stable | tag-test: TEST/AND/CMP |
| `00000011` | тотожне? | тотожне? | stable | eq | abheda | stable | CMP/SETE |
| `00000100` | сполучити | сполучити | stable | cons | saṃyuj | stable | allocate+STORE-pair |
| `00000101` | перше | перше | stable | car | ādi | stable | LOAD-pair-head |
| `00000110` | решта | решта | stable | cdr | śeṣa | stable | LOAD-pair-tail |
| `00000111` | за-умовою | за-умовою | stable | cond | anukrama | stable | TEST/CMP+Jcc |
| `00001000` | функція | функція | stable | lambda | — | stable | — |
| `00001001` | визначити | визначити | stable | define | — | stable | — |
| `00001010` | визначити-макрос | визначити-макрос | stable | defmacro | — | stable | — |
| `00001011` | — | визначити | candidate | def | — | candidate | — |
| `00001100` | додати | додати | stable | — | yoga | stable | ADD / ADDSD |
| `00001101` | відняти | відняти | stable | — | viyoga | stable | SUB/NEG / SUBSD |
| `00001110` | помножити | помножити | stable | — | guṇana | stable | IMUL / MULSD |
| `00001111` | поділити | поділити | stable | — | haraṇa | stable | DIVSD; exact-rational routine |
| `00010000` | модуль | модуль | stable | abs | rūpa | stable | TEST/NEG/CMOV |
| `00010001` | найменше | найменше | stable | min | alpatara | stable | CMP/CMOV-min |
| `00010010` | найбільше | найбільше | stable | max | brhattara | stable | CMP/CMOV-max |
| `00010011` | остача | остача | stable | mod | avasiṣṭa | stable | IDIV-remainder |
| `00010100` | частка | частка | stable | quotient | bhāga | stable | IDIV-quotient |
| `00010101` | корінь | корінь | stable | sqrt | mūla | stable | SQRTSD |
| `00010110` | цілий-корінь | цілий-корінь | stable | isqrt | sakalamūla | stable | integer-sqrt routine |
| `00010111` | найменше-у-списку | найменше-у-списку | stable | min-list | — | stable | loop+CMP/CMOV-min |
| `00011000` | найбільше-у-списку | найбільше-у-списку | stable | max-list | — | stable | loop+CMP/CMOV-max |
| `00011001` | — | найбільший-фрагмент | candidate | largest-chunk | — | candidate | — |
| `00011010` | менше? | менше? | stable | — | hīna? | stable | CMP/SETL |
| `00011011` | більше? | більше? | stable | — | adhika? | stable | CMP/SETG |
| `00011100` | рівне? | рівне? | stable | — | sama? | stable | CMP/SETE |
| `00011101` | не-більше? | не-більше? | stable | — | na-adhika? | stable | CMP/SETLE |
| `00011110` | не-менше? | не-менше? | stable | — | na-hīna? | stable | CMP/SETGE |
| `00011111` | — | неспадне-починаючи-з? | candidate | nondecreasing-from? | — | candidate | — |
| `00100000` | — | незростаюче-починаючи-з? | candidate | nonincreasing-from? | — | candidate | — |
| `00100001` | хибне? | хибне? | stable | not | na | stable | TEST/SETE |
| `00100010` | однакові? | однакові? | stable | equal? | tulya? | stable | — |
| `00100011` | символ? | символ? | stable | symbol? | nāman? | stable | tag-test: symbol |
| `00100100` | текст? | текст? | stable | string? | śabda? | stable | tag-test: string |
| `00100101` | текст-передує? | текст-передує? | stable | string<? | śabda-hīna? | stable | — |
| `00100110` | числовий-буфер? | числовий-буфер? | stable | numeric-buffer? | — | stable | tag-test: numeric-buffer |
| `00100111` | список | список | stable | list | śreṇī | stable | — |
| `00101000` | довжина | довжина | stable | length | pramāṇa | stable | list-walk+LOAD-pair-tail |
| `00101001` | приєднати | приєднати | stable | append | saṅkalana | stable | — |
| `00101010` | зворот | зворот | stable | reverse | viloma | stable | — |
| `00101011` | елемент-списку-за-індексом | елемент-списку-за-індексом | stable | nth | kramāṅka | stable | indexed-list-walk+LOAD |
| `00101100` | значення-у-списку? | значення-у-списку? | stable | member? | sambaddha? | stable | — |
| `00101101` | знайти-за-ключем | знайти-за-ключем | stable | assoc | saṃbandha | stable | — |
| `00101110` | пара | пара | stable | pair | dvandva | stable | — |
| `00101111` | друге | друге | stable | second | dvitīya | stable | LOAD-tail+LOAD-head |
| `00110000` | третє | третє | stable | third | tṛtīya | stable | 2xLOAD-tail+LOAD-head |
| `00110001` | четверте | четверте | stable | fourth | caturtha | stable | 3xLOAD-tail+LOAD-head |
| `00110010` | п'яте | п'яте | stable | fifth | pañcama | stable | 4xLOAD-tail+LOAD-head |
| `00110011` | — | перше-від-першого | candidate | caar | — | candidate | LOAD-head+LOAD-head |
| `00110100` | — | перше-від-решти | candidate | cadr | — | candidate | LOAD-tail+LOAD-head |
| `00110101` | — | решта-від-решти | candidate | cddr | — | candidate | LOAD-tail+LOAD-tail |
| `00110110` | — | перше-після-трьох-решт | candidate | cadddr | — | candidate | 3xLOAD-tail+LOAD-head |
| `00110111` | відобразити | відобразити | stable | map | āvartana | stable | — |
| `00111000` | відсіяти | відсіяти | stable | filter | kalpana | stable | — |
| `00111001` | згорнути | згорнути | stable | reduce | saṅgraha | stable | — |
| `00111010` | зчепити | зчепити | stable | string-append | śabdasaṃyoga | stable | — |
| `00111011` | довжина-тексту | довжина-тексту | stable | string-length | śabdapramāṇa | stable | — |
| `00111100` | текст-порожній? | порожній-текст? | stable | string-empty? | śūnya? | stable | — |
| `00111101` | префікс-тексту? | текст-починається-з? | candidate | string-prefix? | pūrva? | stable | — |
| `00111110` | фрагмент-у-тексті? | фрагмент-у-тексті? | stable | string-contains? | śabdasambaddha? | stable | — |
| `00111111` | перший-символ-тексту | перший-символ-тексту | stable | string-first | prathamavarṇa | stable | — |
| `01000000` | решта-символів-тексту | решта-символів-тексту | stable | string-rest | śeṣavarṇa | stable | — |
| `01000001` | відрізати | відрізати | stable | string-slice | cheda | stable | — |
| `01000010` | символ-у-текст | символ-у-текст | stable | symbol->string | nāman-śabda | stable | — |
| `01000011` | текст-у-символ | текст-у-символ | stable | string->symbol | śabda-nāman | stable | — |
| `01000100` | кодова-точка-у-текст | кодова-точка-у-текст | stable | codepoint->string | varṇa-śabda | stable | — |
| `01000101` | текст-у-кодову-точку | текст-у-кодову-точку | stable | string->codepoint | śabda-varṇa | stable | — |
| `01000110` | число-у-текст | число-у-текст | stable | number->string | saṅkhyā-śabda | stable | — |
| `01000111` | — | цифра-у-текст | candidate | digit->string | — | candidate | — |
| `01001000` | друкувати | друкувати | stable | print | mudraṇa | stable | — |
| `01001001` | показати | показати | stable | princ | darśana | stable | — |
| `01001010` | прочитати | прочитати | stable | read | pāṭhana | stable | — |
| `01001011` | прочитати-усе | прочитати-усе | stable | read-all | pāṭhana-sarva | stable | — |
| `01001100` | значення-у-текст | значення-у-текст | stable | write-to-string | likhana | stable | — |
| `01001101` | обчислити | обчислити | stable | eval | vicāraṇa | stable | — |
| `01001110` | середовище | середовище | stable | env | āśraya | stable | — |
| `01001111` | вектор | вектор | stable | vector | samūha | stable | allocate+STORE-vector |
| `01010000` | створити-вектор | створити-вектор | stable | make-vector | samūha-nirmāṇa | stable | allocate+fill-vector |
| `01010001` | довжина-вектора | довжина-вектора | stable | vector-length | samūha-pramāṇa | stable | LOAD-vector-length |
| `01010010` | елемент-вектора | елемент-вектора | stable | vector-ref | samūha-āvartana | stable | LOAD-vector-element |
| `01010011` | встановити-елемент-вектора! | встановити-елемент-вектора! | stable | vector-set! | — | stable | STORE-vector-element |
| `01010100` | — | буфер-32-бітних-цілих-зі-знаком | candidate | i32-buffer | — | candidate | allocate+STORE-i32-buffer |
| `01010101` | — | буфер-32-бітних-чисел-з-плавною-комою | candidate | f32-buffer | — | candidate | allocate+STORE-f32-buffer |
| `01010110` | — | тип-числового-буфера | candidate | numeric-buffer-type | — | candidate | LOAD-buffer-tag |
| `01010111` | — | довжина-числового-буфера | candidate | numeric-buffer-length | — | candidate | LOAD-buffer-length |
| `01011000` | — | елемент-числового-буфера | candidate | numeric-buffer-ref | — | candidate | LOAD-buffer-element |
| `01011001` | — | відобразити-числовий-буфер | candidate | numeric-buffer-map | — | candidate | loop; AVX2 specialization possible |
| `01011010` | монотонний-нс | монотонний-час-у-наносекундах | candidate | mono-ns | kāla-mono | stable | — |
| `01011011` | поточний-юнікс-час | поточний-юнікс-час | stable | unix-time-now | kāla-unix | stable | — |
| `01011100` | — | сирий-запит-мережевого-часу | candidate | ntp-query-raw | — | candidate | — |
| `01011101` | — | сирі-декларації-часових-поясів | candidate | timezone-declarations-raw | — | candidate | — |
| `01011110` | поточний-всч | поточний-всесвітній-координований-час | candidate | utc-now | kāla-adya | stable | — |
| `01011111` | всч-із-юнікс | всесвітній-координований-час-із-часу-юнікс | candidate | utc-from-unix | — | stable | — |
| `01100000` | юнікс-спостереження-у-всч | спостереження-часу-юнікс-у-всесвітній-координований-час | candidate | unix-time-observation->utc | — | stable | — |
| `01100001` | мілісекунди-із-наносекунд | мілісекунди-із-наносекунд | stable | milliseconds-from-nanoseconds | kāla-millisecondāni | stable | IDIV/IMUL-reciprocal |
| `01100010` | монотонний-мс | монотонний-час-у-мілісекундах | candidate | mono-ms | kāla-mono-ms | stable | — |
| `01100011` | назва-часового-поясу | назва-часового-поясу | stable | timezone-name | deśa-kāla-nāma | stable | — |
| `01100100` | визначити-часовий-пояс | визначити-часовий-пояс | stable | timezone-detect | deśa-kāla-jñāna | stable | — |
| `01100101` | зміщення-часового-поясу-в-секундах | зміщення-часового-поясу-в-секундах | stable | timezone-offset-seconds | deśa-kāla-śeṣa | stable | — |
| `01100110` | дедлайн-досягнуто? | граничний-час-досягнуто? | candidate | deadline-reached? | avadhi-gatā? | stable | CMP/SETGE |
| `01100111` | дедлайн-досягнуто-на-момент? | граничний-час-досягнуто-на-момент? | candidate | deadline-reached-at? | avadhi-gatā-kadā? | stable | CMP/SETGE |
| `01101000` | минуло-нс | минуло-наносекунд | candidate | elapsed-ns | atīta-ns | stable | SUB |
| `01101001` | дедлайн-від | граничний-час-від | candidate | deadline-from | avadhi-nirmāṇa | stable | ADD |
| `01101010` | дедлайн-через-нс | граничний-час-через-наносекунди | candidate | deadline-after-ns | avadhi-anantara-ns | stable | ADD |
| `01101011` | запитати-інтернет-час | синхронізувати-час-через-інтернет | candidate | internet-time-sync | — | stable | — |
| `01101100` | порожня-карта | порожня-карта | stable | map-empty | kośa-śūnya | stable | — |
| `01101101` | отримати-з-карти | отримати-з-карти | stable | map-get | kośa-grahaṇa | stable | — |
| `01101110` | вставити-в-карту | вставити-в-карту | stable | map-insert | kośa-niveśana | stable | — |
| `01101111` | ключ-у-карті? | ключ-у-карті? | stable | map-contains? | kośa-sambaddha? | stable | — |
| `01110000` | карта-у-список | карта-у-список | stable | map->list | kośa-śreṇī | stable | — |
| `01110001` | порожній-вектор | порожній-вектор | stable | vec-empty | samūha-śūnya | stable | empty-vector object |
| `01110010` | додати-до-вектора | додати-до-вектора | stable | vec-conj | samūha-yukti | stable | allocate+copy+STORE-vector |
| `01110011` | розмір-вектора | розмір-вектора | stable | vec-count | samūha-gaṇana | stable | LOAD-vector-length |
| `01110100` | елемент-вектора-за-індексом | елемент-вектора-за-індексом | stable | vec-nth | samūha-kramāṅka | stable | LOAD-vector-element |
| `01110101` | вектор-у-список | вектор-у-список | stable | vec->list | samūha-śreṇī | stable | — |
| `01110110` | вектор-із-списку | вектор-із-списку | stable | vec-from-list | śreṇī-samūha | stable | — |
| `01110111` | факт? | факт? | stable | is-fact? | jñāna-satya? | stable | — |
| `01111000` | описати | описати | stable | describe | varṇana | stable | — |
| `01111001` | зібрати-факти-про | зібрати-факти-про | stable | collect-facts-about | jñāna-saṅgraha | stable | — |
| `01111010` | атом-у-списку? | атом-у-списку? | stable | contains-atom? | aṇu-sambaddha? | stable | — |
| `01111011` | пряме-виведення | пряме-виведення | stable | forward-in | abhimukha-anumāna | stable | — |
| `01111100` | логічний-висновок | логічний-висновок | stable | reason-in | anumāna | stable | — |
| `01111101` | конфлікт? | є-конфлікт? | candidate | check-conflict | virodha-parīkṣā | stable | — |
| `01111110` | модуль-відомий? | модуль-відомий? | stable | module-known? | — | stable | — |
| `01111111` | поточні-клаузи-модуля | поточні-клаузи-модуля | stable | module-clauses-now | — | stable | — |
| `10000000` | довести-мету | довести-мету | stable | prove-goal | siddhi-sādhana | stable | — |
| `10000001` | довести-мети | довести-мети | stable | prove-goals | siddhi-sādhana-sarva | stable | — |
| `10000010` | пояснити-доведення | пояснити-доведення | stable | explain-proof | siddhi-vyākhyā | stable | — |
| `10000011` | джерело-доведення | джерело-доведення | stable | source-of | siddhi-mūla | stable | — |
| `10000100` | походження | походження | stable | provenance | utpatti | stable | — |
| `10000101` | міркування | міркування | stable | reason | tarka | stable | — |
| `10000110` | пояснити-міркування | пояснити-міркування | stable | reason-explain | tarka-vyākhyā | stable | — |
| `10000111` | уніфікувати | уніфікувати | stable | unify | ekīkaraṇa | stable | — |
| `10001000` | логічна-змінна | логічна-змінна | stable | logic-var | tarka-cihna | stable | — |
| `10001001` | змінна? | змінна? | stable | var? | cihna? | stable | — |
| `10001010` | підставити | підставити | stable | apply-subst | pratyāroha | stable | — |
| `10001011` | розіменувати | розіменувати | stable | walk | vicāraṇa-gamana | stable | — |
| `10001100` | змінна-зустрічається? | змінна-зустрічається? | stable | occurs-check | parivṛtti-parīkṣā | stable | — |
| `10001101` | твердження? | твердження? | stable | claim? | pratyaya? | stable | — |
| `10001110` | зміст-твердження | зміст-твердження | stable | claim-statement | pratyaya-vākya | stable | — |
| `10001111` | стан-розгляду-твердження | стан-розгляду-твердження | stable | claim-review | pratyaya-parīkṣā | stable | — |
| `10010000` | доказ? | доказ? | stable | evidence? | pramāṇa? | stable | — |
| `10010001` | метод-доказу | метод-доказу | stable | evidence-method | pramāṇa-vidhi | stable | — |
| `10010010` | результат-доказу | результат-доказу | stable | evidence-outcome | pramāṇa-phala | stable | — |
| `10010011` | спостереження? | спостереження? | stable | observation? | pratyakṣa? | stable | — |
| `10010100` | зміст-спостереження | зміст-спостереження | stable | observation-statement | pratyakṣa-vākya | stable | — |
| `10010101` | намір? | намір? | stable | intent? | saṅkalpa? | stable | — |
| `10010110` | мета-наміру | мета-наміру | stable | intent-goal | saṅkalpa-lakṣya | stable | — |
| `10010111` | підтримувальний-доказ | підтримувальний-доказ | stable | supporting-evidence | sahāya-pramāṇa | stable | — |
| `10011000` | без-змін | повернути-без-змін | candidate | identity | svabhāva | stable | MOV/pass-through |
| `10011001` | генерувати-символ | створити-унікальний-символ | candidate | gensym | nāman-nirmāṇa | stable | — |
| `10011010` | та | та | stable | and | — | stable | short-circuit TEST/Jcc |
| `10011011` | або | або | stable | or | — | stable | short-circuit TEST/Jcc |
| `10011100` | нехай | нехай | stable | let | — | stable | — |
| `10011101` | нехай* | нехай-послідовно | candidate | let* | — | stable | — |
| `10011110` | — | ланцюжок-першого-аргументу | candidate | — | — | candidate | — |
| `10011111` | — | ланцюжок-останнього-аргументу | candidate | — | — | candidate | — |
| `10100000` | — | розібрати-текст-формату-джейсон | candidate | json-parse | — | candidate | — |
| `10100001` | — | обчислити-хеш-ша-256-тексту-у-шістнадцятковому-записі | candidate | sha256-hex | — | candidate | — |
| `10100010` | запустити-процес | запустити-процес | candidate | process-run | — | stable | — |
| `10100011` | прочитати-текст-з-з'єднання-протоколу-керування-передаванням | прочитати-текст-з-з'єднання-протоколу-керування-передаванням | candidate | tcp-read | — | stable | — |
| `10100100` | записати-текст-у-з'єднання-протоколу-керування-передаванням | записати-текст-у-з'єднання-протоколу-керування-передаванням | candidate | tcp-write | — | stable | — |
| `10100101` | слухати-порт-протоколу-керування-передаванням | слухати-порт-протоколу-керування-передаванням | candidate | tcp-listen | — | stable | — |
| `10100110` | прочитати-файл | прочитати-файл | candidate | read-file | — | stable | — |
| `10100111` | записати-файл | записати-файл | candidate | write-file | — | stable | — |
