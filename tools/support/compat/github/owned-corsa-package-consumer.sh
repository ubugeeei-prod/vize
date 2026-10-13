#!/usr/bin/env bash
set -euo pipefail
: "${RUNNER_TEMP:?}" "${SOURCE_SHA:?}" "${CARGO_TARGET_DIR:?}"
node --input-type=module <<'JS'
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';

assert.equal(process.platform, 'linux');
const root = fs.realpathSync(process.cwd());
const capture = path.join(process.env.RUNNER_TEMP, 'native-source-recipe-owned-corsa-package-consumer');
assert.equal(fs.existsSync(capture), false, 'never overwrite prior evidence');
fs.mkdirSync(capture);
const parse = createRequire(path.join(root, 'package.json'))('@iarna/toml').parse;
const sha = value => createHash('sha256').update(value).digest('hex');
const names = ['corsa_core', 'corsa_jsonrpc', 'corsa_client', 'corsa_lsp', 'corsa_orchestrator', 'corsa'];
const sourceManifest = fs.readFileSync(path.join(root, 'Cargo.toml'), 'utf8');
const marker = '[profile.release]\n';
assert.equal(sourceManifest.split(marker).length, 2);
const profiles = marker + sourceManifest.split(marker)[1];
assert(profiles.includes('[profile.ci-opt]\ninherits = "release"\nlto = "thin"\ncodegen-units = 16'));
assert.equal(fs.realpathSync(process.env.CARGO_TARGET_DIR), fs.realpathSync(path.join(root, 'target')));
const fixture = 'tests/_fixtures/ci/owned-corsa-package-consumer/';
const fixtureManifest = fs.readFileSync(path.join(root, fixture, 'Cargo.toml'), 'utf8');
const fixtureSource = fs.readFileSync(path.join(root, fixture, 'src/lib.rs'));
const parsed = parse(fixtureManifest);
assert.deepEqual(Object.keys(parsed.dependencies).sort(), names.toSorted());
assert.equal(Object.hasOwn(parsed, 'patch'), false);
for (const name of names) assert.deepEqual(parsed.dependencies[name], { package: 'vize_' + name, version: '=0.443.0' });
const stage = path.join(capture, 'path-graph-vendor-seed');
const consumer = path.join(capture, 'consumer');
const negative = path.join(capture, 'default-feature-negative');
const vendor = path.join(capture, 'sealed-vendor');
for (const directory of [stage, consumer, negative]) fs.mkdirSync(path.join(directory, 'src'), { recursive: true });
let seed = fixtureManifest;
for (const name of names) {
  const before = `${name} = { package = "vize_${name}", version = "=0.443.0" }`;
  assert.equal(seed.split(before).length, 2);
  seed = seed.replace(before, before.slice(0, -2) + ', path = ' + JSON.stringify(path.join(root, 'vendor', name)) + ' }');
}
assert.equal(Object.hasOwn(parse(seed), 'patch'), false);
fs.writeFileSync(path.join(stage, 'Cargo.toml'), seed + '\n' + profiles);
fs.writeFileSync(path.join(stage, 'src/lib.rs'), fixtureSource);
fs.copyFileSync(path.join(root, 'Cargo.lock'), path.join(stage, 'Cargo.lock'));
for (const directory of [consumer, negative]) fs.writeFileSync(path.join(directory, 'Cargo.toml'), fixtureManifest + '\n' + profiles);
for (const directory of [consumer, negative]) fs.copyFileSync(path.join(root, 'Cargo.lock'), path.join(directory, 'Cargo.lock'));
fs.writeFileSync(path.join(consumer, 'src/lib.rs'), fixtureSource);
const negativeSource = 'pub use corsa::orchestrator::RaftConfig;\n';
fs.writeFileSync(path.join(negative, 'src/lib.rs'), negativeSource);
const commands = [];
function command(label, binary, args, cwd = root, expectedStatus = 0) {
  const started = new Date().toISOString();
  const result = spawnSync(binary, args, { cwd, env: { ...process.env, NODE_EXE: process.execPath }, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: 30 * 60 * 1000 });
  fs.writeFileSync(path.join(capture, label + '.stdout'), result.stdout ?? '');
  fs.writeFileSync(path.join(capture, label + '.stderr'), result.stderr ?? '');
  const outcome = { binary, args, cwd, started, finished: new Date().toISOString(), status: result.status, signal: result.signal, error: result.error?.message ?? null };
  fs.writeFileSync(path.join(capture, label + '.outcome.json'), JSON.stringify(outcome, null, 2) + '\n');
  commands.push(outcome);
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null);
  assert.equal(result.status, expectedStatus, result.stderr);
  return result.stdout;
}
const sourceSha = command('git-head', 'git', ['--no-replace-objects', 'rev-parse', 'HEAD']).trim();
assert.equal(sourceSha, process.env.SOURCE_SHA);
const rootLock = parse(fs.readFileSync(path.join(root, 'Cargo.lock'), 'utf8'));
const registryLocks = new Map(rootLock.package.filter(entry => entry.source).map(entry => [entry.name + '@' + entry.version + '@' + entry.source, entry.checksum]));
function originalRegistryLocks(directory) {
  const lock = parse(fs.readFileSync(path.join(directory, 'Cargo.lock'), 'utf8'));
  for (const entry of lock.package.filter(entry => entry.source && !entry.name.startsWith('vize_corsa'))) {
    const identity = entry.name + '@' + entry.version + '@' + entry.source;
    assert.equal(entry.checksum, registryLocks.get(identity), 'registry provider tuple remains the actual source lock: ' + identity);
  }
}
assert.equal(command('git-tracked-status', 'git', ['--no-replace-objects', 'status', '--porcelain', '--untracked-files=no']), '');
command('rustc-version', 'rustc', ['-Vv']);
command('cargo-version', 'cargo', ['-Vv']);
const sourceMetadata = JSON.parse(command('vendor-seed-metadata', 'cargo', ['metadata', '--offline', '--format-version', '1', '--features', 'distributed'], stage));
const seedOwn = sourceMetadata.packages.filter(entry => entry.name.startsWith('vize_corsa'));
originalRegistryLocks(stage);
assert.deepEqual(seedOwn.map(entry => entry.name).sort(), names.map(name => 'vize_' + name).sort());
for (const entry of seedOwn) assert.equal(fs.realpathSync(entry.manifest_path), path.join(root, 'vendor', entry.name.slice(5), 'Cargo.toml'));
command('vendor-registry-inputs', 'cargo', ['vendor', '--locked', '--offline', '--versioned-dirs', vendor], stage);
const packageTarget = path.join(capture, 'package-target');
command('package-six', 'cargo', ['package', '--locked', '--offline', '--no-verify', '--target-dir', packageTarget, ...names.flatMap(name => ['-p', 'vize_' + name])]);
command('seal-packages', 'python3', ['tools/support/compat/github/owned-corsa-package-custody.py', root, path.join(packageTarget, 'package'), vendor, path.join(root, 'tools/support/compat/github/owned-corsa-source-manifest.json'), sourceSha, path.join(capture, 'package-custody.json')]);
const config = '[source.crates-io]\nreplace-with = "sealed-package-graph"\n[source.sealed-package-graph]\ndirectory = ' + JSON.stringify(vendor) + '\n';
for (const directory of [consumer, negative]) {
  fs.mkdirSync(path.join(directory, '.cargo'));
  fs.writeFileSync(path.join(directory, '.cargo/config.toml'), config);
}
const metadataReceipts = [];
function metadata(label, directory, features) {
  const value = JSON.parse(command(label, 'cargo', ['metadata', '--offline', '--format-version', '1', ...features], directory));
  originalRegistryLocks(directory);
  const own = value.packages.filter(entry => entry.name.startsWith('vize_corsa'));
  assert.deepEqual(own.map(entry => entry.name).sort(), names.map(name => 'vize_' + name).sort());
  assert.equal(value.packages.some(entry => names.includes(entry.name)), false, 'no published upstream core/client branch');
  const runtime = value.packages.filter(entry => entry.name === 'corsa_runtime');
  assert.equal(runtime.length, 1); assert.equal(runtime[0].version, '1.14.0');
  for (const entry of own) {
    assert.equal(entry.version, '0.443.0');
    assert.equal(entry.source, 'registry+https://github.com/rust-lang/crates.io-index');
    assert.equal(fs.realpathSync(entry.manifest_path), path.join(vendor, entry.name + '-0.443.0', 'Cargo.toml'));
  }
  const orchestrator = own.find(entry => entry.name === 'vize_corsa_orchestrator');
  const facade = own.find(entry => entry.name === 'vize_corsa');
  for (const entry of [orchestrator, facade]) {
    const node = value.resolve.nodes.find(item => item.id === entry.id);
    assert.equal(node.features.includes('experimental-distributed'), features.length > 0);
  }
  metadataReceipts.push({ label, ownedPackages: own.map(({ id, name, version, manifest_path }) => ({ id, name, version, manifest: manifest_path })) });
  return value;
}
metadata('default-metadata', consumer, []);
const recipe = ['--locked', '--offline', '--profile', 'ci-opt', '--lib', '--message-format=json', '--config', 'profile.ci-opt.inherits="release"', '--config', 'profile.ci-opt.lto="thin"', '--config', 'profile.ci-opt.codegen-units=16'];
command('default-build', 'cargo', ['build', ...recipe], consumer);
metadata('distributed-metadata', consumer, ['--features', 'distributed']);
command('distributed-build', 'cargo', ['build', ...recipe, '--features', 'distributed'], consumer);
metadata('default-negative-metadata', negative, []);
const negativeOutput = command('default-negative-check', 'cargo', ['check', ...recipe], negative, 101);
const errors = negativeOutput.split('\n').filter(Boolean).map(line => JSON.parse(line)).filter(row => row.reason === 'compiler-message' && row.message.level === 'error');
assert.equal(errors.length, 1);
assert.equal(errors[0].message.code.code, 'E0432');
assert.equal(errors[0].message.message, 'unresolved import `corsa::orchestrator`');
const primary = errors[0].message.spans.filter(span => span.is_primary);
assert.equal(primary.length, 1);
const { file_name: filename, ...span } = primary[0];
assert.equal(path.resolve(negative, filename), path.join(negative, 'src/lib.rs'));
assert.deepEqual(span, {
  byte_start: 15, byte_end: 27, line_start: 1, line_end: 1, column_start: 16, column_end: 28,
  is_primary: true,
  text: [{ text: negativeSource.trim(), highlight_start: 16, highlight_end: 28 }],
  label: 'could not find `orchestrator` in `corsa`', suggested_replacement: null,
  suggestion_applicability: null, expansion: null,
});
assert.equal(command('git-final-tracked-status', 'git', ['--no-replace-objects', 'status', '--porcelain', '--untracked-files=no']), '');
assert.equal(command('git-final-head', 'git', ['--no-replace-objects', 'rev-parse', 'HEAD']).trim(), sourceSha);
fs.writeFileSync(path.join(capture, 'receipt.json'), JSON.stringify({ sourceSha, authority: 'independent-six-normalized-packages-sealed-offline-consumer', sourceManifestSHA256: sha(sourceManifest), fixtureManifestSHA256: sha(fixtureManifest), fixtureSourceSHA256: sha(fixtureSource), packageCustodySHA256: sha(fs.readFileSync(path.join(capture, 'package-custody.json'))), originalProfilesSHA256: sha(profiles), originalTarget: process.env.CARGO_TARGET_DIR, metadata: metadataReceipts, commands, defaultCompiled: true, distributedCompiled: true, defaultExperimentalSurfaceRefused: true, sourcePatches: false, directorySourceReplacement: true, publicRegistryAcceptance: false, nativeRuntimeExecution: false }, null, 2) + '\n');
JS
