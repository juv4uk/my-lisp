# Codebase stats — ecosystem overview (tokei)

**Дата:** 2026-08-25 · **Автор:** Vyasa · **Інструмент:** tokei 12.1.2
**Типи:** Rust + Lisp + Python + TypeScript (без docs/comments у підсумку)

| Репо | Мова | Files | Code | Comments | Blanks |
|---|---|---|---|---|---|
| **my-lisp** | Rust | 82 | 18,716 | 1,361 | 1,787 |
| | Python | 4 | 934 | 89 | 183 |
| **cml** | Rust | 40 | 6,681 | 253 | 623 |
| | Python | 6 | 828 | 84 | 157 |
| **fpga-lisp** | Python | 17 | 1,509 | 86 | 251 |
| **my-lisp-panini** | Lisp | 2 | 217 | 41 | 24 |
| | Markdown (data) | — | — | — | — |
| **shiva-sutras** | TS+Rust+MD | 3+1 | ~1,187 | ~29 | ~48 |
| **my-idea** | Rust | 66 | 14,445 | 1,026 | 1,361 |
| **tauricode** | TypeScript | 2,663 | 479,812 | 12,645 | 41,170 |
| | Rust | 51 | 6,778 | 235 | 704 |

**Разом code:** ~531k рядків (tauricode = 90% як Tauri scaffold)
**Без tauricode:** ~52k рядків власного коду

## Спостереження
1. my-lisp = найбільший власний Rust проєкт (18.7k) — відповідає ролі центральної мови
2. panini має 120 файлів але більшість — markdown дані (63k total, 49k code — це корпус)
3. fpga-lisp — чистий Python (RTL .sv не рахується tokei як код)
4. tauricode — величезний scaffold але це згенерований Tauri boilerplate, не власна логіка
