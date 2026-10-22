#!/usr/bin/env python3
"""Measures a launcher prototype on Linux: start time, memory (PSS) and CPU,
at rest in the menu and while a game runs. The launcher and all its child
processes are counted; the emulator is not.

Mesure un prototype de lanceur sous Linux : démarrage, mémoire (PSS) et
processeur, au repos et pendant un jeu. Le lanceur et ses processus enfants
sont comptés, pas l'émulateur.

Usage: measure-linux.py NAME WINDOW_TITLE -- command [args...]
Needs: an X display (DISPLAY), xdotool. The launcher must be configured so
that "down, Enter" from the menu starts a game (any emulator, real or fake)
lasting longer than the measurement.
"""
import json
import os
import subprocess
import sys
import time

SECONDS = int(os.environ.get("MEASURE_SECONDS", "30"))
IGNORE = {"retroarch", "dolphin-emu", "pcsx2-qt", "sleep"}
TICK = os.sysconf("SC_CLK_TCK")


def children_map():
    m = {}
    for d in os.listdir("/proc"):
        if not d.isdigit():
            continue
        try:
            with open(f"/proc/{d}/stat") as f:
                parts = f.read().rsplit(")", 1)[1].split()
            m.setdefault(int(parts[1]), []).append(int(d))
        except OSError:
            pass
    return m


def comm(pid):
    try:
        with open(f"/proc/{pid}/comm") as f:
            return f.read().strip()
    except OSError:
        return ""


def tree(root):
    cm = children_map()
    out, todo = [], [root]
    while todo:
        p = todo.pop()
        if comm(p) in IGNORE:
            continue
        out.append(p)
        todo.extend(cm.get(p, []))
    return out


def pss_mb(pid):
    try:
        with open(f"/proc/{pid}/smaps_rollup") as f:
            for line in f:
                if line.startswith("Pss:"):
                    return int(line.split()[1]) / 1024
    except OSError:
        pass
    return 0.0


def cpu_ticks(pid):
    try:
        with open(f"/proc/{pid}/stat") as f:
            parts = f.read().rsplit(")", 1)[1].split()
        return int(parts[11]) + int(parts[12])
    except OSError:
        return 0


def measure(root, phase):
    seen = {}
    mem = []
    t0 = time.monotonic()
    for p in tree(root):
        seen[p] = cpu_ticks(p)
    start = dict(seen)
    while time.monotonic() - t0 < SECONDS:
        time.sleep(2)
        pids = tree(root)
        for p in pids:
            if p not in start:
                start[p] = 0  # new process: count all its CPU
            seen[p] = cpu_ticks(p)
        mem.append(sum(pss_mb(p) for p in pids))
    elapsed = time.monotonic() - t0
    used = sum(seen[p] - start[p] for p in seen) / TICK
    return {
        "phase": phase,
        "processes": len(tree(root)),
        "memory_pss_mb": round(sum(mem) / len(mem), 1),
        "memory_max_mb": round(max(mem), 1),
        "cpu_percent_one_core": round(100 * used / elapsed, 2),
    }


def main():
    name, title = sys.argv[1], sys.argv[2]
    cmd = sys.argv[sys.argv.index("--") + 1:]
    t0 = time.monotonic()
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    subprocess.run(["xdotool", "search", "--sync", "--onlyvisible", "--name", title],
                   stdout=subprocess.DEVNULL, timeout=60)
    startup_ms = round((time.monotonic() - t0) * 1000)
    time.sleep(5)
    results = [measure(proc.pid, "menu")]
    subprocess.run(["xdotool", "key", "Down"]); time.sleep(1)
    subprocess.run(["xdotool", "key", "Return"]); time.sleep(3)
    results.append(measure(proc.pid, "in_game"))
    proc.terminate()
    try:
        proc.wait(5)
    except subprocess.TimeoutExpired:
        proc.kill()
    for r in results:
        r.update(prototype=name, startup_ms=startup_ms)
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
