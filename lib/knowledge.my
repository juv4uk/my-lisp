;; knowledge.my - A module system for the Lisp Knowledge Representation

;; --- append-only fact journal --------------------------------------------
;; Earlier, `*knowledge-base*` held one snapshot per module — every
;; `defmodule`/`tell-knowledge` call replaced it outright. That snapshot
;; could never answer "when did the module learn this?", and there was no
;; way to take a fact back at all (`retract-knowledge` didn't exist).
;; `*knowledge-journal*` replaces it as the single source of truth: a
;; flat, ever-growing list of `(tell module-name clause)`/`(retract
;; module-name clause)` events, newest first (the same `cons`-prepend
;; convention `*working-memory*`/`*usage-counts*` already use elsewhere in
;; this project). No timestamps — my-lisp has no clock primitive, and
;; inventing one here would be exactly the kind of unearned precision
;; McCarthy principle 6 warns against; list order (oldest to newest once
;; reversed) is the only ordering this journal actually has, and it's
;; honest about that. A module's current clause list is now a *projection*
;; over the journal (`module-clauses-now`), computed on demand, not stored
;; anywhere directly.
;;
;; Раніше `*knowledge-base*` тримав один знімок на модуль — кожен виклик
;; `defmodule`/`tell-knowledge` повністю його переписував. Той знімок
;; ніколи не міг відповісти "коли модуль це дізнався?", і не було способу
;; забрати факт назад узагалі (`retract-knowledge` не існував).
;; `*knowledge-journal*` замінює його як єдине джерело правди: плаский,
;; постійно зростаючий список подій `(tell module-name clause)`/`(retract
;; module-name clause)`, найновіші спершу (той самий `cons`-паттерн, що й
;; `*working-memory*`/`*usage-counts*` уже використовують деінде в
;; проєкті). Без часових міток — my-lisp не має примітиву годинника, і
;; вигадати його тут було б саме тією невиправданою точністю, проти якої
;; застерігає принцип 6 МакКарті; порядок списку (від найстарішого до
;; найновішого після розвороту) — єдиний порядок, який цей журнал
;; насправді має, і це чесно визнано. Поточний список clause модуля тепер
;; — *проекція* журналу (`module-clauses-now`), обчислена на вимогу, а не
;; збережена десь напряму.
;;
;; Früher hielt `*knowledge-base*` einen Schnappschuss pro Modul — jeder
;; `defmodule`/`tell-knowledge`-Aufruf ersetzte ihn vollständig. Dieser
;; Schnappschuss konnte nie beantworten "wann hat das Modul das gelernt?",
;; und es gab keine Möglichkeit, einen Fakt überhaupt zurückzunehmen
;; (`retract-knowledge` existierte nicht). `*knowledge-journal*` ersetzt
;; ihn als einzige Quelle der Wahrheit: eine flache, stetig wachsende
;; Liste von `(tell module-name clause)`/`(retract module-name
;; clause)`-Ereignissen, neueste zuerst (dieselbe `cons`-Präfix-Konvention,
;; die `*working-memory*`/`*usage-counts*` anderswo im Projekt bereits
;; verwenden). Keine Zeitstempel — my-lisp hat kein Uhr-Primitiv, und eines
;; hier zu erfinden wäre genau die unverdiente Präzision, vor der
;; McCarthy-Prinzip 6 warnt; die Listenreihenfolge (älteste zuerst nach
;; Umkehrung) ist die einzige Ordnung, die dieses Journal tatsächlich hat,
;; und das wird ehrlich anerkannt. Die aktuelle Clause-Liste eines Moduls
;; ist jetzt eine *Projektion* über das Journal (`module-clauses-now`), auf
;; Anfrage berechnet, nicht irgendwo direkt gespeichert.
(def *knowledge-journal* '())

;; clauses->tell-events turns a plain clause list into one `tell` event per
;; clause — ordinary my-lisp, not a macro, since nothing here needs the
;; top-level `def`-rebinding trick; only the macros below (which actually
;; grow `*knowledge-journal*`) do.
(def clauses->tell-events
  (lambda (module-name clauses)
    (map (lambda (clause) (list 'tell module-name clause)) clauses)))

;; defmodule registers a list of clauses under a specific module name —
;; same public shape as before, but now expands to pushing one `tell`
;; event per clause onto `*knowledge-journal*` instead of replacing a
;; snapshot. A behavior change worth naming explicitly, not hiding: calling
;; `defmodule` twice for the same name used to silently shadow the first
;; call; now it *accumulates* both calls' clauses, which is the more
;; honest reading of "append-only" — nothing an earlier call told the
;; journal is ever quietly discarded by a later one.
(defmacro defmodule (name rules)
  (list 'def '*knowledge-journal*
        (list 'append
              (list 'clauses->tell-events (list 'quote name) rules)
              '*knowledge-journal*)))

;; retract-knowledge is the capability the journal actually earns that the
;; old snapshot model never had at all: taking one clause back out of a
;; module. `clause` is left unquoted, the same convention `defmodule`'s
;; `rules` follows, so the caller decides whether to pass a literal or a
;; variable holding the clause to remove.
(defmacro retract-knowledge (module-name clause)
  (list 'def '*knowledge-journal*
        (list 'cons
              (list 'list (list 'quote 'retract) (list 'quote module-name) clause)
              '*knowledge-journal*)))

;; module-journal-events filters the whole journal down to one module's
;; own events, preserving their relative (newest-first) order.
(def module-journal-events
  (lambda (module-name journal)
    (cond
      ((atom journal) '())
      ((equal? (second (car journal)) module-name)
       (cons (car journal) (module-journal-events module-name (cdr journal))))
      (t (module-journal-events module-name (cdr journal))))))

;; module-known? distinguishes "no module by this name was ever told
;; anything" from "the module exists but every clause it was ever told has
;; since been retracted" — the second case must still project to an empty
;; clause list, not `Module-not-found`. Kept as its own check (not folded
;; into `module-clauses-now`) precisely so callers can tell those two
;; apart, the same distinction `reason-in`/`forward-in`/`describe` already
;; promised before the journal existed.
(def module-known?
  (lambda (module-name)
    (cond
      ((atom (module-journal-events module-name *knowledge-journal*)) '())
      (t t))))

;; apply-journal-event folds one event onto a running clause list: `tell`
;; adds the clause, `retract` removes one matching occurrence — reusing
;; `lib/forward.my`'s own `retract-fact` (`equal?`-based, removes the first
;; match) rather than writing a second copy of the same logic.
(def apply-journal-event
  (lambda (clauses event)
    (cond
      ((eq (car event) 'tell) (cons (third event) clauses))
      ((eq (car event) 'retract) (retract-fact (third event) clauses))
      (t clauses))))

;; module-clauses-now is the projection itself: a module's events, put
;; back into chronological (oldest-first) order and folded left to right
;; through `apply-journal-event`. Every reader of a module's clauses
;; (`reason-in`, `forward-in`, `describe`) goes through this now, instead
;; of reading a stored snapshot — the journal is the only thing actually
;; stored.
(def module-clauses-now
  (lambda (module-name)
    (reduce apply-journal-event '()
            (reverse (module-journal-events module-name *knowledge-journal*)))))

;; load-knowledge reads a file and loads its module definition
(defmacro load-knowledge (module-name)
  (list 'load module-name))

;; reason-in queries a specific module by name
(def reason-in
  (lambda (module-name goal)
    (cond
      ((module-known? module-name) (reason goal (module-clauses-now module-name)))
      (t 'Module-not-found))))

;; --- forward-chaining integration (lib/forward.my) ----------------------
;; `reason-in` asks a targeted question of a module (backward-chaining:
;; goal in, one proof out). `forward-in` asks a module to materialize
;; everything it can derive (forward-chaining: run-multi to a fixpoint).
;; No new clause format needed — a module's flat clause list from
;; `defmodule` (facts as zero-condition clauses, rules as `(head cond1
;; cond2 ...)`) is already exactly what `run-multi` expects: a fact is just
;; a "rule" whose empty condition list is trivially satisfied, firing it
;; unconditionally, which is a harmless no-op re-derivation of the fact
;; itself. That's why `forward-in` starts `run-multi` from an empty fact
;; list — the module's own facts bootstrap the fixpoint.
;;
;; forward-chaining інтеграція (lib/forward.my) — `reason-in` ставить
;; модулю конкретне питання (backward-chaining: ціль на вхід, одне
;; доведення на вихід). `forward-in` просить модуль матеріалізувати все,
;; що можна вивести (forward-chaining: run-multi до fixpoint). Новий
;; формат clause не потрібен — плаский список clause модуля з `defmodule`
;; (факти як clause без умов, правила як `(head cond1 cond2 ...)`) уже
;; точно те, чого чекає `run-multi`: факт — просто "правило" з порожнім
;; списком умов, який тривіально задоволений, тож застосовується
;; безумовно — нешкідливе повторне виведення самого факту. Тому
;; `forward-in` стартує `run-multi` з порожнього списку фактів — власні
;; факти модуля запускають fixpoint.
;;
;; Forward-Chaining-Integration (lib/forward.my) — `reason-in` stellt
;; einem Modul eine gezielte Frage (Backward-Chaining: Ziel rein, ein
;; Beweis raus). `forward-in` bittet ein Modul, alles zu materialisieren,
;; was es ableiten kann (Forward-Chaining: run-multi bis zum Fixpunkt).
;; Kein neues Clause-Format nötig — die flache Clause-Liste eines Moduls
;; aus `defmodule` (Fakten als Clauses ohne Bedingungen, Regeln als `(head
;; cond1 cond2 ...)`) ist bereits genau das, was `run-multi` erwartet: ein
;; Fakt ist einfach eine "Regel" mit leerer, trivial erfüllter
;; Bedingungsliste, die bedingungslos feuert — eine harmlose erneute
;; Ableitung des Fakts selbst. Deshalb startet `forward-in` `run-multi` mit
;; einer leeren Faktenliste — die eigenen Fakten des Moduls starten den
;; Fixpunkt.
(def forward-in
  (lambda (module-name)
    (cond
      ((module-known? module-name) (run-multi (module-clauses-now module-name) '()))
      (t 'Module-not-found))))

;; check-conflict checks if the negation of the first rule's head is
;; provable. Retract is deliberately exempt from this check (per explicit
;; agreement): conflict detection guards against *adding* contradictory
;; information, not against *removing* it — taking a clause back out can
;; never itself contradict anything already known.
(def check-conflict
  (lambda (module-name rules)
    (cond
      ((module-known? module-name)
       (let ((head (car (car rules))))
         (let ((proofs (reason-in module-name (list 'not head))))
           (cond
             ((atom proofs) '())
             (t t)))))
      (t '()))))

;; tell-knowledge adds new clauses to a module, creating it if it doesn't
;; exist yet — same conflict check as before, but on success it now pushes
;; `tell` events onto `*knowledge-journal*` instead of rebuilding a
;; snapshot; `module-clauses-now` picks up every clause any `tell-knowledge`
;; or `defmodule` call for this module ever contributed, in order.
(defmacro tell-knowledge (module-name rules)
  (list 'cond
        (list (list 'check-conflict (list 'quote module-name) rules)
              (list 'quote 'Conflict-detected))
        (list 't
              (list 'def '*knowledge-journal*
                    (list 'append
                          (list 'clauses->tell-events (list 'quote module-name) rules)
                          '*knowledge-journal*)))))

;; --- Advice ingestion boundary -----------------------------------------
;; `advise` is the guarded write path for knowledge from outside the trusted
;; source tree (controlled language today, an LLM translator later). It
;; accepts one clause as data, validates its whole shape, checks an explicit
;; opposite, and only then appends a journal event. Absence is not
;; contradiction: negation-as-failure in rule bodies remains separate from
;; explicitly stored `(not goal)` knowledge. Every result is structured data:
;; `(accepted ...)`, `(rejected ...)`, or `(conflict ...)`.
;;
;; `advise` — захищений шлях запису для знань з-за меж довіреного дерева
;; джерел (сьогодні з контрольованої мови, пізніше від LLM-перекладача). Він
;; приймає один clause як дані, перевіряє всю його форму, шукає явно задану
;; протилежність і лише тоді додає подію до журналу. Відсутність — не
;; суперечність: negation-as-failure у тілах правил лишається окремою від
;; явно збереженого знання `(not goal)`. Кожен результат — структуровані
;; дані: `(accepted ...)`, `(rejected ...)` або `(conflict ...)`.
;;
;; `advise` ist der geschützte Schreibpfad für Wissen von außerhalb des
;; vertrauenswürdigen Quellbaums (heute kontrollierte Sprache, später ein
;; LLM-Übersetzer). Es nimmt eine Clause als Daten entgegen, prüft ihre ganze
;; Form, sucht einen expliziten Gegensatz und hängt erst dann ein
;; Journalereignis an. Abwesenheit ist kein Widerspruch: Negation als
;; Fehlschlag in Regelrümpfen bleibt von explizit gespeichertem `(not goal)`-
;; Wissen getrennt. Jedes Ergebnis sind strukturierte Daten: `(accepted ...)`,
;; `(rejected ...)` oder `(conflict ...)`.

(def knowledge-proper-list?
  (lambda (value)
    (cond
      ((atom value)
       (cond ((eq value '()) t) (t '())))
      (t (knowledge-proper-list? (cdr value))))))

(def knowledge-terms-valid?
  (lambda (terms)
    (cond
      ((atom terms)
       (cond ((eq terms '()) t) (t '())))
      ((knowledge-term-valid? (car terms))
       (knowledge-terms-valid? (cdr terms)))
      (t '()))))

(def knowledge-term-valid?
  (lambda (term)
    (cond
      ((atom term) t)
      ((atom (car term))
       (cond
         ((eq (car term) 'var)
          (cond
            ((= (length term) 2) (symbol? (second term)))
            (t '())))
         ((knowledge-proper-list? term) (knowledge-terms-valid? term))
         (t '())))
      (t '()))))

(def knowledge-goal-valid?
  (lambda (goal)
    (cond
      ((atom goal) '())
      ((eq (knowledge-proper-list? goal) '()) '())
      ((eq (symbol? (car goal)) '()) '())
      ((eq (car goal) 'not)
       (cond
         ((= (length goal) 2) (knowledge-goal-valid? (second goal)))
         (t '())))
      (t (knowledge-terms-valid? (cdr goal))))))

(def knowledge-goals-valid?
  (lambda (goals)
    (cond
      ((atom goals)
       (cond ((eq goals '()) t) (t '())))
      ((knowledge-goal-valid? (car goals))
       (knowledge-goals-valid? (cdr goals)))
      (t '()))))

(def knowledge-clause-valid?
  (lambda (clause)
    (cond
      ((atom clause) '())
      ((eq (knowledge-proper-list? clause) '()) '())
      ((eq (knowledge-goal-valid? (car clause)) '()) '())
      (t (knowledge-goals-valid? (cdr clause))))))

;; Explicit opposites operate on heads, not whole clauses: a rule and a fact
;; may derive the same head, and either is sufficient evidence.
;; Явні протилежності працюють із головами, не з цілими clause: правило і
;; факт можуть вивести ту саму голову, і кожного достатньо як доказу.
;; Explizite Gegensätze arbeiten mit Köpfen statt ganzen Clauses: Regel und
;; Fakt können denselben Kopf ableiten; jeder ist als Beleg ausreichend.
(def opposite-knowledge-head
  (lambda (head)
    (cond
      ((eq (car head) 'not) (second head))
      (t (list 'not head)))))

(def advice-conflict-proof
  (lambda (module-name clause)
    (cond
      ((module-known? module-name)
       (reason-in module-name (opposite-knowledge-head (car clause))))
      (t '()))))

(def advice-decision
  (lambda (module-name clause)
    (cond
      ((eq (symbol? module-name) '())
       (list 'rejected (list 'reason 'invalid-module) (list 'input clause)))
      ((eq (knowledge-clause-valid? clause) '())
       (list 'rejected (list 'reason 'invalid-clause) (list 'input clause)))
      (t
       (let ((opposite (opposite-knowledge-head (car clause)))
             (proofs (advice-conflict-proof module-name clause)))
         (cond
           ((atom proofs)
            (list 'accepted (list 'module module-name) (list 'knowledge clause)))
           (t
            (list 'conflict
                  (list 'new clause)
                  (list 'existing opposite)
                  (list 'proof (car proofs))))))))))

;; The accepted branch evaluates `def` in the caller's frame, then returns
;; the structured decision. A helper lambda would write into a disposable
;; child frame, so sequencing intentionally lives in the macro expansion.
;; Прийнята гілка виконує `def` у фреймі викликача й повертає структуроване
;; рішення. Допоміжна lambda писала б у тимчасовий дочірній фрейм, тому
;; послідовність навмисно міститься безпосередньо в macro expansion.
;; Der akzeptierte Zweig führt `def` im Frame des Aufrufers aus und gibt die
;; strukturierte Entscheidung zurück. Eine Hilfs-Lambda würde in einen
;; kurzlebigen Kind-Frame schreiben; daher liegt die Sequenz im Makro selbst.
(defmacro advise (module-name clause)
  (list 'cond
        (list
          (list 'equal?
                (list 'car (list 'advice-decision (list 'quote module-name) clause))
                (list 'quote 'accepted))
          (list 'second
                (list 'list
                      (list 'def '*knowledge-journal*
                            (list 'append
                                  (list 'clauses->tell-events
                                        (list 'quote module-name)
                                        (list 'list clause))
                                  '*knowledge-journal*))
                      (list 'list
                            (list 'quote 'accepted)
                            (list 'list
                                  (list 'quote 'module)
                                  (list 'quote module-name))
                            (list 'list (list 'quote 'knowledge) clause)))))
        (list 't (list 'advice-decision (list 'quote module-name) clause))))

;; `advise-all` is the transactional companion to `advise`. Translators often
;; produce several mutually dependent clauses, so validating and writing them
;; one at a time could leave half an answer in the journal. The whole batch is
;; checked against the module plus the proposed clauses; only one journal
;; replacement happens after every clause passes and no opposite is derivable.
;;
;; `advise-all` — транзакційний відповідник `advise`. Перекладачі часто
;; породжують кілька взаємозалежних clause, тому поелементна перевірка й запис
;; могли б лишити в журналі половину відповіді. Увесь пакет перевіряється проти
;; модуля разом із запропонованими clause; журнал змінюється один раз лише після
;; успішної перевірки всіх елементів і відсутності вивідної протилежності.
;;
;; `advise-all` ist das transaktionale Gegenstück zu `advise`. Übersetzer
;; erzeugen oft mehrere voneinander abhängige Clauses; eine elementweise
;; Prüfung könnte daher eine halbe Antwort im Journal hinterlassen. Das ganze
;; Paket wird gegen das Modul samt vorgeschlagenen Clauses geprüft; das Journal
;; wird erst einmalig ersetzt, wenn alles gültig und kein Gegenteil ableitbar
;; ist.
(def knowledge-clauses-valid?
  (lambda (clauses)
    (cond
      ((atom clauses) (eq clauses '()))
      ((knowledge-clause-valid? (car clauses))
       (knowledge-clauses-valid? (cdr clauses)))
      (t '()))))

(def advice-negative-head-conflict
  (lambda (rules all-rules)
    (cond
      ((atom rules) '())
      (t
       (let ((head (car (car rules))))
         (cond
           ((eq (car head) 'not)
            (let ((positive (second head)))
              (let ((proofs (reason positive all-rules)))
                (cond
                  ((atom proofs)
                   (advice-negative-head-conflict (cdr rules) all-rules))
                  (t (list head positive (car proofs)))))))
           (t (advice-negative-head-conflict (cdr rules) all-rules))))))))

(def advice-batch-conflict
  (lambda (clauses remaining all-rules)
    (cond
      ((atom remaining)
       (let ((global (advice-negative-head-conflict all-rules all-rules)))
         (cond
           ((atom global) '())
           (t (list (car clauses) (car global) (third global))))))
      (t
       (let ((opposite (opposite-knowledge-head (car (car remaining)))))
         (let ((proofs (reason opposite all-rules)))
           (cond
             ((atom proofs)
              (advice-batch-conflict clauses (cdr remaining) all-rules))
             (t (list (car remaining) opposite (car proofs))))))))))

(def advice-all-decision
  (lambda (module-name clauses)
    (cond
      ((eq (symbol? module-name) '())
       (list 'rejected (list 'reason 'invalid-module) (list 'input clauses)))
      ((atom clauses)
       (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
      ((eq (knowledge-proper-list? clauses) '())
       (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
      ((eq (knowledge-clauses-valid? clauses) '())
       (list 'rejected (list 'reason 'invalid-clause) (list 'input clauses)))
      (t
       (let ((existing (cond
                         ((module-known? module-name)
                          (module-clauses-now module-name))
                         (t '()))))
         (let ((conflict (advice-batch-conflict clauses clauses
                                                (append clauses existing))))
           (cond
             ((atom conflict)
              (list 'accepted
                    (list 'module module-name)
                    (list 'knowledge clauses)))
             (t
              (list 'conflict
                    (list 'new (car conflict))
                    (list 'existing (second conflict))
                    (list 'proof (third conflict)))))))))))

;; As with `advise`, sequencing stays in the macro so `def` updates the
;; caller's persistent frame rather than a helper lambda's temporary frame.
;; Як і в `advise`, послідовність лишається в макросі, щоб `def` оновлював
;; сталий фрейм викликача, а не тимчасовий фрейм допоміжної lambda.
;; Wie bei `advise` bleibt die Sequenz im Makro, damit `def` den dauerhaften
;; Aufrufer-Frame statt des temporären Frames einer Hilfs-Lambda aktualisiert.
(defmacro advise-all (module-name clauses)
  (list 'cond
        (list
          (list 'equal?
                (list 'car
                      (list 'advice-all-decision
                            (list 'quote module-name) clauses))
                (list 'quote 'accepted))
          (list 'second
                (list 'list
                      (list 'def '*knowledge-journal*
                            (list 'append
                                  (list 'clauses->tell-events
                                        (list 'quote module-name) clauses)
                                  '*knowledge-journal*))
                      (list 'list
                            (list 'quote 'accepted)
                            (list 'list
                                  (list 'quote 'module)
                                  (list 'quote module-name))
                            (list 'list (list 'quote 'knowledge) clauses)))))
        (list 't
              (list 'advice-all-decision
                    (list 'quote module-name) clauses))))

;; --- Versioned knowledge-package interchange ---------------------------
;; A package is data, never executable code:
;; `((format . my-lisp-knowledge) (version 0 1) (module . astronomy)
;;   (clauses . (((planet earth)) ...)))`.
;; Keeping the envelope as an association list makes it readable by every
;; project that already understands S-expressions, while the independent
;; format version lets the envelope evolve without coupling it to a my-lisp
;; release or the language-contract version.
;;
;; Пакет є даними, а не виконуваним кодом. Association list читається кожним
;; проєктом, що вже розуміє S-вирази, а незалежна версія формату дозволяє
;; розвивати оболонку без прив'язки до релізу my-lisp чи language-contract.
;;
;; Ein Paket besteht aus Daten, niemals aus ausführbarem Code. Die
;; Assoziationsliste ist für jedes S-Ausdruck-Projekt lesbar; die unabhängige
;; Formatversion lässt die Hülle ohne Kopplung an my-lisp- oder
;; Sprachvertragsversionen wachsen.
(def *knowledge-package-version* '(0 1))

(def knowledge-package-entries-valid?
  (lambda (entries)
    (cond
      ((atom entries) (eq entries '()))
      ((atom (car entries)) '())
      ((symbol? (car (car entries)))
       (knowledge-package-entries-valid? (cdr entries)))
      (t '()))))

(def knowledge-package-field
  (lambda (name package)
    (let ((entry (assoc name package)))
      (cond ((atom entry) '()) (t (cdr entry))))))

(def make-knowledge-package
  (lambda (module-name clauses)
    (list (cons 'format 'my-lisp-knowledge)
          (cons 'version *knowledge-package-version*)
          (cons 'module module-name)
          (cons 'clauses clauses))))

(def knowledge-package-decision
  (lambda (package)
    (cond
      ((atom package)
       (list 'rejected (list 'reason 'invalid-package) (list 'input package)))
      ((eq (knowledge-proper-list? package) '())
       (list 'rejected (list 'reason 'invalid-package) (list 'input package)))
      ((eq (knowledge-package-entries-valid? package) '())
       (list 'rejected (list 'reason 'invalid-package) (list 'input package)))
      ((eq (knowledge-package-field 'format package) 'my-lisp-knowledge)
       (cond
         ((equal? (knowledge-package-field 'version package)
                  *knowledge-package-version*)
          (advice-all-decision (knowledge-package-field 'module package)
                               (knowledge-package-field 'clauses package)))
         (t (list 'rejected
                  (list 'reason 'unsupported-version)
                  (list 'version (knowledge-package-field 'version package))))))
      (t (list 'rejected
               (list 'reason 'invalid-package)
               (list 'input package))))))

;; Import uses the same atomic journal update as `advise-all`. The module name
;; is read from data at runtime, so this is a separate macro rather than a thin
;; call to `advise-all`, whose module argument is intentionally literal syntax.
;; Імпорт використовує те саме атомарне оновлення, що й `advise-all`; назва
;; модуля читається з даних під час виконання, тому це окремий макрос.
;; Der Import nutzt dasselbe atomare Journal-Update wie `advise-all`; der
;; Modulname kommt zur Laufzeit aus Daten, daher ist dies ein eigenes Makro.
(defmacro import-knowledge-package (package)
  (list 'cond
        (list
          (list 'equal?
                (list 'car (list 'knowledge-package-decision package))
                (list 'quote 'accepted))
          (list 'second
                (list 'list
                      (list 'def '*knowledge-journal*
                            (list 'append
                                  (list 'clauses->tell-events
                                        (list 'knowledge-package-field
                                              (list 'quote 'module) package)
                                        (list 'knowledge-package-field
                                              (list 'quote 'clauses) package))
                                  '*knowledge-journal*))
                      (list 'knowledge-package-decision package))))
        (list 't (list 'knowledge-package-decision package))))

(defmacro import-knowledge-file (path)
  (list 'import-knowledge-package
        (list 'read (list 'read-file path))))

;; Export is deliberately a plain function: unlike import it does not mutate
;; the knowledge journal. It validates the module and clauses, serializes the
;; canonical package with `write-to-string`, and writes exactly one expression.
;; Експорт не змінює журнал: перевіряє дані, канонічно серіалізує й записує.
;; Export verändert das Journal nicht: prüfen, kanonisch serialisieren, schreiben.
(def write-knowledge-package
  (lambda (path module-name clauses)
    (cond
      ((eq (symbol? module-name) '())
       (list 'rejected (list 'reason 'invalid-module) (list 'input module-name)))
      ((atom clauses)
       (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
      ((eq (knowledge-proper-list? clauses) '())
       (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
      ((eq (knowledge-clauses-valid? clauses) '())
       (list 'rejected (list 'reason 'invalid-clause) (list 'input clauses)))
      (t
       (let ((package (make-knowledge-package module-name clauses)))
         (second (list (write-file path (write-to-string package)) package)))))))

;; TCP transport uses one package per connection and EOF as the frame boundary.
;; TCP may split one write into many reads, so the receiver drains every chunk
;; before parsing exactly one S-expression. The sender closes only after
;; `tcp-write` succeeds. No received bytes are ever passed to `eval`.
;; TCP-транспорт: один пакет на з'єднання, EOF — межа; отримане ніколи не `eval`.
;; TCP-Transport: ein Paket pro Verbindung, EOF als Grenze; Empfang nie per `eval`.
(def tcp-read-to-eof
  (lambda (connection accumulated)
    (let ((chunk (tcp-read connection)))
      (cond
        ((string-empty? chunk) accumulated)
        (t (tcp-read-to-eof connection
                            (string-append accumulated chunk)))))))

(def send-knowledge-package
  (lambda (connection module-name clauses)
    (cond
      ((eq (symbol? module-name) '())
       (list 'rejected (list 'reason 'invalid-module) (list 'input module-name)))
      ((atom clauses)
       (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
      ((eq (knowledge-proper-list? clauses) '())
       (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
      ((eq (knowledge-clauses-valid? clauses) '())
       (list 'rejected (list 'reason 'invalid-clause) (list 'input clauses)))
      (t
       (let ((package (make-knowledge-package module-name clauses)))
         (third (list (tcp-write connection (write-to-string package))
                      (tcp-close connection)
                      package)))))))

(defmacro receive-knowledge-package (connection)
  (list 'second
        (list 'list
              (list 'def '*received-knowledge-package*
                    (list 'read (list 'tcp-read-to-eof connection "")))
              (list 'import-knowledge-package '*received-knowledge-package*))))

;; A newline-framed request/receipt protocol keeps the connection open long
;; enough for the receiver to answer with the structured import decision.
;; `write-to-string` escapes newlines inside string values, so a literal LF is
;; an unambiguous frame boundary. One request and one receipt per connection
;; means no unread suffix has to survive between calls.
;; Протокол із LF-frame лишає з'єднання відкритим для структурованої квитанції.
;; LF-gerahmtes Protokoll hält die Verbindung für eine strukturierte Quittung offen.
(def string-through-line
  (lambda (text accumulated)
    (cond
      ((string-empty? text) '())
      ((eq (string-first text) "\n") (list accumulated))
      (t (string-through-line
           (string-rest text)
           (string-append accumulated (string-first text)))))))

(def tcp-read-frame
  (lambda (connection accumulated)
    (let ((chunk (tcp-read connection)))
      (cond
        ((string-empty? chunk) '())
        (t
         (let ((line (string-through-line
                       (string-append accumulated chunk) "")))
           (cond
             ((atom line)
              (tcp-read-frame connection (string-append accumulated chunk)))
             (t (car line)))))))))

(def exchange-knowledge-package
  (lambda (connection module-name clauses)
    (cond
      ((eq (symbol? module-name) '())
       (list 'rejected (list 'reason 'invalid-module) (list 'input module-name)))
      ((atom clauses)
       (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
      ((eq (knowledge-proper-list? clauses) '())
       (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
      ((eq (knowledge-clauses-valid? clauses) '())
       (list 'rejected (list 'reason 'invalid-clause) (list 'input clauses)))
      (t
       (let ((package (make-knowledge-package module-name clauses)))
         (third
           (list
             (tcp-write connection
                        (string-append (write-to-string package) "\n"))
             (def *knowledge-receipt-text* (tcp-read-frame connection ""))
             (let ((receipt (read *knowledge-receipt-text*)))
               (second (list (tcp-close connection) receipt))))))))))

(defmacro accept-knowledge-exchange (connection)
  (list 'second
        (list 'list
              (list 'def '*received-knowledge-package*
                    (list 'read (list 'tcp-read-frame connection "")))
              (list 'let
                    (list (list 'decision
                                (list 'import-knowledge-package
                                      '*received-knowledge-package*)))
                    (list 'second
                          (list 'list
                                (list 'tcp-write connection
                                      (list 'string-append
                                            (list 'write-to-string 'decision)
                                            "\n"))
                                (list 'second
                                      (list 'list
                                            (list 'tcp-close connection)
                                            'decision))))))))

;; --- "atom as concept entry point" ------------------------------------
;; A bare symbol like `earth` doesn't mean anything by itself — its meaning
;; comes from the facts we've recorded around it. `describe` turns a symbol
;; into an entry point into everything currently known about it: every fact
;; in a module that mentions the symbol, collected as-is. Not a new kind of
;; storage — the same flat facts already stored by `defmodule`/`tell-knowledge`,
;; just queried from a symbol outward instead of from a goal downward.

;; contains-atom? checks whether `item` occurs among the elements of a fact's
;; argument list, e.g. `apple` in `(has-mass apple)`. `eq` only accepts atoms,
;; so non-atom elements (like a `(var x)` term inside a rule head) are simply
;; skipped rather than compared — `describe` only ever looks at facts, but
;; this keeps the helper safe to reuse against rule heads too.
(def contains-atom?
  (lambda (item lst)
    (cond
      ((atom lst) '())
      ((atom (car lst))
       (cond
         ((eq (car lst) item) t)
         (t (contains-atom? item (cdr lst)))))
      (t (contains-atom? item (cdr lst))))))

;; is-fact? — a clause with an empty body, i.e. a fact rather than a rule
;; with conditions. Facts are what `describe` reports; rules describe how
;; new facts get derived, not what's directly known about one symbol.
(def is-fact?
  (lambda (clause)
    (atom (cdr clause))))

;; collect-facts-about walks a module's clause list, keeping every fact
;; (not rule) whose head mentions `item`.
(def collect-facts-about
  (lambda (item clauses)
    (cond
      ((atom clauses) '())
      (t (let ((clause (car clauses)))
           (let ((head (car clause)))
             (cond
               ((eq (is-fact? clause) '()) (collect-facts-about item (cdr clauses)))
               ((eq (contains-atom? item head) '()) (collect-facts-about item (cdr clauses)))
               (t (cons head (collect-facts-about item (cdr clauses)))))))))))

;; describe returns every known fact about `item` within `module-name`,
;; or `Module-not-found` for consistency with `reason-in`.
(def describe
  (lambda (item module-name)
    (cond
      ((module-known? module-name) (collect-facts-about item (module-clauses-now module-name)))
      (t 'Module-not-found))))

;; --- usage tracking (which knowledge is actually alive) ----------------
;; A per-fact/per-rule usage counter, cheap enough to make "is this module
;; still being reasoned over, or has it gone Cyc-style dead" a measurable
;; question instead of a guess. `*usage-counts*` follows the same top-level
;; `def`-rebinding pattern as `*knowledge-base*` above — `def` only mutates
;; the frame it runs in, so accumulation has to happen at the call site
;; (top level), not inside a nested lambda call like `prove-rule`. That's
;; why `record-usage!` is a macro: it expands to a `(def *usage-counts* ...)`
;; form in the caller's own frame, the same trick `tell-knowledge` uses.
;;
;; Лічильник використання на факт/правило, достатньо дешевий, щоб "чи це
;; знання ще живе, чи вже стало мертвим у стилі Cyc" стало вимірюваним
;; питанням, а не здогадкою. `*usage-counts*` слідує тому самому
;; top-level `def`-паттерну перезапису, що й `*knowledge-base*` вище —
;; `def` мутує лише той фрейм, у якому виконується, тож накопичення має
;; відбуватись у місці виклику (на верхньому рівні), а не всередині
;; вкладеного виклику lambda типу `prove-rule`. Тому `record-usage!` —
;; макрос: він розгортається у форму `(def *usage-counts* ...)` у фреймі
;; викликача, той самий прийом, що й `tell-knowledge`.
;;
;; Ein Nutzungszähler pro Fakt/Regel, billig genug, um "lebt dieses Wissen
;; noch, oder ist es im Cyc-Stil tot" zu einer messbaren Frage statt einer
;; Vermutung zu machen. `*usage-counts*` folgt demselben Top-Level
;; `def`-Umbindungsmuster wie `*knowledge-base*` oben — `def` mutiert nur
;; den Frame, in dem es läuft, daher muss die Akkumulation am Aufrufort
;; (Top-Level) stattfinden, nicht innerhalb eines verschachtelten
;; Lambda-Aufrufs wie `prove-rule`. Deshalb ist `record-usage!` ein Makro:
;; es entfaltet sich zu einer `(def *usage-counts* ...)`-Form im Frame des
;; Aufrufers, derselbe Trick wie bei `tell-knowledge`.
(def *usage-counts* '())

(defmacro record-usage! (proof)
  (list 'def '*usage-counts*
        (list 'merge-usage (list 'count-usage proof) '*usage-counts*)))

(def usage-of
  (lambda (rule-head)
    (let ((entry (assoc rule-head *usage-counts*)))
      (cond
        ((atom entry) 0)
        (t (cdr entry))))))
