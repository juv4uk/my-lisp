; A minimal backward-chaining inference engine (micro-Prolog), fulfilling the
; "Advice Taker" vision: deriving new facts from known rules and facts.
; Built directly on top of the unification primitive (`lib/unify.my`).
;
; Rules are structured as `(head . body)`.
; A simple fact has no body: `((parent alice bob))`
; A rule with conditions: `((grandparent (var x) (var y)) (parent (var x) (var z)) (parent (var z) (var y)))`
; 
; `reason` takes a `goal` and a list of `rules`, and returns a list of valid 
; substitutions (bindings) that satisfy the goal, or an empty list if none.
;
; Variable renaming (standardizing apart) is handled by attaching a depth 
; counter to variable names during rule instantiation. For example, `(var x)`
; at depth 3 becomes `(var (x . 3))`. This isolates variables between rules
; and allows for recursive logic!
;
; Мінімальний рушій логічного висновку (micro-Prolog), що втілює бачення
; "Advice Taker": виведення нових фактів з відомих правил та фактів.
; Побудований безпосередньо поверх примітиву унікації (`lib/unify.my`).
;
; Правила структуровані як `(head . body)`.
; Простий факт не має тіла: `((parent alice bob))`
; Правило з умовами: `((grandparent (var x) (var y)) (parent (var x) (var z)) (parent (var z) (var y)))`
;
; `reason` приймає `goal` (запит) та список `rules` (правил), і повертає список 
; валідних підстановок, що задовольняють запит, або порожній список.
;
; Перейменування змінних (standardizing apart) виконується шляхом додавання 
; лічильника глибини до імен змінних під час інстанціювання правила. Наприклад, 
; `(var x)` на глибині 3 стає `(var (x . 3))`. Це ізолює змінні між правилами 
; і дозволяє рекурсивну логіку!
;
; Eine minimale Inferenz-Engine (Micro-Prolog), die die "Advice Taker"-Vision
; erfüllt: Ableitung neuer Fakten aus bekannten Regeln und Fakten.
; Direkt aufgebaut auf dem Unifikations-Primitiv (`lib/unify.my`).
;
; Regeln sind als `(head . body)` strukturiert.
; Ein einfacher Fakt hat keinen Körper: `((parent alice bob))`
; Eine Regel mit Bedingungen: `((grandparent (var x) (var y)) (parent (var x) (var z)) (parent (var z) (var y)))`
;
; `reason` nimmt ein `goal` (Ziel) und eine Liste von `rules` (Regeln) und gibt 
; eine Liste gültiger Substitutionen zurück, die das Ziel erfüllen, oder eine leere Liste.
;
; Variablenumbenennung (standardizing apart) wird behandelt, indem bei der
; Instanziierung der Regel ein Tiefenzähler an die Variablennamen angehängt wird.
; Beispielsweise wird `(var x)` in Tiefe 3 zu `(var (x . 3))`. Dies isoliert Variablen
; zwischen Regeln und ermöglicht rekursive Logik!

