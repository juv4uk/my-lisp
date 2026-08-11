# Ecosystem roadmap · Roadmap екосистеми · Ökosystem-Roadmap

## English

The product is one vertical system with three independently versioned repositories: `my-lisp` owns source semantics and canonical results; `fpga-lisp` owns tagged words, ISA, calling convention, image format, and physical execution; `cml` owns the AOT mapping between those contracts. Compatibility is the pair `(language contract, ISA contract)` plus the exact SHAs tested—not matching repository release numbers.

Verified on 2026-08-11: FPGA already has the seven basic operations, environments, closures, `cond`, a complete `eval(expr, env)`, and a CML end-to-end testbench. CML compiles variables, conditionals, closures, calls, quoted lists, strings, dotted lists, and variadic arguments, and has a partial Tier-1 conformance runner. The remaining problem is a complete, reproducible boundary:

1. ✅ `fpga-lisp` ISA contract `0.2` records tags, opcodes, registers, calling convention, program image, and limits.
2. ✅ `cml` has `compatibility.my` with my-lisp contract `1.0`, target ISA, tested SHAs, features, and limitations.
3. ✅ One generic adapter runs `expr → compile → assemble → simulate → canonical result → expected`.
4. ✅ Atoms, fixnums, proper lists, and dotted pairs are decoded exactly from the FPGA heap—never as `(...)`.
5. Successive blind Tier-1 fixtures run without content-specific adapter changes.
6. CML gets pinned interface CI and a non-blocking latest-heads compatibility job.
7. Guix supplies Rust, Guile, Python, and Icarus Verilog; proprietary board synthesis stays a separate capability.
8. Compiled Lisp grows by value: a small `core.my` subset, then `unify.my`, then `reason.my`.

## Українська

Продукт — одна вертикальна система з трьох незалежно версіонованих репозиторіїв: `my-lisp` володіє semantics джерела й канонічними результатами; `fpga-lisp` — tagged words, ISA, calling convention, image format і фізичним виконанням; `cml` — AOT-відображенням між цими контрактами. Сумісність — це пара `(language contract, ISA contract)` плюс точні перевірені SHA, а не однакові версії релізів.

Перевірено 2026-08-11: FPGA вже має сім базових операцій, environments, closures, `cond`, повний `eval(expr, env)` і CML E2E testbench. CML компілює variables, conditionals, closures, calls, quoted lists, strings, dotted lists і variadic arguments та має частковий Tier-1 conformance runner. Залишилась повна відтворювана межа:

1. ✅ ISA contract `fpga-lisp` `0.2` фіксує tags, opcodes, registers, calling convention, program image і limits.
2. ✅ `cml` має `compatibility.my` із контрактом my-lisp `1.0`, цільовою ISA, перевіреними SHA, features і обмеженнями.
3. ✅ Один generic adapter виконує `expr → compile → assemble → simulate → canonical result → expected`.
4. ✅ Adapter точно декодує atoms, fixnums, proper lists і dotted pairs із FPGA heap — ніколи не згортає їх до `(...)`.
5. Послідовні blind Tier-1 fixtures проходять без content-specific змін adapter-а.
6. CML отримує pinned interface CI та неблокуючий latest-heads compatibility job.
7. Guix надає Rust, Guile, Python та Icarus Verilog; пропрієтарний синтез плати лишається окремою capability.
8. Compiled Lisp росте за цінністю: малий subset `core.my`, потім `unify.my`, потім `reason.my`.

## Deutsch

Das Produkt ist ein vertikales System aus drei unabhängig versionierten Repositories: `my-lisp` besitzt Quellsemantik und kanonische Ergebnisse; `fpga-lisp` besitzt Tagged Words, ISA, Aufrufkonvention, Image-Format und physische Ausführung; `cml` besitzt die AOT-Abbildung zwischen beiden Verträgen. Kompatibilität ist das Paar `(Sprachvertrag, ISA-Vertrag)` plus die tatsächlich geprüften SHAs—nicht gleiche Release-Versionen.

Am 2026-08-11 verifiziert: Das FPGA besitzt die sieben Basisoperationen, Environments, Closures, `cond`, ein vollständiges `eval(expr, env)` und eine CML-End-to-End-Testbench. CML kompiliert Variablen, Bedingungen, Closures, Aufrufe, zitierte Listen, Strings, Dotted Lists und variadische Argumente und besitzt einen partiellen Tier-1-Konformitätsrunner. Offen bleibt eine vollständige reproduzierbare Grenze:

1. ✅ Der `fpga-lisp`-ISA-Vertrag `0.2` hält Tags, Opcodes, Register, Aufrufkonvention, Program-Image und Grenzen fest.
2. ✅ `cml` besitzt `compatibility.my` mit my-lisp-Vertrag `1.0`, Ziel-ISA, geprüften SHAs, Features und Grenzen.
3. ✅ Ein generischer Adapter führt `expr → compile → assemble → simulate → kanonisches Ergebnis → expected` aus.
4. ✅ Atome, Fixnums, echte Listen und Dotted Pairs werden aus dem FPGA-Heap exakt dekodiert—nie als `(...)`.
5. Aufeinanderfolgende blinde Tier-1-Fixtures laufen ohne inhaltsspezifische Adapteränderungen.
6. CML erhält gepinnte Interface-CI und einen nicht blockierenden Latest-Heads-Job.
7. Guix liefert Rust, Guile, Python und Icarus Verilog; proprietäre Board-Synthese bleibt eine getrennte Capability.
8. Kompiliertes Lisp wächst nach Nutzen: ein kleiner `core.my`-Teil, dann `unify.my`, dann `reason.my`.
