;; An immutable content-addressed store is a persistent map from the canonical
;; address defined in `lib/world.my` to ordinary my-lisp data. Re-inserting the
;; same content replaces the same key rather than growing the store. Worlds are
;; stored as `(world-history journal metadata)`, not with recursively duplicated
;; parent snapshots; navigation remains the responsibility of live World values.
;;
;; Незмінний content-addressed store — persistent map від канонічної адреси з
;; `lib/world.my` до звичайних my-lisp-даних. Повторний однаковий вміст замінює
;; той самий ключ і не ростить store. Світ зберігається як `(world-history
;; journal metadata)`, без рекурсивного дублювання parent snapshots.
;;
;; Ein unveränderlicher content-addressed Store ist eine persistente Map von der
;; kanonischen Adresse aus `lib/world.my` zu gewöhnlichen my-lisp-Daten. Gleicher
;; Inhalt ersetzt denselben Schlüssel statt den Store zu vergrößern. Welten
;; werden als `(world-history journal metadata)` ohne rekursive Elternkopien
;; gespeichert.

(def empty-content-store
  (lambda () map-empty))

(def content-store-put
  (lambda (store value)
    (map-insert (knowledge-content-address value) value store)))

(def content-store-get
  (lambda (store address)
    (map-get address store)))

(def content-store-contains?
  (lambda (store address)
    (map-contains? address store)))

(def content-store-put-world
  (lambda (store world)
    (map-insert (world-content-address world)
                (world-address-content world)
                store)))

(def content-store-size
  (lambda (store)
    (length (map->list store))))
