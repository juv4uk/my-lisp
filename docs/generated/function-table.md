# Function table (generated projection)

**Authority:** `lib/surface/semantic-registry.lisp` — projection only, not a second source of truth.

**Machine projection:** `lib/machine/intel-core-i5-6400.lisp` — physical execution paths only; it does not create language meaning.

Regenerate: `cargo run -p my-lisp-cli --bin my-lisp -- scripts/generate-function-table.lisp`

| ID | uk | ukr | ukr status | English | Sanskrit | primary | Intel Core i5-6400 / Skylake |
|----|----|-----|------------|---------|----------|---------|------------------------------|
| `0001` | як-є | як-є | stable | quote | svarūpa | stable | — |
| `0002` | атом? | атом? | stable | atom | aṇu | stable | tag-test: TEST/AND/CMP |
| `0003` | тотожне? | тотожне? | stable | eq | abheda | stable | CMP/SETE |
| `0004` | сполучити | сполучити | stable | cons | saṃyuj | stable | allocate+STORE-pair |
| `0005` | перше | перше | stable | car | ādi | stable | LOAD-pair-head |
| `0006` | решта | решта | stable | cdr | śeṣa | stable | LOAD-pair-tail |
| `0007` | за-умовою | за-умовою | stable | cond | anukrama | stable | TEST/CMP+Jcc |
| `0010` | функція | функція | stable | lambda | — | stable | — |
| `0011` | визначити | визначити | stable | define | — | stable | — |
| `0012` | визначити-макрос | визначити-макрос | stable | defmacro | — | stable | — |
| `1000` | — | визначити | candidate | def | — | candidate | — |
| `0104` | додати | додати | stable | — | yoga | stable | ADD / ADDSD |
| `1001` | відняти | відняти | stable | — | viyoga | stable | SUB/NEG / SUBSD |
| `1002` | помножити | помножити | stable | — | guṇana | stable | IMUL / MULSD |
| `1003` | поділити | поділити | stable | — | haraṇa | stable | DIVSD; exact-rational routine |
| `1004` | модуль | модуль | stable | abs | rūpa | stable | TEST/NEG/CMOV |
| `1005` | найменше | найменше | stable | min | alpatara | stable | CMP/CMOV-min |
| `1006` | найбільше | найбільше | stable | max | brhattara | stable | CMP/CMOV-max |
| `1007` | остача | остача | stable | mod | avasiṣṭa | stable | IDIV-remainder |
| `1008` | частка | частка | stable | quotient | bhāga | stable | IDIV-quotient |
| `1009` | корінь | корінь | stable | sqrt | mūla | stable | SQRTSD |
| `1010` | цілий-корінь | цілий-корінь | stable | isqrt | sakalamūla | stable | integer-sqrt routine |
| `1011` | найменше-у-списку | найменше-у-списку | stable | min-list | — | stable | loop+CMP/CMOV-min |
| `1012` | найбільше-у-списку | найбільше-у-списку | stable | max-list | — | stable | loop+CMP/CMOV-max |
| `1013` | — | найбільший-фрагмент | candidate | largest-chunk | — | candidate | — |
| `1014` | менше? | менше? | stable | — | hīna? | stable | CMP/SETL |
| `1015` | більше? | більше? | stable | — | adhika? | stable | CMP/SETG |
| `1016` | рівне? | рівне? | stable | — | sama? | stable | CMP/SETE |
| `1017` | не-більше? | не-більше? | stable | — | na-adhika? | stable | CMP/SETLE |
| `1018` | не-менше? | не-менше? | stable | — | na-hīna? | stable | CMP/SETGE |
| `1019` | — | неспадне-починаючи-з? | candidate | nondecreasing-from? | — | candidate | — |
| `1020` | — | незростаюче-починаючи-з? | candidate | nonincreasing-from? | — | candidate | — |
| `1021` | хибне? | хибне? | stable | not | na | stable | TEST/SETE |
| `1022` | однакові? | однакові? | stable | equal? | tulya? | stable | — |
| `1023` | символ? | символ? | stable | symbol? | nāman? | stable | tag-test: symbol |
| `1024` | текст? | текст? | stable | string? | śabda? | stable | tag-test: string |
| `1025` | текст-передує? | текст-передує? | stable | string<? | śabda-hīna? | stable | — |
| `1026` | числовий-буфер? | числовий-буфер? | stable | numeric-buffer? | — | stable | tag-test: numeric-buffer |
| `1027` | список | список | stable | list | śreṇī | stable | — |
| `1028` | довжина | довжина | stable | length | pramāṇa | stable | list-walk+LOAD-pair-tail |
| `1029` | приєднати | приєднати | stable | append | saṅkalana | stable | — |
| `1030` | зворот | зворот | stable | reverse | viloma | stable | — |
| `1031` | елемент-списку-за-індексом | елемент-списку-за-індексом | stable | nth | kramāṅka | stable | indexed-list-walk+LOAD |
| `1032` | значення-у-списку? | значення-у-списку? | stable | member? | sambaddha? | stable | — |
| `1033` | знайти-за-ключем | знайти-за-ключем | stable | assoc | saṃbandha | stable | — |
| `1034` | пара | пара | stable | pair | dvandva | stable | — |
| `1035` | друге | друге | stable | second | dvitīya | stable | LOAD-tail+LOAD-head |
| `1036` | третє | третє | stable | third | tṛtīya | stable | 2xLOAD-tail+LOAD-head |
| `1037` | четверте | четверте | stable | fourth | caturtha | stable | 3xLOAD-tail+LOAD-head |
| `1038` | п'яте | п'яте | stable | fifth | pañcama | stable | 4xLOAD-tail+LOAD-head |
| `1039` | — | перше-від-першого | candidate | caar | — | candidate | LOAD-head+LOAD-head |
| `1040` | — | перше-від-решти | candidate | cadr | — | candidate | LOAD-tail+LOAD-head |
| `1041` | — | решта-від-решти | candidate | cddr | — | candidate | LOAD-tail+LOAD-tail |
| `1042` | — | перше-після-трьох-решт | candidate | cadddr | — | candidate | 3xLOAD-tail+LOAD-head |
| `0101` | відобразити | відобразити | stable | map | āvartana | stable | — |
| `0102` | відсіяти | відсіяти | stable | filter | kalpana | stable | — |
| `0103` | згорнути | згорнути | stable | reduce | saṅgraha | stable | — |
| `1043` | зчепити | зчепити | stable | string-append | śabdasaṃyoga | stable | — |
| `1044` | довжина-тексту | довжина-тексту | stable | string-length | śabdapramāṇa | stable | — |
| `1045` | текст-порожній? | порожній-текст? | stable | string-empty? | śūnya? | stable | — |
| `1046` | префікс-тексту? | текст-починається-з? | candidate | string-prefix? | pūrva? | stable | — |
| `1047` | фрагмент-у-тексті? | фрагмент-у-тексті? | stable | string-contains? | śabdasambaddha? | stable | — |
| `1048` | перший-символ-тексту | перший-символ-тексту | stable | string-first | prathamavarṇa | stable | — |
| `1049` | решта-символів-тексту | решта-символів-тексту | stable | string-rest | śeṣavarṇa | stable | — |
| `1050` | відрізати | відрізати | stable | string-slice | cheda | stable | — |
| `1051` | символ-у-текст | символ-у-текст | stable | symbol->string | nāman-śabda | stable | — |
| `1052` | текст-у-символ | текст-у-символ | stable | string->symbol | śabda-nāman | stable | — |
| `1053` | кодова-точка-у-текст | кодова-точка-у-текст | stable | codepoint->string | varṇa-śabda | stable | — |
| `1054` | текст-у-кодову-точку | текст-у-кодову-точку | stable | string->codepoint | śabda-varṇa | stable | — |
| `1055` | число-у-текст | число-у-текст | stable | number->string | saṅkhyā-śabda | stable | — |
| `1056` | — | цифра-у-текст | candidate | digit->string | — | candidate | — |
| `1057` | друкувати | друкувати | stable | print | mudraṇa | stable | — |
| `1058` | показати | показати | stable | princ | darśana | stable | — |
| `1059` | прочитати | прочитати | stable | read | pāṭhana | stable | — |
| `1060` | прочитати-усе | прочитати-усе | stable | read-all | pāṭhana-sarva | stable | — |
| `1061` | значення-у-текст | значення-у-текст | stable | write-to-string | likhana | stable | — |
| `1062` | обчислити | обчислити | stable | eval | vicāraṇa | stable | — |
| `1063` | середовище | середовище | stable | env | āśraya | stable | — |
| `1064` | вектор | вектор | stable | vector | samūha | stable | allocate+STORE-vector |
| `1065` | створити-вектор | створити-вектор | stable | make-vector | samūha-nirmāṇa | stable | allocate+fill-vector |
| `1066` | довжина-вектора | довжина-вектора | stable | vector-length | samūha-pramāṇa | stable | LOAD-vector-length |
| `1067` | елемент-вектора | елемент-вектора | stable | vector-ref | samūha-āvartana | stable | LOAD-vector-element |
| `1068` | встановити-елемент-вектора! | встановити-елемент-вектора! | stable | vector-set! | — | stable | STORE-vector-element |
| `1069` | — | буфер-32-бітних-цілих-зі-знаком | candidate | i32-buffer | — | candidate | allocate+STORE-i32-buffer |
| `1070` | — | буфер-32-бітних-чисел-з-плавною-комою | candidate | f32-buffer | — | candidate | allocate+STORE-f32-buffer |
| `1071` | — | тип-числового-буфера | candidate | numeric-buffer-type | — | candidate | LOAD-buffer-tag |
| `1072` | — | довжина-числового-буфера | candidate | numeric-buffer-length | — | candidate | LOAD-buffer-length |
| `1073` | — | елемент-числового-буфера | candidate | numeric-buffer-ref | — | candidate | LOAD-buffer-element |
| `1074` | — | відобразити-числовий-буфер | candidate | numeric-buffer-map | — | candidate | loop; AVX2 specialization possible |
| `1075` | монотонний-нс | монотонний-час-у-наносекундах | candidate | mono-ns | kāla-mono | stable | — |
| `1076` | поточний-юнікс-час | поточний-юнікс-час | stable | unix-time-now | kāla-unix | stable | — |
| `1077` | — | сирий-запит-мережевого-часу | candidate | ntp-query-raw | — | candidate | — |
| `1078` | — | сирі-декларації-часових-поясів | candidate | timezone-declarations-raw | — | candidate | — |
| `1079` | поточний-всч | поточний-всесвітній-координований-час | candidate | utc-now | kāla-adya | stable | — |
| `1080` | всч-із-юнікс | всесвітній-координований-час-із-часу-юнікс | candidate | utc-from-unix | — | stable | — |
| `1081` | юнікс-спостереження-у-всч | спостереження-часу-юнікс-у-всесвітній-координований-час | candidate | unix-time-observation->utc | — | stable | — |
| `1082` | мілісекунди-із-наносекунд | мілісекунди-із-наносекунд | stable | milliseconds-from-nanoseconds | kāla-millisecondāni | stable | IDIV/IMUL-reciprocal |
| `1083` | монотонний-мс | монотонний-час-у-мілісекундах | candidate | mono-ms | kāla-mono-ms | stable | — |
| `1084` | назва-часового-поясу | назва-часового-поясу | stable | timezone-name | deśa-kāla-nāma | stable | — |
| `1085` | визначити-часовий-пояс | визначити-часовий-пояс | stable | timezone-detect | deśa-kāla-jñāna | stable | — |
| `1086` | зміщення-часового-поясу-в-секундах | зміщення-часового-поясу-в-секундах | stable | timezone-offset-seconds | deśa-kāla-śeṣa | stable | — |
| `1087` | дедлайн-досягнуто? | граничний-час-досягнуто? | candidate | deadline-reached? | avadhi-gatā? | stable | CMP/SETGE |
| `1088` | дедлайн-досягнуто-на-момент? | граничний-час-досягнуто-на-момент? | candidate | deadline-reached-at? | avadhi-gatā-kadā? | stable | CMP/SETGE |
| `1089` | минуло-нс | минуло-наносекунд | candidate | elapsed-ns | atīta-ns | stable | SUB |
| `1090` | дедлайн-від | граничний-час-від | candidate | deadline-from | avadhi-nirmāṇa | stable | ADD |
| `1091` | дедлайн-через-нс | граничний-час-через-наносекунди | candidate | deadline-after-ns | avadhi-anantara-ns | stable | ADD |
| `1092` | запитати-інтернет-час | синхронізувати-час-через-інтернет | candidate | internet-time-sync | — | stable | — |
| `1093` | порожня-карта | порожня-карта | stable | map-empty | kośa-śūnya | stable | — |
| `1094` | отримати-з-карти | отримати-з-карти | stable | map-get | kośa-grahaṇa | stable | — |
| `1095` | вставити-в-карту | вставити-в-карту | stable | map-insert | kośa-niveśana | stable | — |
| `1096` | ключ-у-карті? | ключ-у-карті? | stable | map-contains? | kośa-sambaddha? | stable | — |
| `1097` | карта-у-список | карта-у-список | stable | map->list | kośa-śreṇī | stable | — |
| `1098` | порожній-вектор | порожній-вектор | stable | vec-empty | samūha-śūnya | stable | empty-vector object |
| `1099` | додати-до-вектора | додати-до-вектора | stable | vec-conj | samūha-yukti | stable | allocate+copy+STORE-vector |
| `1100` | розмір-вектора | розмір-вектора | stable | vec-count | samūha-gaṇana | stable | LOAD-vector-length |
| `1101` | елемент-вектора-за-індексом | елемент-вектора-за-індексом | stable | vec-nth | samūha-kramāṅka | stable | LOAD-vector-element |
| `1102` | вектор-у-список | вектор-у-список | stable | vec->list | samūha-śreṇī | stable | — |
| `1103` | вектор-із-списку | вектор-із-списку | stable | vec-from-list | śreṇī-samūha | stable | — |
| `1104` | факт? | факт? | stable | is-fact? | jñāna-satya? | stable | — |
| `1105` | описати | описати | stable | describe | varṇana | stable | — |
| `1106` | зібрати-факти-про | зібрати-факти-про | stable | collect-facts-about | jñāna-saṅgraha | stable | — |
| `1107` | атом-у-списку? | атом-у-списку? | stable | contains-atom? | aṇu-sambaddha? | stable | — |
| `1108` | пряме-виведення | пряме-виведення | stable | forward-in | abhimukha-anumāna | stable | — |
| `1109` | логічний-висновок | логічний-висновок | stable | reason-in | anumāna | stable | — |
| `1110` | конфлікт? | є-конфлікт? | candidate | check-conflict | virodha-parīkṣā | stable | — |
| `1111` | модуль-відомий? | модуль-відомий? | stable | module-known? | — | stable | — |
| `1112` | поточні-клаузи-модуля | поточні-клаузи-модуля | stable | module-clauses-now | — | stable | — |
| `1113` | довести-мету | довести-мету | stable | prove-goal | siddhi-sādhana | stable | — |
| `1114` | довести-мети | довести-мети | stable | prove-goals | siddhi-sādhana-sarva | stable | — |
| `1115` | пояснити-доведення | пояснити-доведення | stable | explain-proof | siddhi-vyākhyā | stable | — |
| `1116` | джерело-доведення | джерело-доведення | stable | source-of | siddhi-mūla | stable | — |
| `1117` | походження | походження | stable | provenance | utpatti | stable | — |
| `1118` | міркування | міркування | stable | reason | tarka | stable | — |
| `1119` | пояснити-міркування | пояснити-міркування | stable | reason-explain | tarka-vyākhyā | stable | — |
| `1120` | уніфікувати | уніфікувати | stable | unify | ekīkaraṇa | stable | — |
| `1121` | логічна-змінна | логічна-змінна | stable | logic-var | tarka-cihna | stable | — |
| `1122` | змінна? | змінна? | stable | var? | cihna? | stable | — |
| `1123` | підставити | підставити | stable | apply-subst | pratyāroha | stable | — |
| `1124` | розіменувати | розіменувати | stable | walk | vicāraṇa-gamana | stable | — |
| `1125` | змінна-зустрічається? | змінна-зустрічається? | stable | occurs-check | parivṛtti-parīkṣā | stable | — |
| `1126` | твердження? | твердження? | stable | claim? | pratyaya? | stable | — |
| `1127` | зміст-твердження | зміст-твердження | stable | claim-statement | pratyaya-vākya | stable | — |
| `1128` | стан-розгляду-твердження | стан-розгляду-твердження | stable | claim-review | pratyaya-parīkṣā | stable | — |
| `1129` | доказ? | доказ? | stable | evidence? | pramāṇa? | stable | — |
| `1130` | метод-доказу | метод-доказу | stable | evidence-method | pramāṇa-vidhi | stable | — |
| `1131` | результат-доказу | результат-доказу | stable | evidence-outcome | pramāṇa-phala | stable | — |
| `1132` | спостереження? | спостереження? | stable | observation? | pratyakṣa? | stable | — |
| `1133` | зміст-спостереження | зміст-спостереження | stable | observation-statement | pratyakṣa-vākya | stable | — |
| `1134` | намір? | намір? | stable | intent? | saṅkalpa? | stable | — |
| `1135` | мета-наміру | мета-наміру | stable | intent-goal | saṅkalpa-lakṣya | stable | — |
| `1136` | підтримувальний-доказ | підтримувальний-доказ | stable | supporting-evidence | sahāya-pramāṇa | stable | — |
| `1137` | без-змін | повернути-без-змін | candidate | identity | svabhāva | stable | MOV/pass-through |
| `1138` | генерувати-символ | створити-унікальний-символ | candidate | gensym | nāman-nirmāṇa | stable | — |
| `1139` | та | та | stable | and | — | stable | short-circuit TEST/Jcc |
| `1140` | або | або | stable | or | — | stable | short-circuit TEST/Jcc |
| `1141` | нехай | нехай | stable | let | — | stable | — |
| `1142` | нехай* | нехай-послідовно | candidate | let* | — | stable | — |
| `1143` | — | ланцюжок-першого-аргументу | candidate | — | — | candidate | — |
| `1144` | — | ланцюжок-останнього-аргументу | candidate | — | — | candidate | — |
| `1145` | — | розібрати-текст-формату-джейсон | candidate | json-parse | — | candidate | — |
| `1146` | — | обчислити-хеш-ша-256-тексту-у-шістнадцятковому-записі | candidate | sha256-hex | — | candidate | — |
| `1147` | запустити-процес | запустити-процес | candidate | process-run | — | stable | — |
| `1148` | прочитати-текст-з-з'єднання-протоколу-керування-передаванням | прочитати-текст-з-з'єднання-протоколу-керування-передаванням | candidate | tcp-read | — | stable | — |
| `1149` | записати-текст-у-з'єднання-протоколу-керування-передаванням | записати-текст-у-з'єднання-протоколу-керування-передаванням | candidate | tcp-write | — | stable | — |
| `1150` | слухати-порт-протоколу-керування-передаванням | слухати-порт-протоколу-керування-передаванням | candidate | tcp-listen | — | stable | — |
| `1151` | прочитати-файл | прочитати-файл | candidate | read-file | — | stable | — |
| `1152` | записати-файл | записати-файл | candidate | write-file | — | stable | — |
