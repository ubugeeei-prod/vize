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
# Only these reviewed placeholders may be removed; changed source is preserved.
STUBS = {
    'davinci/vize_l0/src/diag.rs': '63fb56ed668aa6576805b0341f5fd3094a18b083',
    'davinci/vize_l0/src/diag/diagnostic.rs': 'b941ad7a50a55935017ee3ce27bf7e5a3e4b6478',
    'davinci/vize_l0/src/diag/witness.rs': 'e76c33585afc63e47ff3ed0d7cd924b0911e2d57',
    'davinci/vize_l0/src/dump.rs': '191c6aab0ac9951a4c6d0244c8f43a1f83c1a20c',
    'davinci/vize_l0/src/dump/runtime.rs': '3e08cdccbab5f2ae1fd70d37648285fd8e3ba177',
    'davinci/vize_l0/src/dump/value.rs': '6a370c2559121ab61f72e52bec0d7596841f168e',
    'davinci/vize_l0/src/fact.rs': '665a70593d4ea0df9b62af8131c3da16e0fbf890',
    'davinci/vize_l0/src/key.rs': '764958aa65c58754a43194d4e7877898cc0c62f2',
    'davinci/vize_l0/src/pass.rs': 'b1a2b18d845f79b59036a4a7687173d653fe99e6',
    'davinci/vize_l0/src/pass/observer.rs': '0da8b5e720af851bafcb51401759c4d0212477b8',
    'davinci/vize_l0/src/pass/observer/fusion.rs': '576f27c431f842465f6fa6b49295e49857d4740c',
    'davinci/vize_l0/src/pass/observer/remarks.rs': 'd917180cfbc4aa54ea65041ddf74bbc3a2e185f4',
    'davinci/vize_l0/src/pass/observer/timing.rs': '6f10484e1af6221ed811cdfb973bb5d7560a75b2',
}
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('phase', choices=('prepare', 'moves', 'check'))
    phase = parser.parse_args().phase
    import json
    if phase == 'prepare':
        if not (ROOT / OLD / 'fact.rs').exists():
            raise SystemExit('L0 runtime sources already moved; use check instead')
        collisions = {target for target in MOVES.values() if (ROOT / target).exists()}
        if collisions - STUBS.keys():
            raise SystemExit('Refusing unexpected L0 target collision: ' + ', '.join(sorted(collisions - STUBS.keys())))
        present = [name for name in STUBS if (ROOT / name).exists()]
        for name in present:
            blob = subprocess.check_output(['git', 'hash-object', name], cwd=ROOT, text=True).strip()
            if blob != STUBS[name]:
                raise SystemExit('Refusing to remove changed L0 source: ' + name)
        subprocess.run(['git', 'diff', '--quiet', 'HEAD', '--', *present], cwd=ROOT, check=True)
        MAP.write_text(json.dumps(MOVES, indent=2) + '\n')
        if present:
            subprocess.run(['git', 'rm', '--', *present], cwd=ROOT, check=True)
    else:
        moves = json.loads(MAP.read_text())
        moves['davinci/vize_davinci_derive'] = 'davinci/vize_l0_derive'
        # Validate every source/target before the first index mutation.
        for old, new in moves.items():
            source, target = ROOT / old, ROOT / new
            if source.exists() and target.exists():
                raise SystemExit('Refusing source/target collision: ' + old + ' -> ' + new)
            if not source.exists() and not target.exists():
                raise SystemExit('Missing runtime source and target: ' + old)
            if phase == 'check' and source.exists():
                raise SystemExit('Unmoved runtime source: ' + old)
        if phase == 'moves':
            for old, new in moves.items():
                source, target = ROOT / old, ROOT / new
                if not source.exists():
                    continue
                target.parent.mkdir(parents=True, exist_ok=True)
                subprocess.run(['git', 'mv', old, new], cwd=ROOT, check=True)


if __name__ == '__main__':
    main()
