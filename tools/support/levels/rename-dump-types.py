#!/usr/bin/env python3
"""Replay the bounded Dump API migration from its immutable parent.

Only Rust identifiers, named module references and proc-macro API literals
change. Runtime/fixture literals stay frozen. Moves are committed separately.
--verify compares token/literal streams after ordinary Rust formatting.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import re
import subprocess


def load(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + '.py'))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


names = load('dump-names')
lexer = load('rename-stage-paths')
PROTECTED = {'snapshots', '__snapshots__', '_fixtures', 'fixtures', 'versions'}


def git(repo, *args):
    return subprocess.check_output(['git', '-C', str(repo), *args], text=True)


def absolute(path, owner):
    parts = path.split('::')
    if not owner:
        return path
    parent = owner.split('::')
    if parts[0] in {'crate', '$crate'}:
        return '::'.join([parent[0], *parts[1:]])
    if parts[0] == 'self':
        return '::'.join([*parent, *parts[1:]])
    while parts and parts[0] == 'super':
        parent.pop()
        parts.pop(0)
    if parts and (parts[0].startswith('vize_') or parts[0] in {'core', 'std', 'alloc', 'syn', 'quote', 'proc_macro', 'insta', 'serde', 'serde_json'}):
        return path
    return '::'.join([*parent, *parts])


def relative(path, owner):
    if owner and path.split('::')[0] == owner.split('::')[0]:
        return 'crate::' + path.split('::', 1)[1]
    return path


def qualified_target(path, owner, targets):
    parts = path.split('::')
    if parts[0] not in {'crate', '$crate', 'super', 'self', 'folio', 'values_folio'} and not parts[0].startswith('vize_'):
        return path
    for count in range(len(parts), 0, -1):
        resolved = absolute('::'.join(parts[:count]), owner)
        if resolved in targets:
            return relative(targets[resolved], owner) + ''.join('::' + part for part in parts[count:])
    resolved = absolute(path, owner)
    for prefix in sorted(names.CONCERNS, key=len, reverse=True):
        if resolved == prefix or resolved.startswith(prefix + '::'):
            return relative(names.namespace(prefix) + resolved[len(prefix):], owner)
    return path

def use_leaves(text):
    tokens = re.findall(r'\w+|::|[{},*]', text)
    position = 0
    leaves = []

    def walk(prefix):
        nonlocal position
        if tokens[position] == '{':
            position += 1
            while tokens[position] != '}':
                walk(prefix)
                if tokens[position] == ',':
                    position += 1
            position += 1
            return
        head = tokens[position]
        position += 1
        if position < len(tokens) and tokens[position] == '::':
            position += 1
            walk([*prefix, head])
            return
        alias = None
        if position < len(tokens) and tokens[position] == 'as':
            position += 1
            alias = tokens[position]
            position += 1
        leaves.append(('::'.join([*prefix, head]), alias))

    walk([])
    if position != len(tokens):
        raise ValueError('unconsumed use tree: ' + text)
    return leaves


def owner_at(path, masked, position):
    owner = names.module(path)
    if not owner:
        return None
    for match in re.finditer(r'\bmod\s+(\w+)\s*\{', masked):
        if match.end() > position:
            break
        depth, cursor = 1, match.end()
        while depth and cursor < position:
            depth += (masked[cursor] == '{') - (masked[cursor] == '}')
            cursor += 1
        if depth:
            owner += '::' + match[1]
    return owner

def use_statement(path, statement, targets, owner=None):
    owner = owner or names.module(path)
    visibility, body = statement.split('use ', 1)
    groups = {}
    aliases = {}
    for old, alias in use_leaves(body[:-1]):
        resolved = absolute(old, owner)
        target = targets.get(resolved, names.namespace(resolved))
        if resolved not in targets:
            parts = target.split('::')
            parts[-1] = names.local(path, parts[-1])
            target = '::'.join(parts)
        identifier = old.rsplit('::', 1)[-1]
        wanted = names.local(path, alias or identifier)
        target = relative(target, owner)
        parent, leaf = target.rsplit('::', 1) if '::' in target else ('', target)
        wanted = wanted if wanted != leaf else None
        key = wanted or leaf
        if key in aliases and aliases[key] != target:
            raise ValueError(f'{path}: colliding alias {key}: {aliases[key]} vs {target}')
        aliases[key] = target
        item = leaf + (' as ' + wanted if wanted else '')
        if item not in groups.setdefault(parent, []):
            groups[parent].append(item)
    result = []
    for parent, items in groups.items():
        suffix = items[0] if len(items) == 1 else '{' + ', '.join(items) + '}'
        result.append(visibility + 'use ' + (parent + '::' if parent else '') + suffix + ';')
    return '\n'.join(result)


def canonical(source):
    parts = list(lexer.tokens(source, True))
    mask = ''.join(' ' * (b-a) if kind in {'comment', 'literal'} else source[a:b] for kind,a,b in parts)
    imports = []
    for match in reversed(list(re.finditer(r'(?:pub(?:\([^)]*\))?\s+)?use\s+[^;]+;', mask))):
        statement = source[match.start():match.end()]
        visibility, body = statement.split('use ', 1)
        imports.extend(visibility.strip() + ':' + target + ':' + (alias or '')
                       for target, alias in use_leaves(body[:-1]))
        source = source[:match.start()] + source[match.end():]
    result = ''.join(source[a:b] for kind, a, b in lexer.tokens(source, True)
                     if kind != 'comment' and not source[a:b].isspace())
    return re.sub(r',(?=[})\]])', '', result) + '\n' + '\n'.join(sorted(imports))


def rewrite(path, source, targets):
    if not re.search(r'Folio|\bfolio\b|values_folio|assert_folio_snapshot|DumpPage', source):
        return source
    # Keep IR and shadow document names distinct inside the defining namespace.
    if path == 'crates/vize_l2/src/folio/owned.rs':
        source = re.sub(r'\b(Op|Attribute)\b', r'Ir\1', source)
        source = source.replace('crate::op::{IrAttribute,', 'crate::op::{Attribute as IrAttribute,').replace(', IrOp, Region}', ', Op as IrOp, Region}')
    # Eliminate legacy type aliases, leaving the canonical Page definition.
    source = re.sub(r'/// Compatibility alias[^\n]*\n(?:#\[[^\n]*\]\n)?pub type (?:DisegnoFolio|ImpetoFolio|ReactivityFolio) = \w+;\n', '', source)
    saved = []
    pattern = r'(?:pub(?:\([^)]*\))?\s+)?use\s+[^;]+;'
    def use(match, owner):
        statement = match[0]
        if not re.search(r'Folio|\bfolio\b|values_folio|DumpPage', statement):
            return statement
        saved.append(use_statement(path, statement, targets, owner))
        return f'__DUMP_USE_{len(saved) - 1}__'
    # Mask comments/literals first so quoted examples cannot become imports.
    parts = list(lexer.tokens(source, True))
    mask = ''.join(' ' * (b-a) if kind in {'comment', 'literal'} else source[a:b] for kind,a,b in parts)
    matches = list(re.finditer(pattern, mask))
    for match in reversed(matches):
        source = source[:match.start()] + use(type('Match', (), {'__getitem__': lambda self, key: source[match.start():match.end()]})(), owner_at(path, mask, match.start())) + source[match.end():]
    # Preserve already-resolved qualified paths while rewriting local identifiers.
    owner = names.module(path)
    chain = r'(?:\$crate|[A-Za-z_]\w*)(?:::[A-Za-z_]\w*)+'
    qualified = []
    parts = list(lexer.tokens(source, True))
    mask = ''.join(' ' * (b-a) if kind in {'comment', 'literal'} else source[a:b] for kind,a,b in parts)
    for match in reversed(list(re.finditer(chain, mask))):
        old = match[0]
        updated = qualified_target(old, owner, targets)
        if old.startswith('$crate::'):
            updated = updated.replace('crate::', '$crate::', 1)
        if updated != old:
            qualified.append(updated)
            source = source[:match.start()] + f'__DUMP_PATH_{len(qualified)-1}__' + source[match.end():]
    result = []
    for kind, start, end in lexer.tokens(source, True):
        text = source[start:end]
        if kind == 'literal':
            if path.endswith('vize_davinci_derive/src/model.rs'):
                if text == '"folio"' or 'derive(Folio)' in text or any(word in text for word in ('folio attribute', 'folio name')):
                    text = text.replace('Folio', 'Dump').replace('folio', 'dump')
        elif kind == 'ident':
            text = names.local(path, text)
            if text == 'folio' and (source[:start].rstrip().endswith(('mod', 'attributes(')) or source[max(0,start-2):start] == '#['):
                text = 'dump'
            if text == 'values_folio':
                text = 'values_dump'
        elif kind == 'comment':
            text = text.replace('#[folio(', '#[dump(').replace('[`folio`]', '[`dump`]')
            text = re.sub(chain, lambda m: qualified_target(m[0], owner, targets), text)
            text = re.sub(r'\b[A-Za-z_]\w*\b', lambda m: names.local(path, m[0]), text)
        result.append(text)
    source = ''.join(result)
    for index, target in enumerate(qualified):
        source = source.replace(f'__DUMP_PATH_{index}__', target)
    for index, statement in enumerate(saved):
        source = source.replace(f'__DUMP_USE_{index}__', statement)
    if path == 'crates/vize_davinci/src/folio.rs':
        source = source.replace('pub mod dump;', 'pub mod collector;')
    if path == 'crates/vize_davinci/src/folio/plan.rs':
        source = source.replace('pub struct Page {', '#[dump(name = "fusion-plan-folio")]\npub struct Page {')
    # Concern namespaces are public; old flat exports retain neutral names only.
    if path in {'crates/vize_l2/src/folio.rs', 'crates/vize_l3/src/extract.rs', 'crates/vize_l3/src/placement.rs', 'crates/vize_l3/src/lattice.rs', 'crates/vize_l2_to_l3/src/partition.rs'}:
        source = re.sub(r'(?m)^(?:pub(?:\([^)]*\))? )?mod dump;', 'pub mod dump;', source)
        source = re.sub(r'(?m)^(?:pub(?:\([^)]*\))? )?mod provenance;', 'pub mod provenance;', source)
    if path == 'crates/vize_l2_to_l3/src/lib.rs':
        source = source.replace('mod partition;', 'pub mod partition;')
    if path == 'crates/vize_l2/tests/stage_name_alias.rs':
        start = source.index('#[test]\nfn disegno_folio_remains_a_compatibility_alias()')
        source = source[:start] + """#[test]
