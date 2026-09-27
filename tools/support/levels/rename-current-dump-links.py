#!/usr/bin/env python3
"""Follow current Dump law links without rewriting historical record evidence."""
import argparse
from pathlib import Path
import re
import subprocess

EDITS = {
    'docs/davinci/plan/phase-2-records.md': (
        ('](../../../crates/vize_davinci/tests/repro_folio.rs)',
         '](../../../crates/vize_davinci/tests/repro_dump.rs)'),
    ),
    'docs/davinci/plan/folio-format.md': (
        ('`crates/vize_davinci/tests/repro_folio.rs`',
         '`crates/vize_davinci/tests/repro_dump.rs`'),
    ),
}


def rewrite(name, source):
    for old, new in EDITS.get(name, ()):
        assert source.count(old) + source.count(new) == 1, name
        source = source.replace(old, new)
    return source


def canonical(source):
    # Markdown table padding and separator widths are formatter layout only.
    def cell(value):
        value = value.strip()
        return re.sub('-+', '-', value) if re.fullmatch(':?-+:?', value) else value
    return [tuple(cell(value) for value in line.split('|')) if line.startswith('|')
            else line for line in source.splitlines()]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify', action='store_true')
    parser.add_argument('--base-ref', default='fd5ed0b703544a52df03497a770db6d0bd97a27e')
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[3]
    for name in EDITS:
        source = subprocess.check_output(['git', '-C', str(repo), 'show',
                                          args.base_ref + ':' + name], text=True)
        expected = rewrite(name, source)
        path = repo / name
        if args.verify:
            assert canonical(path.read_text()) == canonical(expected), name
        else:
            assert canonical(path.read_text()) in (canonical(source), canonical(expected)), name
            path.write_text(expected)
    print('two current references follow repro_dump; historical evidence unchanged')


if __name__ == '__main__':
    main()
