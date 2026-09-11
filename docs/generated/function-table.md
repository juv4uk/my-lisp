# Function table (generated projection)

**Authority:** `lib/surface/semantic-registry.wsm` — projection only, not a second source of truth.

Regenerate: `python3 scripts/generate-function-table.py`

| ID | Українська | Повна українська | full-uk status | English | Sanskrit | primary |
|----|------------|------------------|----------------|---------|----------|---------|
| `0001` | як-є | як-є | stable | quote | svarūpa | stable |
| `0002` | атом? | атом? | stable | atom | aṇu | stable |
| `0003` | тотожне? | тотожне? | stable | eq | abheda | stable |
| `0004` | сполучити | сполучити | stable | cons | saṃyuj | stable |
| `0005` | перше | перше | stable | car | ādi | stable |
| `0006` | решта | решта | stable | cdr | śeṣa | stable |
| `0007` | за-умовою | за-умовою | stable | cond | anukrama | stable |
| `0010` | функція | функція | stable | lambda | — | stable |
| `0011` | визначити | визначити | stable | define | — | stable |
| `0012` | визначити-макрос | визначити-макрос | stable | defmacro | — | stable |

… full 161 rows: run generator and open `docs/generated/function-table.md` after `python3 scripts/generate-function-table.py`.

Machine-readable: `lib/generated/function-table.wsm` (same generator).
