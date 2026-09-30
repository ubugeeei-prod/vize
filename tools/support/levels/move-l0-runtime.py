#!/usr/bin/env python3
"""Replay #6833 neutral substrate ownership moves on directory-separated main."""
from pathlib import Path
import argparse
import subprocess

ROOT = Path(__file__).resolve().parents[3]
OLD = Path('davinci/vize_davinci/src')
NEW = Path('davinci/vize_l0/src')
MOVES = {}
for module in ('fact', 'key', 'pass'):
    MOVES[str(OLD / (module + '.rs'))] = str(NEW / (module + '.rs'))
    for p in sorted((ROOT / OLD / module).rglob('*')):
        if p.is_file():
            MOVES[str(p.relative_to(ROOT))] = str(NEW / module / p.relative_to(ROOT / OLD / module))
MOVES[str(OLD / 'diagnostic.rs')] = str(NEW / 'diag.rs')
for p in sorted((ROOT / OLD / 'diagnostic').rglob('*')):
    if p.is_file():
        MOVES[str(p.relative_to(ROOT))] = str(NEW / 'diag' / p.relative_to(ROOT / OLD / 'diagnostic'))
MOVES[str(OLD / 'witness.rs')] = str(NEW / 'diag/verify.rs')
for p in sorted((ROOT / OLD / 'witness').rglob('*')):
    if p.is_file():
        MOVES[str(p.relative_to(ROOT))] = str(NEW / 'diag/verify' / p.relative_to(ROOT / OLD / 'witness'))
MOVES[str(OLD / 'dump.rs')] = str(NEW / 'dump.rs')
for module in ('collector', 'page', 'plan', 'remarks', 'value'):
    MOVES[str(OLD / 'dump' / (module + '.rs'))] = str(NEW / 'dump' / (module + '.rs'))
    for p in sorted((ROOT / OLD / 'dump' / module).rglob('*')):
        if p.is_file():
            MOVES[str(p.relative_to(ROOT))] = str(NEW / 'dump' / module / p.relative_to(ROOT / OLD / 'dump' / module))
# Persist the map so integration is replayable after source moves.
MAP = ROOT / 'tools/support/levels/l0-runtime-moves.json'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('phase', choices=('prepare', 'moves', 'check'))
    phase = parser.parse_args().phase
    import json
    if phase == 'prepare':
        if not (ROOT / OLD / 'fact.rs').exists():
            raise SystemExit('L0 runtime sources already moved; use check instead')
        MAP.write_text(json.dumps(MOVES, indent=2) + '\n')
        stubs = [NEW / 'diag/diagnostic.rs', NEW / 'diag/witness.rs', NEW / 'dump/runtime.rs',
                 NEW / 'pass/observer/fusion.rs', NEW / 'pass/observer/remarks.rs', NEW / 'pass/observer/timing.rs']
        stubs += [Path(target) for target in MOVES.values() if (ROOT / target).exists()]
        for p in dict.fromkeys(stubs):
            if (ROOT / p).exists():
                subprocess.run(['git', 'rm', str(p)], cwd=ROOT, check=True)
    else:
        moves = json.loads(MAP.read_text())
        for old, new in moves.items():
            source, target = ROOT / old, ROOT / new
            if phase == 'moves' and source.exists() and not target.exists():
                target.parent.mkdir(parents=True, exist_ok=True)
                subprocess.run(['git', 'mv', old, new], cwd=ROOT, check=True)
            elif not target.exists():
                raise SystemExit('Missing moved source: ' + new)
        old_derive = ROOT / 'davinci/vize_davinci_derive'
        new_derive = ROOT / 'davinci/vize_l0_derive'
        if phase == 'moves' and old_derive.exists():
            subprocess.run(['git', 'mv', str(old_derive), str(new_derive)], cwd=ROOT, check=True)
        elif not new_derive.exists():
            raise SystemExit('Missing L0 derive crate')


if __name__ == '__main__':
    main()
