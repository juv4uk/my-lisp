; #291 — exact quantity arithmetic laws are owned by Lisp.
; This witness preserves both the surviving Planck×Cs exact-energy law and
; the retired speed-of-light product->quotient inverse law before the last
; Rust semantic test for this slice is deleted.

(load "lib/quantity.lisp")
(load "lib/si.lisp")

(def exact-quantity-arithmetic-rows
  (lambda ()
    (let* ((planck
             (scientific-constant-quantity si:defining-planck-constant))
           (cesium
             (scientific-constant-quantity si:defining-cesium-frequency))
           (energy
             (quantity-product planck cesium))
           (one-second
             (make-quantity
               1
               (make-unit
                 (list (make-dimension (quote second) 1)))))
           (speed
             (scientific-constant-quantity si:defining-speed-of-light))
           (distance
             (quantity-product speed one-second))
           (recovered
             (quantity-quotient distance one-second)))
      (list
        (list
          (quote planck-cesium-energy-shape)
          (equal?
            energy
            (quote
              (quantity/1
                121822045942277331/20000000000000000000000000000000000000000
                (unit/1
                  (dimension/1 kilogram 1)
                  (dimension/1 metre 2)
                  (dimension/1 second -2)))))
          (quote (structural-relation same)))
        (list
          (quote speed-times-second-distance-shape)
          (equal?
            distance
            (quote
              (quantity/1
                299792458
                (unit/1 (dimension/1 metre 1)))))
          (quote (structural-relation same)))
        (list
          (quote recovered-speed-shape)
          (equal?
            recovered
            (quote
              (quantity/1
                299792458
                (unit/1
                  (dimension/1 metre 1)
                  (dimension/1 second -1)))))
          (quote (structural-relation same)))
        (list
          (quote quotient-inverse)
          (equal? recovered speed)
          (quote (structural-relation same)))))))

(def exact-quantity-arithmetic-check
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (quote (exact-quantity-arithmetic-witness (status pass))))
      ((atom rows) (structural-kind atom)
       (list
         (quote exact-quantity-arithmetic-witness)
         (list (quote status) (quote fail))
         (list (quote case) (quote malformed-row-tail))
         (list (quote actual) rows)))
      ((atom rows) (structural-kind pair)
       (let ((row (car rows)))
         (cond
           ((equal? (second row) (third row)) (structural-relation same)
            (exact-quantity-arithmetic-check (cdr rows)))
           ((equal? (second row) (third row)) (structural-relation distinct)
            (list
              (quote exact-quantity-arithmetic-witness)
              (list (quote status) (quote fail))
              (list (quote case) (car row))
              (list (quote actual) (second row))
              (list (quote expected) (third row))))))))))

(def exact-quantity-arithmetic-witness
  (lambda ()
    (exact-quantity-arithmetic-check (exact-quantity-arithmetic-rows))))

(exact-quantity-arithmetic-witness)
