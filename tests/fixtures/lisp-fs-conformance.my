; Deterministic WSM filesystem prototype fixture.
; Детермінований fixture прототипу файлової системи WSM.
;
; The result records old-root visibility, explicit missing status, stored nil,
; content deduplication, and logical revisions in one implementation-neutral
; observation.
; Результат фіксує видимість старого кореня, явний not-found, збережений nil,
; дедуплікацію вмісту та логічні ревізії як спостереження.

(let ((empty (fs-empty)))
  (let ((first-write (fs-write empty "notes/today" (quote (hello world)))))
    (let ((old (car first-write))
          (second-write (fs-write (car first-write) "notes/empty" (quote ()))))
      (let ((new (car second-write)))
        (list
          (fs-read old "notes/today")
          (fs-read old "notes/empty")
          (fs-read new "notes/empty")
          (fs-read new "missing")
          (content-store-size (fs-objects new))
          (fs-revision old)
          (fs-revision new))))))
