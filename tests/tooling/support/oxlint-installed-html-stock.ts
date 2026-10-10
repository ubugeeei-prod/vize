import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  installedHtmlPreflight,
  sourceRoot,
  wholeProcessError,
} from "./oxlint-installed-html-authority.ts";
import { exactPath, sha256 } from "./n8n-installed-authority.ts";

// Adapt the unchanged reviewed collector libraries; never extract or execute a public archive.
export const stockArchiveProgram = String.raw`
import hashlib, importlib.util, json, pathlib, sys
sys.dont_write_bytecode = True
plans = json.load(sys.stdin)
if (not isinstance(plans, list) or not 2 <= len(plans) <= 10
    or {item.get("version") for item in plans} not in ({"1.78.0"}, {"1.86.0"})
    or "oxlint" not in {item.get("name") for item in plans}
    or "@oxlint/binding-darwin-arm64" not in {item.get("name") for item in plans}
    or any(item.get("name") != "oxlint" and not item.get("name", "").startswith("@oxlint/binding-") for item in plans)):
    raise ValueError("finite pinned stock wrapper and native public archive plans required")
root = pathlib.Path(sys.argv[1])
modules = []
for name in ("identity", "registry", "archive"):
    filename = root / "tools/support/release/public_install" / (name + ".py")
    if filename.resolve(strict=True) != filename:
        raise ValueError("reviewed collector module path redirected")
    data = filename.read_bytes()
    modules.append({"path": str(filename), "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)})
    spec = importlib.util.spec_from_file_location(name, filename)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
identity = sys.modules["identity"]
registry = sys.modules["registry"]
archive = sys.modules["archive"]
identity.reject_overrides(__import__("os").environ)
proofs = []
for item in plans:
    metadata = []
    def observed_read(url):
        value = registry.read(url)
        metadata.append({"url": url, "metadata": value})
        return value
    registry.package_metadata(item["name"], item["version"], item, reader=observed_read)
    payload = registry.read_bytes(item["resolved"])
    if len(payload) > 64 * 1024 * 1024:
        raise ValueError("pinned stock archive exceeds its finite 64 MiB bound")
    comparison = archive.archive_comparison(pathlib.Path(item["directory"]), payload, item["integrity"])
    proofs.append({"input": item, "metadataResponses": metadata,
                   "archiveByteLength": len(payload), "archiveSHA256": hashlib.sha256(payload).hexdigest(),
                   "comparison": comparison,
                   "scope": "third-party stock public archive equality; no Vize provenance or semantic oracle claim"})
interpreter = pathlib.Path(sys.executable).resolve(strict=True)
print(json.dumps({"schema": "vize.oxlint.stock-public-archives-v1", "modules": modules,
                  "python": {"path": str(interpreter), "version": sys.version, "sha256": hashlib.sha256(interpreter.read_bytes()).hexdigest()},
                  "packages": proofs}))
`;

export interface StockArchiveInput {
  name: string;
  version: string;
  resolved: string;
  integrity: string;
  directory: string;
  files: Array<{ path: string; bytes: number; sha256: string }>;
  [field: string]: unknown;
}

export function verifyStockArchives(packages: StockArchiveInput[], receiptPath: string) {
  installedHtmlPreflight();
  const input = JSON.stringify(packages);
  const args = ["-I", "-c", stockArchiveProgram, sourceRoot];
  const result = spawnSync("python3", args, {
    cwd: sourceRoot,
    input,
    timeout: 60_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const packet = {
    scope: "stock public archive custody only; no installed Vize execution or semantic credit",
    command: "python3",
    args,
    input,
    adapterSha256: sha256(stockArchiveProgram),
    pid: result.pid,
    status: result.status,
    signal: result.signal,
    error: result.error ? wholeProcessError(result.error) : null,
    stdoutBytes: Array.from(result.stdout ?? []),
    stderrBytes: Array.from(result.stderr ?? []),
  };
  fs.writeFileSync(receiptPath, JSON.stringify(packet, null, 2) + "\n", { flag: "wx" });
  assert.equal(result.error, undefined, JSON.stringify(packet));
  assert.equal(result.signal, null, JSON.stringify(packet));
  assert.equal(result.status, 0, result.stdout.toString("utf8") + result.stderr.toString("utf8"));
  assert.equal(result.stderr.length, 0);
  const proof = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(result.stdout));
  assert.equal(proof.schema, "vize.oxlint.stock-public-archives-v1");
  assert.equal(exactPath(proof.python.path), proof.python.path);
  assert.equal(sha256(fs.readFileSync(proof.python.path)), proof.python.sha256);
  assert.equal(typeof proof.python.version, "string");
  assert.equal(proof.packages.length, packages.length);
  assert.deepEqual(
    proof.modules,
    ["identity", "registry", "archive"].map((name) => {
      const filename = path.join(sourceRoot, "tools/support/release/public_install", name + ".py");
      const bytes = fs.readFileSync(filename);
      return { path: filename, sha256: sha256(bytes), bytes: bytes.length };
    }),
  );
  for (let index = 0; index < packages.length; index++) {
    const actual = proof.packages[index];
    assert.deepEqual(actual.input, packages[index]);
    assert.equal(actual.comparison.allInstalledFileBytesEqualPublicArchive, true);
    assert.equal(actual.comparison.fileCount, packages[index].files.length);
    assert.deepEqual(actual.comparison.files, packages[index].files);
    assert.equal(
      actual.comparison.tarballSHA512,
      Buffer.from(packages[index].integrity.slice(7), "base64").toString("hex"),
    );
  }
  return { receiptPath, receiptSha256: sha256(fs.readFileSync(receiptPath)), packet, proof };
}
