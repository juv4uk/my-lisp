#!/bin/bash
set -euo pipefail

state_root=${XDG_STATE_HOME:-$HOME/.local/state}/my-lisp
pid_file=$state_root/oracle-local.pid

if [ ! -f "$pid_file" ]; then
  echo "local Oracle is not recorded as running"
  exit 0
fi

pid=$(sed -n '1p' "$pid_file")
case "$pid" in
  ''|*[!0-9]*) echo "ERROR: invalid pid file: $pid_file" >&2; exit 1 ;;
esac

if kill -0 "$pid" 2>/dev/null; then
  kill "$pid"
  echo "stopped local Oracle pid=$pid"
else
  echo "local Oracle pid=$pid is already absent"
fi
rm -f "$pid_file"
