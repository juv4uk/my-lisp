# Content identity · Ідентичність вмісту · Inhaltsidentität

## English

`my-lisp` defines identity before choosing a digest algorithm. `knowledge-content-address` returns the canonical, read-back-safe `write-to-string` representation of any knowledge value. Equal structures therefore have exactly the same address across conforming Rust, WASM, and future FPGA implementations; different structures have different canonical text.

`world-content-address` addresses `(world-history journal metadata)`. The journal already contains the complete ordered history, so recursively embedding every `parent` would duplicate the same history many times. Two worlds with equal current clauses but different tell/retract histories intentionally have different addresses. Independently reconstructed worlds with the same journal and metadata have the same address.

The address is an exact variable-length key, not a cryptographic hash. A future SHA-256 or hardware digest layer may compute `hash(canonical-address)` for compact transport and signatures, but it must not redefine semantic identity or become part of the evaluator kernel.

## Українська

`my-lisp` визначає ідентичність раніше, ніж обирає digest-алгоритм. `knowledge-content-address` повертає канонічне, придатне для зворотного `read` представлення `write-to-string`. Тому рівні структури мають точно однакову адресу в Rust, WASM і майбутній FPGA-реалізації; різні структури мають різний канонічний текст.

`world-content-address` адресує `(world-history journal metadata)`. Журнал уже містить повну впорядковану історію, тому рекурсивне вкладання кожного `parent` лише дублювало б її. Світи з однаковими поточними clauses, але різними tell/retract-історіями навмисно мають різні адреси. Незалежно реконструйовані світи з однаковими журналом і metadata мають одну адресу.

Адреса — точний ключ змінної довжини, не криптографічний hash. Майбутній SHA-256 або апаратний digest може обчислювати `hash(canonical-address)` для компактного транспорту й підписів, але не повинен перевизначати семантичну ідентичність чи входити до evaluator kernel.

## Deutsch

`my-lisp` definiert Identität, bevor ein Digest-Algorithmus gewählt wird. `knowledge-content-address` liefert die kanonische, wieder einlesbare `write-to-string`-Darstellung. Gleiche Strukturen haben daher in Rust, WASM und einer zukünftigen FPGA-Implementierung exakt dieselbe Adresse; verschiedene Strukturen verschiedenen kanonischen Text.

`world-content-address` adressiert `(world-history journal metadata)`. Das Journal enthält bereits die vollständige geordnete Geschichte; jeden `parent` rekursiv einzubetten würde sie nur vervielfachen. Welten mit gleichen aktuellen Clauses, aber unterschiedlichen Tell-/Retract-Geschichten haben bewusst verschiedene Adressen. Unabhängig rekonstruierte Welten mit gleichem Journal und gleichen Metadaten haben dieselbe Adresse.

Die Adresse ist ein exakter Schlüssel variabler Länge, kein kryptographischer Hash. Eine spätere SHA-256- oder Hardware-Digest-Schicht darf `hash(canonical-address)` für kompakten Transport und Signaturen berechnen, aber weder semantische Identität neu definieren noch Teil des Evaluator-Kerns werden.
