#!/usr/bin/env python3
"""Move stage-view/DOM witness paths, then rewrite only their named references.

Run --mode moves in a clean checkout and commit those moves separately.
Run --mode refs after that commit; --mode verify checks the reference replay.
Quoted runtime payloads stay intact except explicit module/test path literals.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess

TYPES = {
    'SpolveroFeed': 'StageFeed',
    'SpolveroFeedSchemaMismatch': 'StageFeedSchemaMismatch',
    'SpolveroPage': 'StagePage',
    'SpolveroRemark': 'StageRemark',
    'SpolveroFeedRemark': 'StageFeedRemark',
    'SpolveroNegotiation': 'StageNegotiation',
}
PREFIXES = {
    'playground/src/features/davinci/': 'playground/src/features/stages/',
    'crates/vize_curator/src/inspector/spolvero/': 'crates/vize_curator/src/inspector/stages/',
    'crates/vize_atelier_dom/tests/davinci_l2_': 'crates/vize_atelier_dom/tests/l2_',
}
FILES = {
    'crates/vize_curator/src/inspector/spolvero.rs': 'crates/vize_curator/src/inspector/stages.rs',
    'crates/vize_vitrine/src/wasm/analyze/spolvero.rs': 'crates/vize_vitrine/src/wasm/analyze/stages.rs',
    'playground/src/wasm/types/spolvero.ts': 'playground/src/wasm/types/stages.ts',
    'playground/src/wasm/types/spolvero.test.ts': 'playground/src/wasm/types/stages.test.ts',
}
ALLOWED_SUFFIXES = {'.rs', '.ts', '.vue', '.md', '.tsv', '.json', '.yml', '.pkl', '.toml'}
PROTECTED_SEGMENTS = {'snapshots', '__snapshots__', '_fixtures', 'fixtures', 'versions'}


def git(repo, *args):
    return subprocess.check_output(['git', '-C', str(repo), *args], text=True)


def destination(name):
    if name in FILES:
        return FILES[name]
    for old, new in PREFIXES.items():
        if name.startswith(old):
            return new + name[len(old):]
    return name


def paths(repo):
    return git(repo, 'ls-files').splitlines()


def path_refs(text):
    for old, new in PREFIXES.items():
        text = text.replace(old, new)
    for old, new in FILES.items():
        text = text.replace(old, new)
    text = text.replace('features/davinci/', 'features/stages/')
    text = text.replace('wasm/types/spolvero', 'wasm/types/stages')
    return text


def quoted_ref(name, text):
    text = path_refs(text)
    if name.startswith('playground/src/wasm/types/'):
        text = re.sub(r'([\"\'])\./spolvero\1', r'\1./stages\1', text)
    if name == 'tests/tooling/support/davinci-stage-dependencies.ts':
        text = text.replace('^davinci_l2_', '^l2_')
    if name == 'tests/tooling/davinci-dom-production-boundary.test.ts':
        text = text.replace('"davinci_l2_profile"', '"l2_profile"')
    return text


def tokens(source, rust):
    """Yield code/comment/literal spans; Rust lifetimes are ordinary code."""
    cursor = 0
    while cursor < len(source):
        start = cursor
        if source.startswith('//', cursor):
            cursor = source.find('\n', cursor)
            if cursor == -1:
                cursor = len(source)
            yield 'comment', start, cursor
            continue
        if source.startswith('/*', cursor):
            cursor += 2
            depth = 1
            while depth and cursor < len(source):
                if source.startswith('/*', cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith('*/', cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            if depth:
                raise ValueError('unterminated block comment')
            yield 'comment', start, cursor
            continue
        raw = re.match(r'(?:b|c)?r(#+)?"', source[cursor:]) if rust else None
        if raw:
            end = '"' + (raw[1] or '')
            cursor = source.find(end, cursor + raw.end())
            if cursor == -1:
                raise ValueError('unterminated raw string')
            cursor += len(end)
            yield 'literal', start, cursor
            continue
        quote = source[cursor]
        char = re.match(r"'(?:\\.|[^'\\\n])'", source[cursor:]) if rust else None
        if quote == '"' or (not rust and quote in "'`") or char:
            if char:
                cursor += char.end()
            else:
                cursor += 1
                while cursor < len(source):
                    if source[cursor] == '\\':
                        cursor += 2
                    elif source[cursor] == quote:
                        cursor += 1
                        break
                    else:
                        cursor += 1
                else:
                    raise ValueError('unterminated string')
            yield 'literal', start, cursor
            continue
        ident = re.match(r'[A-Za-z_][A-Za-z_0-9]*', source[cursor:])
        cursor += ident.end() if ident else 1
        yield 'ident' if ident else 'code', start, cursor


def rewrite(name, source):
    needles = [*TYPES, *PREFIXES, *FILES, 'features/davinci/', 'wasm/types/spolvero', 'spolvero::']
    if name in {'crates/vize_curator/src/inspector.rs', 'crates/vize_vitrine/src/wasm/analyze.rs'}:
        needles.append('spolvero')
    if name == 'crates/vize_atelier_dom/Cargo.toml':
        needles.append('davinci_l2_filters')
    if name.startswith('playground/src/wasm/types/'):
        needles.append('./spolvero')
    if name in {'crates/vize_atelier_dom/tests/l2_slots.rs',
                'tests/tooling/support/davinci-stage-dependencies.ts',
                'tests/tooling/davinci-dom-production-boundary.test.ts'}:
        needles.append('davinci_l2_')
    if not any(needle in source for needle in needles):
        return source
    if name.endswith('.vue'):
        source = path_refs(source)
        return re.sub(r'(<script\b[^>]*>)(.*?)(</script>)',
                      lambda m: m[1] + rewrite(name + '.ts', m[2]) + m[3], source, flags=re.S)
    if Path(name).suffix not in {'.rs', '.ts', '.vue'}:
        source = path_refs(source)
        if name == 'crates/vize_atelier_dom/Cargo.toml':
            source = source.replace('name = "davinci_l2_filters"', 'name = "l2_filters"')
        return source
    result = []
    previous = ''
    for kind, start, end in tokens(source, name.endswith('.rs')):
        text = source[start:end]
        if kind == 'literal':
            text = quoted_ref(name, text)
        elif kind == 'comment':
            text = path_refs(text)
            text = re.sub(r'\b(?:' + '|'.join(TYPES) + r')\b', lambda m: TYPES[m[0]], text)
        elif kind == 'ident':
            text = TYPES.get(text, text)
            if text == 'spolvero' and source[end:].lstrip().startswith('::'):
                text = 'stages'
            if name in {'crates/vize_curator/src/inspector.rs', 'crates/vize_vitrine/src/wasm/analyze.rs'}:
                if text == 'spolvero' and previous == 'mod':
                    text = 'stages'
            if name == 'crates/vize_atelier_dom/tests/l2_slots.rs' and text == 'davinci_l2_slots':
                text = 'l2_slots'
            previous = text
        result.append(text)
    after = ''.join(result)
    if name == 'tests/tooling/support/davinci-stage-dependencies.ts':
        after = after.replace('^davinci_l2_', '^l2_')
    return after


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', required=True, type=Path)
    parser.add_argument('--mode', choices=['moves', 'refs', 'verify'], required=True)
    args = parser.parse_args()
    repo = args.repo.resolve()
    names = paths(repo)
    if args.mode == 'moves':
        if git(repo, 'status', '--porcelain', '--untracked-files=no'):
            raise SystemExit('move-only mode requires a clean tracked worktree')
        pairs = [(name, destination(name)) for name in names if destination(name) != name]
        if len({new for _, new in pairs}) != len(pairs):
            raise SystemExit('duplicate destination')
        for old, new in pairs:
            if (repo / new).exists():
                raise SystemExit('destination collision: ' + new)
        for old, new in pairs:
            (repo / new).parent.mkdir(parents=True, exist_ok=True)
            git(repo, 'mv', '--', old, new)
        print(json.dumps({'moveOnlyPaths': len(pairs), 'moves': pairs}, indent=2))
        return
    changes = []
    for name in names:
        path = repo / name
        if (path.suffix not in ALLOWED_SUFFIXES or PROTECTED_SEGMENTS.intersection(path.parts)
                or not path.is_file() or name.startswith('tools/benchmarks/results/')
                or name.startswith('docs/davinci/decisions/')
                or name == 'tools/support/levels/rename-stage-paths.py'):
            continue
        source = path.read_text()
        try:
            after = rewrite(name, source)
        except ValueError as error:
            raise ValueError(name + ': ' + str(error)) from error
        if after != source:
            changes.append(name)
            if args.mode == 'refs':
                path.write_text(after)
    print(json.dumps({'referenceFiles': len(changes), 'paths': changes}, indent=2))
    if args.mode == 'verify' and changes:
        raise SystemExit('unapplied stage references remain')


if __name__ == '__main__':
    main()
