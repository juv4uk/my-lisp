---
name: GNU Guix Reference
description: Essential commands and concepts for using GNU Guix in the my-lisp ecosystem (time-machine, channels, manifests).
---

# GNU Guix Guide for my-lisp Ecosystem

This repository relies on **GNU Guix** to provide exactly reproducible environments across different development machines and Continuous Integration.

## Core Concepts

### 1. Channels (`channels.scm`)
Channels define the Git repositories and specific revisions used to provide packages.
- Our ecosystem pins the Guix revision in `channels.scm`.
- **Purpose**: Instead of relying on whatever the latest version of Rust or GCC is, `channels.scm` guarantees you are building with the exact same toolchain versions every time.

### 2. Manifests (`manifest.scm`)
A manifest is a declarative Scheme file listing the packages needed for the environment (e.g., `rust`, `gcc-toolchain`, `pkg-config`).
- **Purpose**: Using `manifest.scm` removes the need for imperative package installation. 

### 3. Time-Machine
The `guix time-machine` command fetches a historic version of Guix (as defined by `channels.scm`) and executes a command (like `guix shell`) inside that exact environment.
- **Purpose**: This is the core engine for reproducibility. It allows us to travel back in time to the exact state of the package definitions pinned in `channels.scm`.

## Essential Commands

### Enter the Development Environment
To work in this repository, you should enter the Guix shell using the pinned channels and manifest.

```bash
# In WSL or a Linux environment
guix time-machine -C channels.scm -- shell -m manifest.scm
```

### Run Commands in the Environment
You can run a single command (like testing or building) directly without entering the interactive shell:

```bash
guix time-machine -C channels.scm -- shell -m manifest.scm -- cargo test --workspace
```

### Updating the Channel Pin
If you need to update the toolchain (e.g., to get a newer Rust version), you first update your local Guix, then export the new channel revision, verify it builds, and commit it:

```bash
# 1. Update local guix
guix pull

# 2. Export the new current state to a file
guix describe -f channels > channels-new.scm

# 3. Test if the project still builds
guix time-machine -C channels-new.scm -- shell -m manifest.scm -- cargo test --workspace

# 4. If successful, replace the old channels.scm
mv channels-new.scm channels.scm
```

## Agent Guidelines
1. **Never use `apt-get` or `dnf`** to install dependencies for `my-lisp`. The environment must be reproducible via Guix.
2. If a new system dependency (like a C library) is required by a new Rust crate, add it to `manifest.scm`.
3. Always verify changes by running tests through `guix time-machine` before declaring them successful.
