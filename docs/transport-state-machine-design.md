# Transport state machine — agent-send / bus / oracle

**Статус:** DRAFT PROPOSAL · 2026-08-25 · Vyasa (COMPILER STEWARD)
**Задача:** SWARM-TRANSPORT-STATE-MACHINE-DECISION
**Мотивація:** HTTP 204 від prompt_async ≠ delivered; AGENT-MSG конверти
ходять без формалізованого життєвого циклу; різні агенти по-різному
інтерпретують статус доставки

---

## Стани повідомлення

```
COMPOSED → QUEUED → DELIVERED → READ → ACKNOWLEDGED
                ↘ FAILED (terminal)
```

| Стан | Значення | Хто фіксує |
|---|---|---|
| COMPOSED | envelope створено, ще не надіслано | sender |
| QUEUED | транспорт прийняв (HTTP 204 / codex queue / TCP write) | transport |
| DELIVERED | цільова сесія отримала і розпарсила | target runtime |
| READ | цільовий агент обробив як input | target agent |
| ACKNOWLEDGED | цільовий агент надіслав явний ack | target agent |
| FAILED | будь-який етап провалився назавжди | transport |

## Правила чесності

1. **QUEUED ≠ DELIVERED**: opencode prompt_async повертає 202/204 навіть
   коли сесія мертва (opencode#33394, #26635). Тільки direct GET на
   /session/{id} підтверджує існування.
2. **DELIVERED ≠ READ**: codex queue покладає в чергу для наступного
   turn; агент може не обробити негайно.
3. **READ ≠ ACKNOWLEDGED**: агент бачить повідомлення але може не
   погоджуватись або не діяти.
4. **FAILED terminal**: після FAILED повторна доставка = нове повідомлення
   з новим id (не retry старого).

## Відповідність поточним інструментам

| Інструмент | Що гарантує | Чого НЕ гарантує |
|---|---|---|
| opencode prompt_async | QUEUED | DELIVERED, READ (сесія може бути мертва) |
| codex queue | QUEUED | DELIVERED (наступний turn, не негайно) |
| claude --resume --print | DELIVERED + READ (якщо exit 0) | ACKNOWLEDGED |
| comms-log запис | AUDIT TRAIL (не доставка) | — |

## Пропозиція для agent-send v2

Додати `--track` прапорець: після send() автоматично перевірити
target session existence (GET /session/{id}) і оновити comms-log з
DELIVERED vs QUEUED-UNCONFIRMED. Це вже реалізовано вручну в
сесії vyasa (перевірка через curl GET перед логуванням).

## Conformance fixtures

| Сценарій | Очікуваний стан |
|---|---|
| send → 204 → target alive | QUEUED |
| send → 404 → target dead | FAILED |
| send → 200 → codex queued | QUEUED |
| read-file на мертвому шляху | FAILED |

---
*Read-only design doc. Реалізація — окрема задача після ратифікації.*