fn canonical_dump_namespace_keeps_the_full_wire_contract() {
    let page: vize_l2::dump::Page = vize_l2::dump::Page::parse(EMPTY)
        .expect("canonical namespace parses");
    let full = page.print_to_string(DumpMode::Full);
    let replay = vize_l2::dump::Page::parse(&full).expect("full dump parses");

    assert_eq!(full.as_str(), EMPTY);
    assert_eq!(replay, page);
}
"""
    return source


def migrate_storage(source):
    result = []
    for line in source.splitlines(keepends=True):
        fields = line.split('\t')
        if len(fields) == 11:
            fields[2] = names.destination(fields[2])
        result.append('\t'.join(fields))
    return ''.join(result)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path.cwd())
    parser.add_argument('--base', default=names.BASE,
                        help='clean pre-move source revision used for this replay')
    parser.add_argument('--verify', action='store_true')
    parser.add_argument('--moves', action='store_true')
    args = parser.parse_args()
    source_paths = git(args.repo, 'ls-tree', '-r', '--name-only', args.base).splitlines()
    if args.moves:
        if args.verify or git(args.repo, 'status', '--porcelain', '--untracked-files=no').strip():
            raise ValueError('move mode requires a clean tracked checkout')
        moves = {path: names.destination(path) for path in source_paths
                 if path.startswith('crates/') and path.endswith('.rs')
                 and re.search(r'(^|/)(folio|[^/]+_folio)(/|\.)', path)}
        if len(moves) != 45:
            raise ValueError('expected exactly 45 authored module/test moves')
        for old, new in moves.items():
            if not (args.repo / old).exists() or (args.repo / new).exists():
                raise ValueError('missing source or occupied destination: ' + old)
        for old, new in moves.items():
            (args.repo / new).parent.mkdir(parents=True, exist_ok=True)
            subprocess.run(['git', '-C', str(args.repo), 'mv', old, new], check=True)
        print(json.dumps({'moves': len(moves), 'contentEdits': 0}))
        return
    selected = [path for path in source_paths if path.endswith('.rs') and not PROTECTED.intersection(Path(path).parts)]
    payload = ''.join(args.base + ':' + path + '\n' for path in selected)
    process = subprocess.run(['git', '-C', str(args.repo), 'cat-file', '--batch'], input=payload.encode(), stdout=subprocess.PIPE, check=True)
    sources = {}
    offset = 0
    for path in selected:
        end = process.stdout.index(b'\n', offset)
        size = int(process.stdout[offset:end].split()[-1])
        offset = end + 1
        sources[path] = process.stdout[offset:offset+size].decode()
        offset += size + 1
    targets = names.targets(sources)
    changed = []
    writes = []
    for path, original in sources.items():
        expected = rewrite(path, original, targets)
        target = args.repo / names.destination(path)
        if expected != original:
            changed.append(path)
        if args.verify:
            actual = target.read_text()
            if actual != expected and canonical(actual) != canonical(expected):
                formatted = subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true'], input=expected, text=True, stdout=subprocess.PIPE, check=True, cwd=args.repo).stdout
                if canonical(actual) != canonical(formatted):
                    raise ValueError('replay mismatch: ' + str(target))
        elif expected != original:
            actual = target.read_text()
            if actual == original:
                writes.append((target, expected))
            elif canonical(actual) != canonical(expected):
                formatted = subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true'], input=expected, text=True, stdout=subprocess.PIPE, check=True, cwd=args.repo).stdout
                if canonical(actual) != canonical(formatted):
                    raise ValueError('refusing changed consumer: ' + str(target))
    ledger = 'docs/davinci/plan/storage-inventory.tsv'
    original = git(args.repo, 'show', args.base + ':' + ledger)
    expected = migrate_storage(original)
    actual = (args.repo / ledger).read_text()
    if args.verify:
        if actual != expected:
            raise ValueError('storage path inventory replay mismatch')
    elif actual == original:
        writes.append((args.repo / ledger, expected))
    elif actual != expected:
        raise ValueError('refusing changed storage inventory')
    for target, expected in writes:
        target.write_text(expected)
    print(json.dumps({'base': args.base, 'rewrittenRustFiles': len(changed), 'verified': args.verify, 'writes': len(writes)}))


if __name__ == '__main__':
    main()
