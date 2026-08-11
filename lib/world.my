;; An immutable world is ordinary my-lisp data:
;;   (world parent newest-first-knowledge-journal metadata)
;; Every transition returns a new value. The previous world remains its parent,
;; and the new journal shares the complete old journal as its cons tail. No
;; mutable host object or Rust primitive is needed.
;;
;; Незмінний світ — це звичайні дані my-lisp:
;;   (world батько журнал-знань-від-нових-до-старих метадані)
;; Кожен перехід повертає нове значення. Попередній світ лишається батьком,
;; а новий журнал структурно ділить увесь старий журнал як хвіст cons. Жодного
;; мутабельного об'єкта хоста чи нового Rust-примітива не потрібно.
;;
;; Eine unveränderliche Welt besteht aus gewöhnlichen my-lisp-Daten:
;;   (world vorgänger wissenjournal-neueste-zuerst metadaten)
;; Jeder Übergang liefert einen neuen Wert. Die vorige Welt bleibt ihr
;; Vorgänger, und das neue Journal teilt das vollständige alte Journal als
;; Cons-Ende. Kein veränderliches Hostobjekt und kein Rust-Primitiv ist nötig.

(def make-world
  (lambda (parent journal metadata)
    (list 'world parent journal metadata)))

(def empty-world
  (lambda ()
    (make-world '() '() '())))

(def world?
  (lambda (value)
    (cond
      ((atom value) '())
      ((eq (car value) 'world) t)
      (t '()))))

(def world-parent (lambda (world) (second world)))
(def world-journal (lambda (world) (third world)))
(def world-metadata (lambda (world) (fourth world)))

;; Events intentionally have the same data shape as `lib/knowledge.my`'s
;; journal, so a later migration can reuse packages and projections unchanged.
;; Події навмисно мають ту саму форму, що й журнал `lib/knowledge.my`.
;; Ereignisse haben absichtlich dieselbe Form wie in `lib/knowledge.my`.
(def world-record
  (lambda (world event)
    (make-world world
                (cons event (world-journal world))
                (world-metadata world))))

(def world-tell
  (lambda (world module-name clause)
    (world-record world (list 'tell module-name clause))))

(def world-retract
  (lambda (world module-name clause)
    (world-record world (list 'retract module-name clause))))

(def world-module-events
  (lambda (world module-name)
    (filter (lambda (event) (equal? (second event) module-name))
            (world-journal world))))

(def world-remove-first
  (lambda (value values)
    (cond
      ((atom values) '())
      ((equal? value (car values)) (cdr values))
      (t (cons (car values) (world-remove-first value (cdr values)))))))

(def world-apply-event
  (lambda (clauses event)
    (cond
      ((eq (car event) 'tell) (cons (third event) clauses))
      ((eq (car event) 'retract)
       (world-remove-first (third event) clauses))
      (t clauses))))

(def world-module-known?
  (lambda (world module-name)
    (cond
      ((atom (world-module-events world module-name)) '())
      (t t))))

(def world-clauses
  (lambda (world module-name)
    (reduce world-apply-event '()
            (reverse (world-module-events world module-name)))))

;; Pure reasoning adapters: the answer depends only on the explicit world,
;; module, and goal. They deliberately do not inspect `*knowledge-journal*`.
;; Чисті reasoning-адаптери: відповідь залежить лише від явно переданих світу,
;; модуля й цілі; глобальний `*knowledge-journal*` вони не читають.
;; Reine Schlussfolgerungsadapter: Die Antwort hängt nur von der expliziten
;; Welt, dem Modul und dem Ziel ab; `*knowledge-journal*` wird nicht gelesen.
(def reason-in-world
  (lambda (world module-name goal)
    (cond
      ((world-module-known? world module-name)
       (reason goal (world-clauses world module-name)))
      (t 'Module-not-found))))

(def forward-in-world
  (lambda (world module-name)
    (cond
      ((world-module-known? world module-name)
       (run-multi (world-clauses world module-name) '()))
      (t 'Module-not-found))))

;; `advise-world` keeps the established Advice Taker decision vocabulary but
;; makes the state transition explicit. Its result is always `(decision world)`:
;; accepted input carries a newly extended world; rejected or conflicting input
;; carries the exact original world. Validation helpers are shared with
;; `lib/knowledge.my`, so the global convenience API and this pure API cannot
;; silently develop different clause languages.
;;
;; `advise-world` зберігає чинний словник рішень Advice Taker, але робить
;; перехід стану явним. Результат завжди `(рішення світ)`: прийнятий ввід несе
;; новий розширений світ, відхилений або конфліктний — точно початковий світ.
;; Валідатори спільні з `lib/knowledge.my`, тому обидва API не розійдуться.
;;
;; `advise-world` behält das bestehende Entscheidungsvokabular des Advice
;; Takers bei, macht den Zustandsübergang jedoch explizit. Das Ergebnis ist
;; immer `(entscheidung welt)`: Akzeptierte Eingabe enthält eine neue Welt,
;; abgelehnte oder widersprüchliche Eingabe exakt die ursprüngliche Welt.
(def advice-decision-in-world
  (lambda (world module-name clause)
    (cond
      ((eq (symbol? module-name) '())
       (list 'rejected (list 'reason 'invalid-module) (list 'input clause)))
      ((eq (knowledge-clause-valid? clause) '())
       (list 'rejected (list 'reason 'invalid-clause) (list 'input clause)))
      (t
       (let ((opposite (opposite-knowledge-head (car clause))))
         (let ((proofs (cond
                         ((world-module-known? world module-name)
                          (reason-in-world world module-name opposite))
                         (t '()))))
           (cond
             ((atom proofs)
              (list 'accepted
                    (list 'module module-name)
                    (list 'knowledge clause)))
             (t
              (list 'conflict
                    (list 'new clause)
                    (list 'existing opposite)
                    (list 'proof (car proofs)))))))))))

(def advise-world
  (lambda (world module-name clause)
    (let ((decision (advice-decision-in-world world module-name clause)))
      (cond
        ((eq (car decision) 'accepted)
         (list decision (world-tell world module-name clause)))
        (t (list decision world))))))

;; The batch form creates exactly one child world after the whole proposed
;; knowledge set validates and proves conflict-free. Proposed rules can support
;; or contradict one another during the check; no prefix can leak on failure.
;; Пакетна форма створює рівно один дочірній світ лише після validation усього
;; набору й перевірки конфліктів; жоден префікс не просочується при помилці.
;; Die Stapelform erzeugt genau eine Kindwelt erst nach vollständiger Prüfung;
;; bei einem Fehler kann kein Präfix des Pakets durchsickern.
(def advice-all-decision-in-world
  (lambda (world module-name clauses)
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
                         ((world-module-known? world module-name)
                          (world-clauses world module-name))
                         (t '()))))
         (let ((conflict (advice-batch-conflict
                           clauses clauses (append clauses existing))))
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

(def world-tell-all
  (lambda (world module-name clauses)
    (make-world world
                (append (clauses->tell-events module-name clauses)
                        (world-journal world))
                (world-metadata world))))

;; Once `lib/world.my` is loaded, the legacy `defmodule` surface becomes a
;; thin compatibility wrapper over the explicit World transition. It rebuilds
;; only the legacy journal binding from the returned world, so existing
;; `reason-in`/`forward-in` callers keep exactly their old contract while the
;; write semantics now have one implementation: `world-tell-all`. The wrapper
;; can disappear after the remaining global writers migrate.
;;
;; Після завантаження `lib/world.my` старий `defmodule` стає тонкою сумісною
;; обгорткою над явним переходом World. Із поверненого світу він перевизначає
;; лише старий journal-binding: чинні `reason-in`/`forward-in` не змінюють
;; контракт, а семантика запису вже має одну реалізацію — `world-tell-all`.
;; Обгортку можна буде прибрати після міграції решти глобальних writer-ів.
;;
;; Sobald `lib/world.my` geladen ist, wird die alte `defmodule`-Oberfläche zu
;; einer dünnen Kompatibilitätshülle um den expliziten World-Übergang. Aus der
;; gelieferten Welt bindet sie nur das alte Journal neu; bestehende
;; `reason-in`/`forward-in`-Aufrufer behalten ihren Vertrag, während
;; `world-tell-all` die einzige Schreibsemantik liefert. Nach der Migration der
;; übrigen globalen Writer kann diese Hülle entfallen.
(defmacro defmodule (name rules)
  (list 'def '*knowledge-journal*
        (list 'world-journal
              (list 'world-tell-all
                    (list 'make-world '() '*knowledge-journal* '())
                    (list 'quote name)
                    rules))))

;; The other two legacy journal macros follow the same bridge. `tell-knowledge`
;; deliberately keeps its established pre-write conflict check and delegates
;; only the accepted transition; `retract-knowledge` remains unconditional
;; because removing knowledge cannot introduce a contradiction. Both rebuild
;; the compatibility journal from a returned immutable World.
;;
;; Інші два legacy journal-макроси переходять тим самим мостом.
;; `tell-knowledge` зберігає чинну перевірку конфлікту перед записом і делегує
;; лише прийнятий перехід; `retract-knowledge` лишається безумовним, бо
;; вилучення знання не створює суперечності. Обидва відновлюють compatibility-
;; журнал із поверненого незмінного World.
;;
;; Die beiden anderen alten Journal-Makros nutzen dieselbe Brücke.
;; `tell-knowledge` behält seine Konfliktprüfung vor dem Schreiben und
;; delegiert nur den akzeptierten Übergang; `retract-knowledge` bleibt
;; bedingungslos, weil Wissensentzug keinen Widerspruch erzeugen kann. Beide
;; gewinnen das Kompatibilitätsjournal aus einer unveränderlichen World zurück.
(defmacro tell-knowledge (module-name rules)
  (list 'cond
        (list (list 'check-conflict (list 'quote module-name) rules)
              (list 'quote 'Conflict-detected))
        (list 't
              (list 'def '*knowledge-journal*
                    (list 'world-journal
                          (list 'world-tell-all
                                (list 'make-world '() '*knowledge-journal* '())
                                (list 'quote module-name)
                                rules))))))

(defmacro retract-knowledge (module-name clause)
  (list 'def '*knowledge-journal*
        (list 'world-journal
              (list 'world-retract
                    (list 'make-world '() '*knowledge-journal* '())
                    (list 'quote module-name)
                    clause))))

(def advise-all-world
  (lambda (world module-name clauses)
    (let ((decision (advice-all-decision-in-world world module-name clauses)))
      (cond
        ((eq (car decision) 'accepted)
         (list decision (world-tell-all world module-name clauses)))
        (t (list decision world))))))

;; A guarded compatibility transition must be evaluated exactly once. `let`
;; cannot hold it because `def` would then update a disposable lambda frame, so
;; this expansion uses one explicit top-level scratch binding. The argument and
;; pure World transition run once; only accepted results rebind the journal.
;; Guarded compatibility-перехід обчислюється рівно раз. `let` не підходить, бо
;; `def` оновив би тимчасовий lambda-frame, тому expansion має одне явне
;; top-level scratch binding. Аргумент і World-перехід виконуються один раз.
;; Ein geschützter Kompatibilitätsübergang wird genau einmal ausgewertet. `let`
;; eignet sich nicht, da `def` sonst einen temporären Lambda-Frame ändert; diese
;; Expansion nutzt daher eine ausdrückliche Top-Level-Zwischenbindung.
(def legacy-world-transition-expansion
  (lambda (transition)
    (list 'second
          (list 'list
                (list 'def '*legacy-knowledge-transition* transition)
                (list 'cond
                      (list
                        (list 'equal?
                              (list 'car
                                    (list 'car '*legacy-knowledge-transition*))
                              (list 'quote 'accepted))
                        (list 'second
                              (list 'list
                                    (list 'def '*knowledge-journal*
                                          (list 'world-journal
                                                (list 'second
                                                      '*legacy-knowledge-transition*)))
                                    (list 'car '*legacy-knowledge-transition*))))
                      (list 't
                            (list 'car '*legacy-knowledge-transition*)))))))

(defmacro advise (module-name clause)
  (legacy-world-transition-expansion
    (list 'advise-world
          (list 'make-world '() '*knowledge-journal* '())
          (list 'quote module-name)
          clause)))

(defmacro advise-all (module-name clauses)
  (legacy-world-transition-expansion
    (list 'advise-all-world
          (list 'make-world '() '*knowledge-journal* '())
          (list 'quote module-name)
          clauses)))

;; World interchange reuses the established, versioned `my-lisp-knowledge`
;; envelope. Export reads one explicit snapshot. Import validates the envelope
;; as data and then delegates the only possible transition to
;; `advise-all-world`; received data is never evaluated as code.
;; World-обмін перевикористовує чинну версіоновану оболонку
;; `my-lisp-knowledge`. Експорт читає явний snapshot, імпорт передає єдиний
;; можливий перехід в `advise-all-world`; отримані дані ніколи не eval-код.
;; Der World-Austausch nutzt die bestehende versionierte Hülle
;; `my-lisp-knowledge`. Export liest einen expliziten Schnappschuss, Import
;; delegiert den einzigen Übergang an `advise-all-world`; Daten werden nie evaluiert.
(def make-world-knowledge-package
  (lambda (world module-name)
    (cond
      ((eq (symbol? module-name) '())
       (list 'rejected (list 'reason 'invalid-module) (list 'input module-name)))
      ((eq (world-module-known? world module-name) '()) 'Module-not-found)
      (t
       (let ((clauses (world-clauses world module-name)))
         (cond
           ((atom clauses)
            (list 'rejected (list 'reason 'invalid-batch) (list 'input clauses)))
           (t (make-knowledge-package module-name clauses))))))))

(def import-knowledge-package-world
  (lambda (world package)
    (cond
      ((atom package)
       (list (list 'rejected
                   (list 'reason 'invalid-package)
                   (list 'input package))
             world))
      ((eq (knowledge-proper-list? package) '())
       (list (list 'rejected
                   (list 'reason 'invalid-package)
                   (list 'input package))
             world))
      ((eq (knowledge-package-entries-valid? package) '())
       (list (list 'rejected
                   (list 'reason 'invalid-package)
                   (list 'input package))
             world))
      ((eq (knowledge-package-field 'format package) 'my-lisp-knowledge)
       (cond
         ((equal? (knowledge-package-field 'version package)
                  *knowledge-package-version*)
          (advise-all-world world
                            (knowledge-package-field 'module package)
                            (knowledge-package-field 'clauses package)))
         (t
          (list (list 'rejected
                      (list 'reason 'unsupported-version)
                      (list 'version
                            (knowledge-package-field 'version package)))
                world))))
      (t
       (list (list 'rejected
                   (list 'reason 'invalid-package)
                   (list 'input package))
             world)))))

;; Package import completes the writer migration. The legacy macro delegates
;; validation, version handling, conflict detection, and the atomic transition
;; to the data-only World importer; only an accepted result rebinds the journal.
;; Імпорт пакетів завершує міграцію writer-ів: validation, version handling,
;; конфлікти й атомарний перехід делеговано data-only World-імпортеру; журнал
;; перевизначається лише для accepted.
;; Der Paketimport schließt die Writer-Migration ab: Prüfung, Versionierung,
;; Konflikte und atomarer Übergang liegen beim datenreinen World-Importer; nur
;; ein akzeptiertes Ergebnis bindet das Journal neu.
(defmacro import-knowledge-package (package)
  (legacy-world-transition-expansion
    (list 'import-knowledge-package-world
          (list 'make-world '() '*knowledge-journal* '())
          package)))

;; History navigation is derived from parent links, not timestamps. Depth is
;; absolute from the root (`empty-world` = 0). `world-diff from to` returns the
;; chronological event list only when `from` is an ancestor of `to`; divergent
;; branches return `World-not-ancestor` until an explicit merge law exists.
;;
;; Навігація історією спирається на батьківські зв'язки, не timestamps. Глибина
;; абсолютна від кореня (`empty-world` = 0). `world-diff from to` повертає
;; хронологічні події лише для предка; різні гілки дають `World-not-ancestor`,
;; доки не визначено чесний закон merge.
;;
;; Geschichtsnavigation folgt Vorgängerlinks statt Zeitstempeln. Die Tiefe ist
;; absolut ab der Wurzel (`empty-world` = 0). `world-diff from to` liefert
;; chronologische Ereignisse nur für einen Vorfahren; getrennte Zweige ergeben
;; `World-not-ancestor`, bis ein ausdrückliches Merge-Gesetz definiert ist.
(def world-depth
  (lambda (world)
    (cond
      ((atom (world-parent world)) 0)
      (t (+ 1 (world-depth (world-parent world)))))))

(def world-at-depth-from
  (lambda (world current-depth target-depth)
    (cond
      ((= current-depth target-depth) world)
      ((< current-depth target-depth) 'World-not-found)
      ((atom (world-parent world)) 'World-not-found)
      (t (world-at-depth-from (world-parent world)
                              (- current-depth 1)
                              target-depth)))))

(def world-at-depth
  (lambda (world target-depth)
    (cond
      ((< target-depth 0) 'World-not-found)
      (t (world-at-depth-from world (world-depth world) target-depth)))))

(def world-journal-prefix
  (lambda (journal old-journal)
    (cond
      ((equal? journal old-journal) '())
      ((atom journal) 'World-not-ancestor)
      (t
       (let ((rest (world-journal-prefix (cdr journal) old-journal)))
         (cond
           ((world-not-ancestor? rest) rest)
           (t (cons (car journal) rest))))))))

(def world-not-ancestor?
  (lambda (value)
    (cond
      ((atom value) (eq value 'World-not-ancestor))
      (t '()))))

(def world-diff
  (lambda (from to)
    (cond
      ((equal? from to) '())
      ((atom (world-parent to)) 'World-not-ancestor)
      (t
       (let ((earlier (world-diff from (world-parent to))))
         (cond
           ((world-not-ancestor? earlier) earlier)
           (t
            (let ((transition
                    (world-journal-prefix
                      (world-journal to)
                      (world-journal (world-parent to)))))
              (cond
                ((world-not-ancestor? transition) transition)
                (t (append earlier transition)))))))))))

;; Branch comparison stops before merge policy. First align both histories to
;; the same absolute depth, then walk parents together until their values are
;; equal. Since worlds are immutable values, structurally equal histories are
;; the same semantic world even if reconstructed independently from a package.
;; `world-branch-diff` exposes the common base plus both chronological deltas.
;;
;; Порівняння гілок зупиняється до merge-policy. Історії вирівнюються за
;; абсолютною глибиною й разом ідуть до рівного предка. Для immutable-значень
;; структурно рівні історії — той самий семантичний світ. `world-branch-diff`
;; показує спільну базу та обидві хронологічні дельти.
;;
;; Der Zweigvergleich endet vor einer Merge-Policy. Beide Geschichten werden
;; auf gleiche Tiefe gebracht und gemeinsam bis zum gleichen Vorfahren verfolgt.
;; Bei unveränderlichen Werten sind strukturell gleiche Geschichten dieselbe
;; semantische Welt. `world-branch-diff` zeigt Basis und beide Zeitdeltas.
(def world-climb-to-depth
  (lambda (world current-depth target-depth)
    (cond
      ((= current-depth target-depth) world)
      (t (world-climb-to-depth (world-parent world)
                               (- current-depth 1)
                               target-depth)))))

(def world-common-ancestor-aligned
  (lambda (left right)
    (cond
      ((equal? left right) left)
      ((atom (world-parent left)) 'World-no-common-ancestor)
      ((atom (world-parent right)) 'World-no-common-ancestor)
      (t (world-common-ancestor-aligned (world-parent left)
                                        (world-parent right))))))

(def world-common-ancestor
  (lambda (left right)
    (let ((left-depth (world-depth left))
          (right-depth (world-depth right)))
      (let ((target-depth (cond
                            ((< left-depth right-depth) left-depth)
                            (t right-depth))))
        (world-common-ancestor-aligned
          (world-climb-to-depth left left-depth target-depth)
          (world-climb-to-depth right right-depth target-depth))))))

(def world-no-common-ancestor?
  (lambda (value)
    (cond
      ((atom value) (eq value 'World-no-common-ancestor))
      (t '()))))

(def world-branch-diff
  (lambda (left right)
    (let ((base (world-common-ancestor left right)))
      (cond
        ((world-no-common-ancestor? base) base)
        (t
         (list (list 'base base)
               (list 'left (world-diff base left))
               (list 'right (world-diff base right))))))))

;; Content identity starts with a canonical address, not a premature hash
;; primitive. `write-to-string` is deterministic and read-back-safe, so equal
;; knowledge has exactly the same address on every conforming implementation.
;; A world's address covers its complete event journal plus metadata; parent is
;; omitted because the journal already contains the whole history, avoiding a
;; recursively duplicated serialization. This is an exact key, not a fixed-size
;; cryptographic digest. A future SHA layer may hash this key without changing
;; what identity means.
;;
;; Content-ідентичність починається з канонічної адреси, не передчасного hash-
;; примітива. `write-to-string` детермінований і read-back-safe, тому рівне
;; знання має ту саму адресу в кожній conforming-реалізації. Адреса світу
;; охоплює весь журнал подій і metadata; parent не дублюється рекурсивно.
;; Це точний ключ, не криптографічний digest; майбутній SHA лише стисне ключ.
;;
;; Inhaltsidentität beginnt mit einer kanonischen Adresse statt einem
;; voreiligen Hash-Primitiv. `write-to-string` ist deterministisch und
;; rücklesbar, daher hat gleiches Wissen in jeder konformen Implementierung
;; dieselbe Adresse. Die Weltadresse umfasst Journal und Metadaten; der
;; Vorgänger wird nicht rekursiv dupliziert. Dies ist ein exakter Schlüssel,
;; kein kryptographischer Digest; ein späteres SHA kann diesen Schlüssel kürzen.
(def knowledge-content-address
  (lambda (knowledge)
    (write-to-string knowledge)))

(def world-address-content
  (lambda (world)
    (list 'world-history
          (world-journal world)
          (world-metadata world))))

(def world-content-address
  (lambda (world)
    (knowledge-content-address (world-address-content world))))
