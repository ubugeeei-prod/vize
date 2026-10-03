import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { gunzipSync, gzipSync } from 'node:zlib';
import { execFileSync } from 'node:child_process';

const baseline = '05b8b11b90168c59be71e4a1f8a529f2ca7028dd';
const sha = (bytes, algorithm = 'sha256') => crypto.createHash(algorithm).update(bytes).digest('hex');
const record = (bytes) => ({ bytes: bytes.length, sha256: sha(bytes), base64: bytes.toString('base64') });
const evidence = {
  schema: 1,
  label: 'official-vue2-npm-oracle',
  capturedAt: new Date().toISOString(),
  node: process.version,
  source: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
  baseline,
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  packages: [],
};

async function download(url) {
  assert.equal(new URL(url).protocol, 'https:');
  assert.equal(new URL(url).hostname, 'registry.npmjs.org');
  const response = await fetch(url, { signal: AbortSignal.timeout(60000) });
  const bytes = Buffer.from(await response.arrayBuffer());
  assert.equal(new URL(response.url).hostname, 'registry.npmjs.org');
  assert.equal(response.status, 200);
  assert.ok(bytes.length > 0 && bytes.length <= 10 * 1024 * 1024);
  return {
    url,
    finalUrl: response.url,
    status: response.status,
    headers: Object.fromEntries(response.headers.entries()),
    ...record(bytes),
  };
}

function tarFiles(tgz) {
  const tar = gunzipSync(tgz, { maxOutputLength: 20 * 1024 * 1024 });
  const files = [];
  let offset = 0;
  let ended = false;
  const string = (buffer) => buffer.toString('utf8').replace(/\0.*$/s, '');
  const octal = (buffer) => {
    const value = string(buffer).trim();
    assert.match(value, /^[0-7]+$/);
    return Number.parseInt(value, 8);
  };
  while (offset + 512 <= tar.length) {
    const header = tar.subarray(offset, offset + 512);
    if (header.every((byte) => byte === 0)) {
      assert.ok(tar.subarray(offset).every((byte) => byte === 0));
      ended = true;
      break;
    }
    const expectedChecksum = octal(header.subarray(148, 156));
    const actualChecksum = header.reduce((sum, byte, index) => sum + (index >= 148 && index < 156 ? 32 : byte), 0);
    assert.equal(actualChecksum, expectedChecksum);
    const prefix = string(header.subarray(345, 500));
    const name = (prefix ? prefix + '/' : '') + string(header.subarray(0, 100));
    assert.ok(name.startsWith('package/'));
    assert.ok(!name.split('/').some((part) => part === '..' || part === '.'));
    const size = octal(header.subarray(124, 136));
    const kind = header[156];
    assert.ok(kind === 0 || kind === 48 || kind === 53, 'Only ordinary npm files/directories are admitted');
    assert.ok(offset + 512 + size <= tar.length);
    if (kind !== 53) {
      assert.ok(!files.some((file) => file.path === name), 'Duplicate tar file');
      const contents = Buffer.from(tar.subarray(offset + 512, offset + 512 + size));
      files.push({ path: name, ...record(contents) });
    } else assert.equal(size, 0);
    offset += 512 + Math.ceil(size / 512) * 512;
  }
  assert.ok(ended, 'Missing complete tar terminator');
  return { uncompressed: { bytes: tar.length, sha256: sha(tar) }, files };
}

async function capture(name, version) {
  const entry = { name, version };
  evidence.packages.push(entry);
  entry.registry = await download('https://registry.npmjs.org/' + name + '/' + version);
  const metadata = JSON.parse(Buffer.from(entry.registry.base64, 'base64'));
  assert.equal(metadata.name, name);
  assert.equal(metadata.version, version);
  entry.publishedIntegrity = metadata.dist.integrity;
  entry.publishedShasum = metadata.dist.shasum;
  entry.tarball = await download(metadata.dist.tarball);
  const tgz = Buffer.from(entry.tarball.base64, 'base64');
  assert.equal(sha(tgz, 'sha1'), metadata.dist.shasum);
  assert.match(metadata.dist.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/);
  assert.equal(crypto.createHash('sha512').update(tgz).digest('base64'), metadata.dist.integrity.slice(7));
  entry.publishedSRIAndShasumVerified = true;
  entry.tar = tarFiles(tgz);
  const file = (path) => {
    const matches = entry.tar.files.filter((item) => item.path === 'package/' + path);
    assert.equal(matches.length, 1, 'Expected unique original ' + path);
    return matches[0];
  };
  const packageJson = JSON.parse(Buffer.from(file('package.json').base64, 'base64'));
  assert.equal(packageJson.name, name);
  assert.equal(packageJson.version, version);
  entry.packageJsonVerified = true;
  entry.licenseFiles = entry.tar.files.filter((item) => /^package\/licen[cs]e(?:[.-]|$)/i.test(item.path)).map((item) => item.path);
  assert.ok(entry.licenseFiles.length > 0, 'Original package license is mandatory');
  return { entry, packageJson, file };
}

try {
  assert.equal(execFileSync('git', ['rev-parse', 'HEAD^'], { encoding: 'utf8' }).trim(), baseline);
  const compiler = await capture('vue-template-compiler', '2.7.16');
  const build = Buffer.from(compiler.file('build.js').base64, 'base64').toString('utf8');
  compiler.file('LICENSE');
  const requires = [...build.matchAll(/\brequire\(\s*(['"])([^'"]+)\1\s*\)/g)].map((match) => match[2]);
  compiler.entry.literalRequires = [...new Set(requires)].sort();
  assert.deepEqual(compiler.entry.literalRequires, ['de-indent', 'he']);
  assert.equal(compiler.packageJson.dependencies['de-indent'], '^1.0.2');
  assert.equal(compiler.packageJson.dependencies.he, '^1.2.0');
  await capture('de-indent', '1.0.2');
  await capture('he', '1.2.0');
  evidence.success = true;
} catch (error) {
  evidence.success = false;
  evidence.error = { name: error.name, message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  const raw = Buffer.from(JSON.stringify(evidence));
  const gzip = gzipSync(raw, { mtime: 0 });
  const base64 = gzip.toString('base64');
  const chunkSize = 16000;
  const chunks = Math.ceil(base64.length / chunkSize);
  const frame = { label: evidence.label, rawBytes: raw.length, rawSha256: sha(raw), gzipBytes: gzip.length, gzipSha256: sha(gzip), chunks };
  console.log('VIZE_VUE2_BEGIN ' + JSON.stringify(frame));
  for (let index = 0; index < chunks; index++) console.log('VIZE_VUE2_CHUNK ' + index + '/' + chunks + ' ' + base64.slice(index * chunkSize, (index + 1) * chunkSize));
  console.log('VIZE_VUE2_END ' + JSON.stringify(frame));
}
