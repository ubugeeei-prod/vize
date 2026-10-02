import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {gzipSync} from 'node:zlib';
import {execFileSync} from 'node:child_process';

const source = path.resolve(process.argv[2]);
const mode = process.argv[3];
const out = process.env.RUNNER_TEMP!;
const sha = (bytes: Buffer | string) => crypto.createHash('sha256').update(bytes).digest('hex');
const git = (args: string[]) => execFileSync('git', ['-C', source, ...args], {maxBuffer: 4e6}).toString().trim();
const lockPath = path.join(source, 'Cargo.lock');
const beforeFile = path.join(out, 'wasmtime-before.json');
const afterFile = path.join(out, 'wasmtime-after.json');
const base = '688d47bb61c4eda7ebe9eb2b8e5f4a2348e15f67';
const expected = '2576a9b5e5d4336e5da362f048d3aaa3f2bc11ae';
const pins = () => Object.fromEntries(['Cargo.toml', 'Cargo.lock', '.github/workflows/check.yml',
  'davinci/vize_extension_host/Cargo.toml',
  ...['expression-echo', 'output-echo', 'typed-expression-echo'].map(guest =>
    'davinci/vize_extension_host/tests/guests/' + guest + '/Cargo.lock')
].map(p => [p, sha(fs.readFileSync(path.join(source, p)))]));
const before = () => JSON.parse(fs.readFileSync(beforeFile, 'utf8'));
const packageRows = (bytes: Buffer) => bytes.toString().split('[[package]]\n').slice(1).map(block => ({
  name: block.match(/^name = "([^"]+)"/m)?.[1], version: block.match(/^version = "([^"]+)"/m)?.[1],
  source: block.match(/^source = "([^"]+)"/m)?.[1] ?? null,
  checksum: block.match(/^checksum = "([^"]+)"/m)?.[1] ?? null,
  dependencies: block.match(/^dependencies = \[\n([\s\S]*?)\n\]/m)?.[1] ?? null,
  completeBlockSha256: sha(block), completeBlock: block
}));
const emit = (name: string, bytes: Buffer) => {
  const compressed = gzipSync(bytes), encoded = compressed.toString('base64'), width = 3000;
  const chunks = Math.ceil(encoded.length / width);
  console.log('VIZE_WASMTIME_BEGIN ' + JSON.stringify({name, bytes: bytes.length, sha256: sha(bytes),
    gzipBytes: compressed.length, gzipSha256: sha(compressed), chunks}));
  for (let i = 0; i < chunks; i++) console.log('VIZE_WASMTIME_FRAME ' + name + ' ' + i + '/' + chunks + ' ' + encoded.slice(i * width, (i + 1) * width));
  console.log('VIZE_WASMTIME_END ' + name);
};
if (mode === 'before') {
  assert.equal(git(['rev-parse', 'HEAD']), expected); assert.equal(git(['log', '-1', '--format=%P']), base);
  assert.equal(git(['status', '--porcelain=v1']), '');
  const inputs = pins();
  assert.equal(inputs['Cargo.toml'], '3ca8556b3ebdcafbb5d619cabaf8ee7dd355e396d18065e73c2b7cd6c4446428');
  assert.equal(inputs['Cargo.lock'], '8e03a98133b3f0be46e7cab5911426bb6204bee5db0c313e4181392a098362c1');
  assert.equal(inputs['.github/workflows/check.yml'], '398e00fcb45fd011ef8e77bdb7400f8a74c2adb2ecde2424f11efaf4fb2fcf90');
  const cargo = execFileSync('cargo', ['--version']).toString().trim(); assert.match(cargo, /^cargo 1\.98\.0 /);
  const record = {source: expected, parent: base, sourceTree: git(['rev-parse', 'HEAD^{tree}']), inputs, cargo,
    rustc: execFileSync('rustc', ['--version']).toString().trim(), node: process.version,
    workflowCommit: process.env.GITHUB_SHA, runId: process.env.GITHUB_RUN_ID, attempt: process.env.GITHUB_RUN_ATTEMPT,
    command: ['cargo', 'update', '-p', 'wasmtime', '--precise', '48.0.4'], sourceLock: fs.readFileSync(lockPath).toString('base64')};
  fs.writeFileSync(beforeFile, JSON.stringify(record, null, 2) + '\n');
} else if (mode === 'after') {
  const initial = before(); assert.equal(git(['rev-parse', 'HEAD']), expected);
  assert.deepEqual(git(['diff', '--name-only']).split('\n'), ['Cargo.lock']);
  assert.equal(git(['status', '--porcelain=v1']), 'M Cargo.lock');
  const inputs = pins(); for (const [p, hash] of Object.entries(initial.inputs)) if (p !== 'Cargo.lock') assert.equal(inputs[p], hash);
  const old = packageRows(Buffer.from(initial.sourceLock, 'base64')), current = packageRows(fs.readFileSync(lockPath));
  assert.equal(current.filter(row => row.name === 'wasmtime').length, 1);
  assert.equal(current.find(row => row.name === 'wasmtime')?.version, '48.0.4');
  const diff = [...new Set([...old.map(r => r.name), ...current.map(r => r.name)])].flatMap(name => {
    const a = old.filter(r => r.name === name), b = current.filter(r => r.name === name);
    return JSON.stringify(a) === JSON.stringify(b) ? [] : [{name, before: a, after: b}];
  });
  const completeCargoLockDiff = execFileSync('git', ['-C', source, 'diff', '--no-ext-diff', '--', 'Cargo.lock'], {maxBuffer: 4e6}).toString();
  fs.writeFileSync(afterFile, JSON.stringify({source: expected, pins: inputs, changedPackages: diff, completeCargoLockDiff,
    packageCounts: {before: old.length, after: current.length}, scope: 'Actual resolver output only; minimal closure and audit/runtime acceptance require later independent review.'}, null, 2) + '\n');
} else if (mode === 'emit') {
  console.log('VIZE_WASMTIME_STATUS ' + JSON.stringify({workflowCommit: process.env.GITHUB_SHA,
    source: expected, before: process.env.BEFORE_OUTCOME, resolver: process.env.RESOLVER_OUTCOME, guard: process.env.GUARD_OUTCOME,
    runId: process.env.GITHUB_RUN_ID, attempt: process.env.GITHUB_RUN_ATTEMPT, sourceCheckout: fs.existsSync(path.join(source, '.git'))}));
  if (fs.existsSync(beforeFile)) emit('before.json', fs.readFileSync(beforeFile));
  if (fs.existsSync(lockPath)) emit('Cargo.lock', fs.readFileSync(lockPath));
  if (fs.existsSync(afterFile)) emit('after.json', fs.readFileSync(afterFile));
} else throw Error('Unknown capture mode');
