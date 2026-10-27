"""Materialize immutable #10845/#10682 audit cases; no corpus mutation."""
import hashlib
import json
import re
import tarfile
import sys
from pathlib import Path

root, destination = map(Path, sys.argv[1:])
destination.mkdir(parents=True)
corpus = root / 'tests/fuzz/regressions/js_ts_expression'
expected = {
    'issue-10845.input': '60c49daaab7e06a062b0ffa7dced0eb4ddab9dbee1b52f1657fc4f1c08aa14a5',
    'issue-10845.slow-1.input': 'de0029da1a229d4e336d73374bedd33e64649c741a28752d56b4dee40076f60a',
    'issue-10845.slow-2.input': 'e73212cb497085f26e85b59ef09aabc4bbdb485965f71ebb8899dc666b89e293',
    'issue-10845.min.input': '73e51fe565438b9e18d8696f4bc57d368310898539e1f406387b7592a9e1a926',
    'issue-10401.input': '6d668844f745732cdc92bdafa21f72ba5d09d9c14305d4ea9174fb802493dcc8',
}
manifest = []

def emit(name, data, admitted):
    (destination / name).write_bytes(data)
    manifest.append({'name': name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'require_admission': admitted})

for name, digest in expected.items():
    data = (corpus / name).read_bytes()
    assert hashlib.sha256(data).hexdigest() == digest, name
    emit(name, data, True)
assert (root / 'vendor/oxc_parser/src/speculation/fixtures/issue-10845.input').read_bytes() == (corpus / 'issue-10845.input').read_bytes()
original = (corpus / 'issue-10401.input').read_bytes()
for label, suffix in [('x', b'\n// x'), ('greater', b'\n// >')]:
    emit(f'10401-append-{label}.input', original + suffix, True)
for size in [4096, 8192, 16384]:
    # Historical #10682 raw size controls: admission rejects these prefixes.
    emit(f'10401-prefix-{size}.input', original[:size] + b'\n// >', False)
for name, source in [
    ('source-fact-jsdoc.input', b'/** @type {number} */ const x = 1;'),
    ('source-fact-legacy.input', b'077'),
    ('source-fact-generics.input', b'const result = f<Map<string, Array<number>>>(value);'),
]:
    emit(name, source, True)
for count in [64, 128, 256]:
    emit(f'minimal-overlap-{count}.input', ('a < [a < [' * count + '>').encode(), False)
archive = root / 'tools/support/compat/parser/historical-10682.tar.gz'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == '780df7dfa9517de9155d5dffa8ec167a1f9c98391f1dd107f735be80d361185a'
observer = root / 'tools/support/compat/parser/frozen-10682-observer.rs'
assert hashlib.sha256(observer.read_bytes()).hexdigest() == '9d095b94ec587d9f22cfa696a1bb5c6bd23813eced6a633d2989c6a270f05238'
packets = destination / 'historical-packets'
packets.mkdir()
with tarfile.open(archive, 'r:gz') as retained:
    def read(name):
        return retained.extractfile(name).read()
    index = json.loads(read('index.json'))
    (packets / 'index.json').write_bytes(read('index.json'))
    for case in index['cases']:
        name = case['id']
        assert re.fullmatch(r'[a-z0-9-]+', name), name
        data = read(f'cases/{name}.input')
        assert hashlib.sha256(data).hexdigest() == case['input_sha256']
        emit(f'historical-10682-{name}.input', data, case['require_admission'])
        manifest[-1]['historical_id'] = name
        for kind in ['controls', 'program-observer']:
            data = read(f'packets/{name}.{kind}.json')
            assert hashlib.sha256(data).hexdigest() == case[kind+'_sha256']
            (packets/f'{name}.{kind}.json').write_bytes(data)
(destination / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
