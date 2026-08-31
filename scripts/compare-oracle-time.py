#!/usr/bin/env python3
"""Compare UTC observations from local and remote WSM Oracles.

This measures clock offset, not physical distance. Both requests are started
concurrently; each sample records the client's monotonic midpoint and RTT.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import re
import socket
import subprocess
import time

UTC_RE = re.compile(r"\(value \(utc (\d+) (\d+) (\d+) (\d+) (\d+) (\d+) (\d+)\)\)")


def utc_ns(response: str) -> int:
    match = UTC_RE.search(response)
    if not match:
        raise ValueError(f"utc-now value not found in response: {response[:240]!r}")
    year, month, day, hour, minute, second, nano = map(int, match.groups())
    # Gregorian conversion without datetime range/float limitations.
    if not 1 <= month <= 12 or not 0 <= nano < 1_000_000_000:
        raise ValueError(f"invalid UTC value: {match.group(0)}")
    import calendar

    return calendar.timegm((year, month, day, hour, minute, second)) * 1_000_000_000 + nano


def sample(name: str, host: str, port: int, timeout: float) -> dict[str, int | str]:
    # `eval` is the stable wire operation supported by the deployed Oracle;
    # `oracle-eval` is a newer agent-facing wrapper and is not universal.
    request = b"(request (id 1) (op eval) (source \"(utc-now)\"))\n"
    started = time.monotonic_ns()
    wall_before = time.time_ns()
    with socket.create_connection((host, port), timeout=timeout) as sock:
        sock.settimeout(timeout)
        sock.sendall(request)
        chunks: list[bytes] = []
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            chunks.append(chunk)
            if b"\n" in chunk:
                break
    wall_after = time.time_ns()
    finished = time.monotonic_ns()
    response = b"".join(chunks).decode("utf-8", errors="strict")
    return {
        "name": name,
        "oracle_utc_ns": utc_ns(response),
        "client_midpoint_utc_ns": (wall_before + wall_after) // 2,
        "rtt_ns": finished - started,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--local", default="127.0.0.1:9999")
    parser.add_argument("--remote", default="100.113.68.50:9999")
    parser.add_argument("--samples", type=int, default=5)
    parser.add_argument("--timeout", type=float, default=3.0)
    parser.add_argument("--sync-system-clock", action="store_true",
                        help="enable NTP via timedatectl and wait for synchronization")
    parser.add_argument("--sync-wait-seconds", type=int, default=10)
    args = parser.parse_args()

    def endpoint(value: str) -> tuple[str, int]:
        host, separator, port = value.rpartition(":")
        if not separator or not host or not port.isdigit():
            raise SystemExit(f"invalid endpoint: {value!r}")
        return host, int(port)

    local = endpoint(args.local)
    remote = endpoint(args.remote)
    if args.samples < 1:
        raise SystemExit("--samples must be positive")
    if args.sync_wait_seconds < 1:
        raise SystemExit("--sync-wait-seconds must be positive")

    if args.sync_system_clock:
        try:
            subprocess.run(["timedatectl", "set-ntp", "true"], check=True,
                           timeout=args.timeout)
        except (OSError, subprocess.SubprocessError) as exc:
            raise SystemExit(f"system clock synchronization failed: {exc}") from exc
        deadline = time.monotonic() + args.sync_wait_seconds
        synchronized = False
        while time.monotonic() < deadline:
            try:
                state = subprocess.run(
                    ["timedatectl", "show", "-p", "NTPSynchronized", "--value"],
                    check=True, capture_output=True, text=True, timeout=args.timeout,
                ).stdout.strip().lower()
            except (OSError, subprocess.SubprocessError) as exc:
                raise SystemExit(f"cannot verify NTP synchronization: {exc}") from exc
            if state == "yes":
                synchronized = True
                break
            time.sleep(1)
        if not synchronized:
            raise SystemExit("system clock is not NTP-synchronized after waiting")
        print(f"system-clock=ntp-synchronized wait-seconds={args.sync_wait_seconds}")

    rows: list[dict[str, int | str]] = []
    for _ in range(args.samples):
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            futures = [
                pool.submit(sample, "local", *local, args.timeout),
                pool.submit(sample, "remote", *remote, args.timeout),
            ]
            rows.extend(f.result() for f in futures)

    pairs = zip(rows[0::2], rows[1::2])
    offsets = []
    print("oracle-time-observation/1")
    for index, (first, second) in enumerate(pairs, 1):
        by_name = {str(first["name"]): first, str(second["name"]): second}
        local_row, remote_row = by_name["local"], by_name["remote"]
        local_offset = int(local_row["oracle_utc_ns"]) - int(local_row["client_midpoint_utc_ns"])
        remote_offset = int(remote_row["oracle_utc_ns"]) - int(remote_row["client_midpoint_utc_ns"])
        offset = int(remote_row["oracle_utc_ns"]) - int(local_row["oracle_utc_ns"])
        uncertainty = (int(local_row["rtt_ns"]) + int(remote_row["rtt_ns"])) // 2
        offsets.append(offset)
        print(f"sample={index} offset-ns={offset} uncertainty-ns={uncertainty} "
              f"local-rtt-ns={local_row['rtt_ns']} remote-rtt-ns={remote_row['rtt_ns']} "
              f"local-clock-offset-ns={local_offset} remote-clock-offset-ns={remote_offset}")
    offsets.sort()
    median = offsets[len(offsets) // 2]
    spread = offsets[-1] - offsets[0]
    print(f"baseline-offset-ns={median} sample-spread-ns={spread} samples={len(offsets)} status=observed")
    print("meaning=clock-offset-not-physical-distance")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
