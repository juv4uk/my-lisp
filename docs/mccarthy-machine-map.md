# Машина, яку шукав Джон Маккарті

**Статус:** дослідницький огляд за першоджерелами
**Джерело:** Manus AI, через власника · **Дата:** 2026-08-23
**Оформлення:** Сакші (ox-alpha)
**Звʼязок:** lib/epistemic.my, docs/language-core-axioms.md, Worlds/journal

---

## Важлива межа

Маккарті не залишив одну завершену специфікацію «нервової системи ШІ».
Він протягом десятиліть формував **дослідницьку програму**: якою має бути
мова знань, як машина представляє дію й ситуацію, як reason-ити за неповного
знання, як подати контекст і як агент може знати межі власного знання.

## 1. Головна відповідь

Машина Маккарті мала не просто видавати правильні відповіді. Вона мала:

1. зберігати знання про світ у зрозумілій формальній мові;
2. робити висновки, включно з висновками про дії;
3. описувати зміну світу через ситуації, дії та наслідки;
4. діяти за неповного знання через defeasible/default assumptions,
   не плутаючи їх із вічними фактами;
5. знати частину того, чого не може вирішити з наявної памʼяті,
   і шукати зовнішню інформацію;
6. залишатися зрозумілою своїм конструкторам, а не лише статистично успішною.

> «Logical AI is more ambitious. We propose to understand the facts of the
> common sense world well enough to make a system that can act intelligently
> and can learn explicitly from its experience.» — Джон Маккарті [7]

Це не product blueprint — це **анатомія вимог до мислячої машини**.

## 2. Карта читання: сім ключових текстів

| Рік | Праця | Питання |
| --- | --- | --- |
| 1958 | Programs with Common Sense | Зародок Advice Taker, logical AI. |
| 1969 | Some Philosophical Problems (з Хейзом) | Situation calculus: situations, actions, strategies, knowledge. |
| 1977 | Epistemological Problems of AI | Epistemological vs heuristic problem. |
| 1980 | Circumscription | Formal default reasoning, non-monotonic. |
| 1993/98 | Formalizing Context (з Бувацом) | Contexts as first-class objects; ist(c,p). |
| 1995 | Making Robots Conscious of Their Mental States | Reasoning про own state і nonknowledge. |
| пізній draft | Human-Level AI: The Logical Road | Незавершений синтез, чесні gaps. |

Порядок для розуміння ідеї — не implementation manuals.

## 3. 1958: Advice Taker

Програма маніпулює sentences формальної мови: declarative або imperative
conclusions; imperative запускає action [1]. Logic = метод представляти
інформацію в памʼяті компʼютера [1].

Радикальне: knowledge-level reprogrammability — нове правило/факт у тому
самому medium змінює висновки без переписування програми. Не «prompt».

Чого не обіцяє: NLU, perception, масштаб KB, fast search. Formalization +
inference — необхідні, недостатні.

## 4. 1969: ситуація, дія, стратегія, знання

З Хейзом — situation calculus [2]: S0 + action → Result(A,S0)=S1; fluents;
reasoning чи strategy досягає goal. Abstract: situation, fluent, action,
strategy, result, knowledge; loops; acquisition of knowledge [3].

Дві adequate representation:
- Epistemological: чи formalism виражає knowledge/actions/causality/goals?
- Heuristic: чи system знаходить conclusions достатньо швидко?

«База фактів» ≠ розумна дія. «Сильний search» ≠ подання потрібного знання.

## 5. 1977: epistemological vs heuristic problem

| Частина | Питання |
| --- | --- |
| Epistemological | Які facts/categories/causal relations/contexts/goals/defaults представляти? |
| Heuristic | Як вибирати relevant, контролювати search, рахувати відповідь у ресурсах? |

Найкорисніша рамка для екосистеми: semantic contract ≠ inference strategy;
knowledge model ≠ current suggestion; proof of possibility ≠ efficient
implementation.

## 6. 1980: circumscription

Monotonic logic не скасує conclusion від нових фактів; common sense інакше:
Bird(Tweety) → usually Flies; Penguin(Tweety) → retract. Circumscription [5]
— logical operation, що мінімізує abnormal cases under chosen vocabulary.
Agent живе з відкличними припущеннями.

Урок для статусної лексики екосистеми:

| Вид твердження | Дисципліна |
| --- | --- |
| Direct observation | Provenance + перевірка |
| Derived theorem | Proof/derivation |
| Default inference | Явні assumptions + умови відкликання |
| Hypothesis | Proposal status |

