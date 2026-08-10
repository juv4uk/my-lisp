; A controlled-natural-language bridge from a fixed sentence shape to a
; knowledge clause `(head . body)`, in `lib/knowledge.my`'s / `lib/reason.my`'s
; format — the "text -> structure" half of the Advice Taker vision from
; private/lisp-to-knowledge.md §6, deliberately without a real NLP model.
;
; `understand` takes a *word list*, not a raw string — my-lisp has no string
; primitives (split/char-access), and adding them just to tokenize sentences
; would grow the Rust built-in surface for a feature this project's own
; CLAUDE.md says to avoid growing it for. So `(all planet have mass)` stands
; in for "All planets have mass": the caller has already done tokenization
; and (crucially) picked the exact singular class name the knowledge base
; uses — there is no morphology here (no plural stripping, no verb
; conjugation). This is "controlled" natural language in the strict sense:
; three fixed sentence shapes, matched structurally, not parsed statistically.
; A real free-text `understand` (LLM-backed) is a separate, later step — see
; PLAN.md Крок 9.6, deliberately removed once already for lacking this
; project's usual rigor (tests, trilingual docs, no hidden network calls).
;
; Shapes recognized:
;   (X is a Y)   / (X is an Y) / (X is Y)  -> fact:  ((Y X))
;   (X V Y)                                -> fact:  ((V X Y))
;   (all X have Y)                         -> rule:  ((has (var w) Y) (X (var w)))
;
; Місток контрольованої природної мови від фіксованої форми речення до
; знаннєвого clause `(head . body)`, у форматі `lib/knowledge.my`/`lib/reason.my`
; — половина "текст -> структура" бачення Advice Taker з
; private/lisp-to-knowledge.md §6, свідомо без справжньої NLP-моделі.
;
; `understand` приймає *список слів*, не сирий рядок — my-lisp не має
; рядкових примітивів (split/доступ до символів), а додавати їх лише заради
; токенізації речень означало б розширювати поверхню Rust built-in саме
; там, де власний `CLAUDE.md` проєкту просить цього не робити. Тож
; `(all planet have mass)` заміняє "All planets have mass": виклик уже
; токенізований, і (важливо) там уже точна однина назви класу, яку
; використовує база знань — тут немає морфології (без відкидання множини,
; без відмінювання дієслів). Це "контрольована" природна мова в строгому
; сенсі: три фіксовані форми речень, зіставлені структурно, не статистично.
; Справжній `understand` вільного тексту (на LLM) — окремий, пізніший крок —
; див. PLAN.md Крок 9.6, вже раз свідомо видалений за брак звичної для
; проєкту строгості (тестів, трилінгвальної документації, прихованих
; мережевих викликів).
;
; Розпізнавані форми:
;   (X is a Y)   / (X is an Y) / (X is Y)  -> факт:  ((Y X))
;   (X V Y)                                -> факт:  ((V X Y))
;   (all X have Y)                         -> правило: ((has (var w) Y) (X (var w)))
;
; Eine Brücke kontrollierter natürlicher Sprache von einer festen Satzform zu
; einem Wissens-Clause `(head . body)`, im Format von `lib/knowledge.my`/
; `lib/reason.my` — die Hälfte "Text -> Struktur" der Advice-Taker-Vision aus
; private/lisp-to-knowledge.md §6, bewusst ohne echtes NLP-Modell.
;
; `understand` nimmt eine *Wortliste*, keinen rohen String entgegen — my-lisp
; hat keine String-Primitive (split/Zeichenzugriff), und sie nur zur
; Tokenisierung von Sätzen hinzuzufügen würde die Rust-Built-in-Oberfläche
; genau dort vergrößern, wo das eigene CLAUDE.md des Projekts davon abrät.
; `(all planet have mass)` steht daher für "All planets have mass": der
; Aufrufer hat die Tokenisierung bereits erledigt und (entscheidend) den
; exakten Singular-Klassennamen gewählt, den die Wissensbasis verwendet —
; hier gibt es keine Morphologie (kein Pluralabbau, keine Verbkonjugation).
; Dies ist "kontrollierte" natürliche Sprache im strengen Sinn: drei feste
; Satzformen, strukturell abgeglichen, nicht statistisch geparst. Ein
; echtes `understand` für Freitext (LLM-gestützt) ist ein separater, späterer
; Schritt — siehe PLAN.md Schritt 9.6, bereits einmal bewusst entfernt wegen
; fehlender Sorgfalt (Tests, trilinguale Doku, versteckte Netzwerkaufrufe).
;
; Erkannte Formen:
;   (X is a Y)   / (X is an Y) / (X is Y)  -> Fakt:   ((Y X))
;   (X V Y)                                -> Fakt:   ((V X Y))
;   (all X have Y)                         -> Regel:  ((has (var w) Y) (X (var w)))

(def strip-article
  (lambda (words)
    (cond
      ((eq (car words) 'a) (cdr words))
      ((eq (car words) 'an) (cdr words))
      (t words))))

(def understand-is
  (lambda (words)
    (let ((subject (car words))
          (after-is (strip-article (cddr words))))
      (list (list (car after-is) subject)))))

(def understand-relation
  (lambda (words)
    (list (list (second words) (car words) (third words)))))

(def understand-universal
  (lambda (words)
    (let ((subject-class (second words))
          (property (cadddr words)))
      (list (list 'has (list 'var 'w) property)
            (list subject-class (list 'var 'w))))))

(def understand
  (lambda (words)
    (cond
      ((eq (car words) 'all) (understand-universal words))
      ((eq (second words) 'is) (understand-is words))
      (t (understand-relation words)))))
