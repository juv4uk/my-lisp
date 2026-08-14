# Swarm build-cache & target-directory policy (WSL)

Status: adopted from audit SWARM-WSL-OFF-DRVFS-BUILD-CACHE-POLICY (2026-08-13)
Auditor: engineer-1

## Rule

All per-user build caches and build-target directories MUST live on the
native filesystem (under `/home/<user>/`), never on `/mnt/c` (DrvFs).

Rationale: DrvFs is slow and does not support `chmod`/`fchmod`
(`fchmod(fd, 0777) = -1 EPERM`), which breaks Cargo/toolchain builds
(e.g. my-idea's tauri-build ACL step). Builds that put `target/` or
caches inside repos on `/mnt/c` hit both problems.

## Per-tool rules

- Rust: `export CARGO_TARGET_DIR="$HOME/.cache/<project>-target"`.
  - my-idea: `~/.cache/my-idea-target` (in use). The stray duplicate
    `~/.cache/my-idea-target2` (828M) should be consolidated: pick one
    name, delete the other, rebuild.
  - my-lisp and cml: currently build into `/mnt/c/GitHub/<repo>/target`
    (DrvFs; 1.3G and 54M). Adopt `CARGO_TARGET_DIR` in their build env.
  - Keep `Cargo.lock` committed: targets are then always reproducible
    and safe to delete.
- Node/bun: caches already native (`~/.npm`, `~/.bun/install/cache`,
  `BUN_INSTALL=$HOME/.bun`). `node_modules` stays project-local (on
  DrvFs); it is disposable because `package-lock.json` / `bun.lock`
  reproduce it. `bun install` reuses the native tarball cache.
- Guix: already native (`/var/guix`, `~/.cache/guix`, per-user profiles
  under `/var/guix/profiles/per-user`). No action. Channel pin
  `5375f33` (channels.scm) keeps builds reproducible.
- fpga tools (iverilog/verilator/yosys/vvp): small native caches only;
  no DrvFs footprint. No action.

## Safe cleanup

Safe to delete (reproducible, no secrets):
- DrvFs `target/` dirs: `/mnt/c/GitHub/my-lisp/target` (1.3G),
  `/mnt/c/GitHub/my-idea/target` (810M), `/mnt/c/GitHub/cml/target` (54M)
- DrvFs `node_modules`: `/mnt/c/GitHub/tauricode/node_modules` (4.4G),
  `/mnt/c/GitHub/my-idea/node_modules` (215M)
- Native regenerable caches when space is needed:
  `~/.cargo/registry/cache`, `~/.npm/_cacache`, `~/.bun/install/cache`,
  `~/.cache/guix` (re-downloads, slow link — keep unless needed)
- NOT to delete: any `target/` binary currently running a swarm-node
  (e.g. `/mnt/c/GitHub/my-lisp/target/debug/swarm-node`) while that
  process lives; duplicate native target dirs (consolidate, don't blind
  delete until confirmed identical).

Never delete credential/secret-bearing files during cleanup
(`~/.ssh`, `~/.git-credentials`, npmrc tokens).

## Verification

- `guix shell -m manifest.scm bash coreutils --pure -- bash -c '...'`
  confirms repo toolchain; nothing from caches is needed.
- After cleanup, `cargo check` / `bun install` must work from a clean
  tree using only locks + native caches.

## Implementation (SWARM-WSL-OFF-DRVFS-CACHE-IMPLEMENTATION, 2026-08-13)

Deployed per-user via `~/.profile` (login shells) on all 6 swarm users
(user, my-lisp, my-idea, fpga-lisp, cml, my-lisp-panini). Block:

```sh
# bun on login PATH if installed for this user:
if [ -d "$HOME/.bun/bin" ]; then
    case ":$PATH:" in
        *":$HOME/.bun/bin:"*) ;;
        *) export PATH="$HOME/.bun/bin:$PATH" ;;
    esac
fi
# cargo target dir: keep off /mnt/c (DrvFs = slow + chmod EPERM).
# Safe-path guard: only apply when HOME/.cache is on the native fs.
case "$HOME" in
    /mnt/*) ;;
    *)
        if [ -z "${CARGO_TARGET_DIR:-}" ]; then
            export CARGO_TARGET_DIR="$HOME/.cache/${USER}-target"
        fi
        ;;
esac
```

Safe-path checks baked in: the guard skips the export entirely when
`$HOME` is under `/mnt/*` (DrvFs), and never overrides an existing
`CARGO_TARGET_DIR` (respects per-project overrides like my-idea's
`~/.cache/my-idea-target`).

Verified 2026-08-13:
- All 6 users (login shell): `CARGO_TARGET_DIR` = native
  `~/.cache/<user>-target`, none under `/mnt`; `bash -n` clean.
- bun now on login PATH for user (`~/.bun/bin/bun`) and my-idea;
  absent users correctly have no bun.
- Smoke build as my-lisp (`cargo new hello && cargo build -q`):
  binary produced at `/home/my-lisp/.cache/my-lisp-target/debug/hello`
  — cargo honours the default off-DrvFs target dir.

Stray duplicate `~/.cache/my-idea-target2` (828M) is still to be
consolidated per the per-tool rules above (defer to a cleanup window;
do not blind-delete).