Але JTMS/status vocabulary ≠ circumscription: різні formalisms, зіставляння
тільки через explicit contract.

## 7. Context як обʼєкт

Formalizing Context (Бувац): contexts as first-class objects; ist(c,p).
Context incomplete, власні assumptions/vocabulary/lifting rules.

Не плутати з my-lisp World:

| World | Formal context |
| --- | --- |
| Immutable knowledge/history state, provenance, transitions. | Logic object, relative to which p asserted. |
| Reproducibility/audit. | Context-dependent inference. |
| Implemented boundary. | Separate formalism; не імітувати передчасно. |

Worlds можуть колись host context-aware semantics — але не називати їх `ist`
без реалізації контрактів.

## 8. 1995: self-knowledge як operational necessity

Program має facts про own mental state [7]. Найпрактичніше: robot може
висновити, що НЕ МОЖЕ вирішити питання з memory → seek externally.

Міст до epistemic.my: observation/claim/evidence/intent + structured blocked
result = система не вдає всезнання. Не claim свідомості.

## 9. Пізній синтез: Human-Level AI: The Logical Road

Incomplete draft [8]: logical AI + commonsense + nonmonotonic + situation
calculus + elaboration tolerance + approximate objects + consciousness.
Потрібні conceptual advances, не merely scaling. Elaboration tolerance:
formalism витримує новий detail без total rewrite — library-first style
лишає exact evidence, де representation перестає бути expressive.

## 10. Синтез: нервова система в його сенсі

(Синтезація автора огляду, не canonical diagram Маккарті.)

```text
formal language (facts/rules/actions/goals)
        ↓
situation/context/state model
        ↓
reasoning: deduction + defaults + search
        ↙                    ↘
strategy/action         self-knowledge
effect in world         known/unknown/need-info
        ↘                    /
      changed situation → observation/advice
                 ↓
         revised knowledge
```

Три moral constraints:
1. Representation explicit enough to inspect.
2. Reasoning distinguishes knowledge from default/absence/unknown.
3. Action and information seeking justified within representation.

## 11. Де тут my-lisp, а де — не він

| Ідея Маккарті | Резонує | Чого не приписувати |
| --- | --- | --- |
| Symbolic form спільний для code/knowledge | Lisp data/code, metacircular. | Само по собі не дає common sense. |
| Explicit knowledge history | Worlds, journal, provenance. | World ≠ formal context. |
| Epistemology ≠ heuristic | Contracts separate from strategy. | Heuristics ≠ human-level planner. |
| Incompleteness | Statuses/partial/blocked/JTMS-like. | Не circumscription без minimization. |
| Self-knowledge of limits | epistemic.my blocked outcomes. | Не claim свідомості. |
| Explicit learning | Reviewable imports, provenance-bearing candidates. | LLM output не auto-admit як fact. |

Neural/LLM/multimodal tools — сучасні organs around his symbolic core,
contract-bound: perception/model proposal → evidence → review/validation →
symbolic admission. Neural — observer/candidate generator; symbolic tree
decides formal meaning, proof status, provenance.

## 12. Практичний порядок

| Зараз | Потім за потреби | Значно пізніше |
| --- | --- | --- |
| Small core; Worlds; provenance; explicit results; epistemic.my v0; tests. | Observation→review→admission path; narrow action/intent boundary; context experiments. | Default logic/circumscription; planning; context lifting; bounded info seeking; neural-worker ecology. |

## 13. Що читати першим

1. Programs with Common Sense (1958)
2. Some Philosophical Problems (1969)
3. Epistemological Problems (1977)
4. Circumscription (1980)
5. Making Robots Conscious (1995)
6. Human-Level AI: The Logical Road (draft)

## References

[1] http://jmc.stanford.edu/articles/mcc59.html
[2] http://jmc.stanford.edu/articles/mcchay69.html
[3] https://www.sciencedirect.com/science/chapter/edited-volume/pii/B9780934613033500337
[4] https://www.ijcai.org/Proceedings/77-2/Papers/094.pdf
[5] http://jmc.stanford.edu/articles/circumscription.html
[6] https://www-formal.stanford.edu/jmc/mccarthy-buvac-98/context.pdf
[7] https://aaai.org/papers/0013-ss95-05-013-making-robots-conscious-of-their-mental-states/
[8] http://jmc.stanford.edu/articles/logicalai.html