(def reason
  (lambda (goal rules)
    (prove-goal goal rules '() rules 0)))

; Proves a single goal against a list of rules, accumulating successful substitutions and proof trees.
; Returns a list of `(subst proof)` pairs.
(def prove-goal
  (lambda (goal rules subst all-rules depth)
    (cond
      ((atom rules) '())
      (t (append 
           (prove-rule goal (car rules) subst all-rules depth)
           (prove-goal goal (cdr rules) subst all-rules depth))))))

; Recursively rename variables in a term to include the depth counter
(def rename-vars
  (lambda (term depth)
    (cond
      ((atom term) term)
      ((var? term) (list 'var (cons (second term) depth)))
      (t (cons (rename-vars (car term) depth) (rename-vars (cdr term) depth))))))

(def map-proofs
  (lambda (f lst)
    (cond
      ((atom lst) '())
      (t (cons (f (car lst)) (map-proofs f (cdr lst)))))))

; Tries to unify the goal with the head of a single rule, then proves its body.
(def prove-rule
  (lambda (goal rule subst all-rules depth)
    (let ((renamed-rule (rename-vars rule depth)))
      (let ((new-subst (unify goal (car renamed-rule) subst)))
        (cond
          ((failed-subst? new-subst) '())
          (t (let ((body-results (prove-goals (cdr renamed-rule) new-subst all-rules (+ depth 1))))
               (map-proofs (lambda (res) 
                             (list (car res) 
                                   (list 'proved goal (car renamed-rule) (cadr res)))) 
                           body-results))))))))

; Proves a conjunction (list) of goals sequentially, threading substitutions
; and accumulated proofs together as one `(subst proofs)` state through
; `thread-conjunction` (lib/unify.my) — the same conjunction-walking kernel
; `lib/forward.my`'s `match-conditions` threads a bare substitution through.
; `try-one` here is `prove-goal-state`: given one goal and the current
; `(subst proofs)`, it either handles `not` specially (negation as failure
; has no positive proof of its own — success means the inner goal could
; *not* be proved) or recursively searches `all-rules` for the goal like
; any other, in both cases appending exactly one new proof node to `proofs`.
; Returns a list of `(final-subst list-of-proofs)` pairs, byte-identical in
; shape and order to what this function returned before this refactor.
;
; Доводить кон'юнкцію (список) цілей послідовно, протягуючи підстановку й
; накопичені доведення разом як один стан `(підстановка доведення)` через
; `thread-conjunction` (lib/unify.my) — те саме ядро обходу кон'юнкції, крізь
; яке `match-conditions` у `lib/forward.my` протягує голу підстановку.
; `try-one` тут — `prove-goal-state`: для однієї цілі й поточного стану
; `(підстановка доведення)` вона або обробляє `not` окремо (заперечення
; через невдачу не має власного позитивного доведення — успіх означає, що
; внутрішню ціль *не вдалось* довести), або рекурсивно шукає ціль у
; `all-rules`, як і будь-яку іншу, в обох випадках дописуючи рівно один
; новий вузол доведення до `proofs`. Повертає список пар `(фінальна-підстановка
; список-доведень)`, побайтово ідентичний за формою й порядком тому, що ця
; функція повертала до цього рефакторингу.
;
; Beweist eine Konjunktion (Liste) von Zielen sequentiell und fädelt
; Substitution und angesammelte Beweise gemeinsam als einen Zustand
; `(substitution beweise)` durch `thread-conjunction` (lib/unify.my) — denselben
; Konjunktions-Durchlaufkern, durch den `match-conditions` in
; `lib/forward.my` eine bloße Substitution fädelt. `try-one` ist hier
; `prove-goal-state`: für ein Ziel und den aktuellen Zustand `(substitution
; beweise)` behandelt es entweder `not` gesondert (Negation als Fehlschlag
; hat keinen eigenen positiven Beweis — Erfolg bedeutet, dass das innere
; Ziel *nicht* bewiesen werden konnte) oder durchsucht rekursiv `all-rules`
; nach dem Ziel wie jedes andere, in beiden Fällen genau einen neuen
; Beweisknoten an `proofs` anhängend. Liefert eine Liste von
; `(end-substitution beweisliste)`-Paaren, byte-identisch in Form und
; Reihenfolge zu dem, was diese Funktion vor diesem Refactoring lieferte.
(def prove-goals
  (lambda (goals subst all-rules depth)
    (thread-conjunction goals (list subst '())
      (lambda (goal state) (prove-goal-state goal state all-rules depth)))))

(def prove-goal-state
  (lambda (goal state all-rules depth)
    (let ((subst (car state))
          (proofs (second state)))
      (cond
        ((eq (car goal) 'not)
         (let ((not-result (prove-goal (second goal) all-rules subst all-rules depth)))
           (cond
             ((atom not-result)
              ; The inner goal failed, so `not` succeeds!
              (list (list subst (append proofs (list (list 'proved-not (second goal)))))))
             (t '()))))
        (t (map-goal-results (prove-goal goal all-rules subst all-rules depth) proofs))))))

(def map-goal-results
  (lambda (results proofs)
    (cond
      ((atom results) '())
      (t (cons (list (car (car results)) (append proofs (list (second (car results)))))
                (map-goal-results (cdr results) proofs))))))

; Translates a proof tree into a readable trace via side-effecting `print`.
(def explain-proof
  (lambda (proof)
    (explain-proof-node proof 0)))

(def explain-proof-node
  (lambda (node level)
    (cond
      ((eq (car node) 'proved)
       (let* ((_2 (print 'Proved:))
              (_3 (print (second node)))
              (_4 (print 'using))
              (_5 (print 'rule:))
              (_6 (print (third node))))
         (explain-proof-list (cadddr node) (+ level 1))))
      ((eq (car node) 'proved-not)
       (let* ((_2 (print 'Proved))
              (_3 (print 'by))
              (_4 (print 'failure:))
              (_5 (print 'not))
              (_6 (print (second node))))
         '()))
      (t '()))))

(def explain-proof-list
  (lambda (nodes level)
    (cond
      ((atom nodes) '())
      (t (let* ((_1 (print-indent level))
                (_2 (print '|-))
                (_3 (explain-proof-node (car nodes) level)))
           (explain-proof-list (cdr nodes) level))))))

(def print-indent
  (lambda (level)
    (cond
      ((eq level 0) '())
      (t (let ((_ (print '..)))
           (print-indent (- level 1)))))))

; `reason` returning an empty list is ambiguous by itself: it could mean "known
; to be false" or just "not derivable from what's currently known" — the system
; has no way to say so out loud. `reason-explain` makes that distinction explicit:
; on success it explains the first proof (as `explain-proof` does); on failure it
; says so instead of silently returning nothing. This is the "I know X" vs "I
; could not prove X from what I know" distinction from private/lisp-to-knowledge.md §12.
;
; Порожній список від `reason` сам собою неоднозначний: він може означати "відомо,
; що хибне" або просто "не виводиться з поточних знань" — система не має способу
; сказати це вголос. `reason-explain` робить цю різницю явною: при успіху пояснює
; перше доведення (як `explain-proof`), при невдачі — прямо каже про це замість
; мовчазного порожнього списку. Це різниця "я знаю X" проти "я не зміг довести X
; з наявних знань" з private/lisp-to-knowledge.md §12.
;
; `reason` gibt bei Misserfolg eine leere Liste zurück, die für sich genommen
; mehrdeutig ist: sie könnte "bekanntermaßen falsch" oder einfach "aus dem
; aktuellen Wissen nicht ableitbar" bedeuten — das System kann das nicht laut
; sagen. `reason-explain` macht diesen Unterschied explizit: bei Erfolg erklärt
; es den ersten Beweis (wie `explain-proof`), bei Misserfolg sagt es das statt
; stillschweigend nichts zurückzugeben. Das ist der Unterschied zwischen "ich
; weiß X" und "ich konnte X aus meinem Wissen nicht beweisen" aus
; private/lisp-to-knowledge.md §12.
(def reason-explain
  (lambda (goal rules)
    (let ((results (reason goal rules)))
      (cond
        ((atom results)
         (let* ((_1 (print 'Cannot))
                (_2 (print 'prove:))
                (_3 (print goal)))
           '()))
        (t (explain-proof (second (car results))))))))

; A growing knowledge tree can silently become "dead" (Cyc-style write-only
; archive) if nothing ever measures whether its facts and rules are actually
; being reasoned over. `count-usage` walks a proof tree and turns that into a
; concrete number: an alist of `(rule-head . times-used)`, one entry per rule
; that contributed to the proof. `equal?` (not `eq`) compares rule heads
; because they're compound terms, not atoms.
;
; Дерево знань, що росте, може непомітно "померти" (write-only архів у стилі
; Cyc), якщо ніщо не вимірює, чи факти та правила справді використовуються в
; міркуваннях. `count-usage` обходить дерево доведення й перетворює це на
; конкретне число: alist `(голова-правила . скільки-разів-використано)`, по
; запису на кожне правило, що взяло участь у доведенні. `equal?` (не `eq`),
; бо голови правил — складені терми, не атоми.
;
; Ein wachsender Wissensbaum kann still "tot" werden (write-only Archiv im
; Cyc-Stil), wenn nichts misst, ob seine Fakten und Regeln tatsächlich
; verwendet werden. `count-usage` durchläuft einen Beweisbaum und macht daraus
; eine konkrete Zahl: eine Alist `(regelkopf . anzahl-verwendungen)`, ein
; Eintrag pro Regel, die zum Beweis beigetragen hat. `equal?` (nicht `eq`),
; weil Regelköpfe zusammengesetzte Terme sind, keine Atome.
(def add-usage
  (lambda (entry alist)
    (cond
      ((atom alist) (list entry))
      ((equal? (car (car alist)) (car entry))
       (cons (cons (car entry) (+ (cdr entry) (cdr (car alist)))) (cdr alist)))
      (t (cons (car alist) (add-usage entry (cdr alist)))))))

(def merge-usage
  (lambda (a b)
    (cond
      ((atom a) b)
      (t (merge-usage (cdr a) (add-usage (car a) b))))))

; `proved-not` nodes (from negation-as-failure) have no rule head of their
; own, so they contribute nothing here — only positive rule applications count.
(def count-usage
  (lambda (node)
    (cond
      ((eq (car node) 'proved)
       (add-usage (cons (third node) 1) (count-usage-list (cadddr node))))
      (t '()))))

(def count-usage-list
  (lambda (nodes)
    (cond
      ((atom nodes) '())
      (t (merge-usage (count-usage (car nodes)) (count-usage-list (cdr nodes)))))))

; --- statement provenance ------------------------------------------------
; `reason-explain` already separates "proved" from "cannot prove"; `provenance`
; goes one level deeper and turns a proof-tree node into an explicit
; `(statement goal (source fact|rule) (rule rule-head) (derived-from ...))`
; record — private/lisp-to-knowledge.md §12's "X, because A and B, by rule C"
; instead of a bare "X". Deliberately no `(confidence ...)` or `(time ...)`
; field: this engine is exact and deterministic (see the exact-number
; principle elsewhere in this project) — every proof is either found or not,
; so a numeric confidence would be invented precision, not a measured one.
;
; `reason-explain` вже розрізняє "доведено" від "не можу довести"; `provenance`
; йде на рівень глибше й перетворює вузол дерева доведення на явний запис
; `(statement ціль (source fact|rule) (rule голова-правила) (derived-from ...))`
; — "X, тому що A і B, за правилом C" з private/lisp-to-knowledge.md §12
; замість голого "X". Свідомо без полів `(confidence ...)`/`(time ...)`: цей
; рушій точний і детермінований (див. принцип точних чисел деінде в проєкті) —
; кожне доведення або знайдено, або ні, тож числова впевненість була б
; вигаданою точністю, не виміряною.
;
; `reason-explain` unterscheidet bereits "bewiesen" von "kann nicht beweisen";
; `provenance` geht eine Ebene tiefer und wandelt einen Beweisbaum-Knoten in
; einen expliziten Datensatz `(statement ziel (source fact|rule) (rule
; regelkopf) (derived-from ...))` um — "X, weil A und B, nach Regel C" statt
; nacktem "X" aus private/lisp-to-knowledge.md §12. Bewusst ohne Felder
; `(confidence ...)`/`(time ...)`: diese Engine ist exakt und deterministisch
; (siehe das Prinzip exakter Zahlen an anderer Stelle im Projekt) — jeder
; Beweis ist entweder gefunden oder nicht, eine numerische Konfidenz wäre
; erfundene statt gemessener Präzision.
(def source-of
  (lambda (node)
    (cond
      ((atom (cadddr node)) 'fact)
      (t 'rule))))

(def provenance
  (lambda (node)
    (cond
      ((eq (car node) 'proved)
       (list 'statement (second node)
             (list 'source (source-of node))
             (list 'rule (third node))
             (list 'derived-from (provenance-list (cadddr node)))))
      (t (list 'statement (second node) (list 'source 'not-proved))))))

(def provenance-list
  (lambda (nodes)
    (cond
      ((atom nodes) '())
      (t (cons (provenance (car nodes)) (provenance-list (cdr nodes)))))))
