; memory-layout-contract.my — shared memory layout contract across the ecosystem
; Спільний контракт структури пам'яті для всієї екосистеми.
; Gemeinsamer Speicherlayout-Vertrag für das gesamte Ökosystem.
;
; This defines a 64-bit IEEE 754 NaN-boxing memory layout for Lisp values.
; It extends the 32-bit fpga-lisp ISA tags (which live in the lower 32 bits)
; and provides a unified representation for strings, rationals, and inexact
; numbers across my-lisp, cml, and fpga-lisp.

((kind . memory-layout-contract)
 (version . (1 0))
 (format . nan-boxing-64)
 
 (nan-marker . ((bits . 12) (position . (63 52)) (value . #xfff)))
 
 (fpga-lisp-compatibility . 
  "The lower 32 bits perfectly match the fpga-lisp 32-bit ISA word.
   bits (31 28) = tag
   bits (27 0) = payload")
 
 (layout . ((float . "IEEE-754 64-bit double (when exponent is not all 1s)")
            (tagged . ((nan-marker . (63 52))
                       (extended-payload . (51 32))
                       (tag . (31 28))
                       (payload . (27 0))))))

 (tags . ((fixnum . 0)
          (cons . 1)
          (symbol . 2)
          (nil . 3)
          (true . 4)
          (primitive . 5)
          (string . 6)    ; New: extended-payload + payload = 48-bit pointer to heap
          (rational . 7)  ; New: extended-payload + payload = 48-bit pointer to heap
          (closure . 8)   ; New: extended-payload + payload = 48-bit pointer to heap
          (tcp-conn . 9))) ; New: host resource handle

 (heap-representation . 
  ((string . "null-terminated UTF-8 byte array")
   (rational . "two consecutive 64-bit pointers (numerator, denominator) to BigInt blocks")
   (closure . "two consecutive 64-bit pointers (environment, compiled-code-pointer)")))

 (notes . "By defining this contract, cml can now safely emit pointers with tag 6, 7, and 8, and my-lisp will transition its enum Value to u64 under the hood."))
