from pathlib import Path
import re

GUARD = '''; #115 — Lisp owns the authority verdict.
; Usage: my-lisp scripts/authority-guard.lisp after CI writes change facts as
; Lisp data to tests/changed-host-tests.lisp. Host code transports only diff
; direction; this Lisp program owns the semantic allow/deny verdict.

(def second (lambda (x) (car (cdr x))))
(def third (lambda (x) (car (cdr (cdr x)))))

(def authority-rows (read-all (read-file "tests/authority-inventory.lisp")))
(def changed-host-tests (read-all (read-file "tests/changed-host-tests.lisp")))

(def find-authority
  (lambda (path rows)
    (cond
      ((atom rows) (structural-kind empty-list) ())
      ((equal? path (second (car rows))) (structural-relation same)
       (third (car rows)))
      (t t (find-authority path (cdr rows))))))

(def allowed-authority?
  (lambda (class)
    (cond
      ((eq class (quote observer)) (identity-relation same) t)
      ((eq class (quote mechanism)) (identity-relation same) t)
      (t t ()))))

(def authority-violation
  (lambda (path class change-kind)
    (list (quote semantic-authority-violation)
          path class change-kind
          "Host tests may observe mechanism; Lisp owns meaning. See #112/#113.")))

(def check-changes
  (lambda (changes)
    (cond
      ((atom changes) (structural-kind empty-list)
       (quote (authority-ok)))
      ((atom changes) (structural-kind pair)
       (let ((change (car changes)))
         (let ((path (second change))
               (change-kind (third change)))
           (let ((class (find-authority path authority-rows)))
             (cond
               ((eq change-kind (quote deletion-only)) (identity-relation same)
                (check-changes (cdr changes)))
               ((allowed-authority? class) t
                (check-changes (cdr changes)))
               (t t
                (authority-violation path class change-kind)))))))
      (t t
       (authority-violation (quote invalid-change-data)
                            (quote unknown)
                            (quote modified))))))

(def authority-verdict (check-changes changed-host-tests))

; Keep diagnostic output in a successful top-level form. The following
; top-level form may intentionally fail; transcript atomicity must not erase
; the already-completed diagnostic form.
(print authority-verdict)

(cond
  ((eq (car authority-verdict) (quote semantic-authority-violation))
   (identity-relation same)
   (car ()))
  (t t t))
'''

Path('scripts/authority-guard.lisp').write_text(GUARD, encoding='utf-8')

ci_path = Path('.github/workflows/ci.yml')
ci = ci_path.read_text(encoding='utf-8')
needle = '          git diff --name-only "$BASE_SHA" "$HEAD_SHA" > /tmp/changed.txt\n          cat /tmp/changed.txt'
replacement = '          git diff --name-only "$BASE_SHA" "$HEAD_SHA" > /tmp/changed.txt\n          git diff --numstat "$BASE_SHA" "$HEAD_SHA" > /tmp/changed-numstat.txt\n          cat /tmp/changed.txt'
if needle not in ci:
    raise SystemExit('classify diff block not found')
ci = ci.replace(needle, replacement, 1)

pattern = re.compile(
    r'          : > tests/changed-host-tests\.lisp\n.*?          \./target/debug/my-lisp scripts/authority-guard\.lisp',
    re.S,
)
new_block = '''          : > tests/changed-host-tests.lisp
          while IFS=$'\\t' read -r additions deletions path; do
            case "$path" in
              crates/*/tests/*.rs)
                change_kind=modified
                if [[ "$additions" == "0" && "$deletions" != "0" && "$deletions" != "-" ]]; then
                  change_kind=deletion-only
                fi
                printf '(change "%s" %s)\\n' "$path" "$change_kind" >> tests/changed-host-tests.lisp
                ;;
            esac
          done < /tmp/changed-numstat.txt
          ./target/debug/my-lisp scripts/authority-guard.lisp'''
ci, count = pattern.subn(lambda _: new_block, ci, count=1)
if count != 1:
    raise SystemExit(f'authority guard block replacements: {count}')
ci_path.write_text(ci, encoding='utf-8')

contract_path = Path('crates/my-lisp/tests/authority_guard_contract.rs')
contract = contract_path.read_text(encoding='utf-8')
old = '    assert!(guard.contains("(t ())"), "unknown/forbidden authority must fail closed");'
new = '    assert!(guard.contains("authority-verdict") && guard.contains("(car ())"),\n        "unknown/forbidden authority must still fail closed after a separate diagnostic form");'
if old not in contract:
    raise SystemExit('old fail-closed assertion not found')
contract_path.write_text(contract.replace(old, new, 1), encoding='utf-8')