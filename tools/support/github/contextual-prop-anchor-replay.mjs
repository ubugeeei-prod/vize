import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  readFileSync,
  writeFileSync,
  appendFileSync,
  readdirSync,
  realpathSync,
  copyFileSync,
  cpSync,
} from "node:fs";
import { join, resolve, relative, dirname } from "node:path";

const mode = process.argv[2],
  directory = resolve(process.argv[3]);
const hash = (b) => createHash("sha256").update(b).digest("hex");
const json = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + "\n");
if (mode === "prepare") {
  assert.equal(process.version, "v26.12.1");
  const physical = (root, version) => {
    const store = join(root, "node_modules/.pnpm");
    const names = readdirSync(store).filter(
      (n) => n === `@typescript+typescript-linux-x64@${version}`,
    );
    assert.equal(names.length, 1);
    return realpathSync(
      join(store, names[0], "node_modules/@typescript/typescript-linux-x64/lib/tsc"),
    );
  };
  const native = physical("source", "7.0.2"),
    beta = physical("reporter", "7.1.0-beta");
  const providers = [];
  for (const [name, path, version] of [
    ["native-702", native, "7.0.2"],
    ["beta", beta, "7.1.0-beta"],
  ]) {
    const result = spawnSync(path, ["--version"]);
    assert.equal(result.status, 0);
    assert.deepEqual(result.stderr, Buffer.alloc(0));
    assert.equal(result.stdout.toString(), `Version ${version}\n`);
    const sha256 = hash(readFileSync(path));
    if (name === "native-702")
      assert.equal(sha256, "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4");
    cpSync(dirname(path), join(directory, `provider-${name}`), { recursive: true });
    providers.push({
      name,
      path,
      version,
      sha256,
      versionStdout: [...result.stdout],
      versionStderr: [...result.stderr],
    });
  }
  const args = ["--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"];
  const before = spawnSync(beta, args, { cwd: resolve("reporter") });
  writeFileSync(join(directory, "public-before.stdout"), before.stdout);
  writeFileSync(join(directory, "public-before.stderr"), before.stderr);
  json(join(directory, "public-before.json"), {
    argv: [beta, ...args],
    status: before.status,
    stdout: [...before.stdout],
    stderr: [...before.stderr],
  });
  assert.equal(before.status, 2);
  assert.deepEqual(before.stderr, Buffer.alloc(0));
  assert.equal(
    before.stdout.toString(),
    "src/Route.vue(147,8): error TS2552: Cannot find name 'from'. Did you mean '_from'?\n  This location is in virtual code produced by the content mapper 'vize@0.569.0' and has no corresponding location in the original file.\n",
  );
  const manifests = {};
  for (const name of ["vize", "vue", "typescript"]) {
    const p = realpathSync(`reporter/node_modules/${name}`),
      bytes = readFileSync(join(p, "package.json"));
    manifests[name] = { physicalPath: p, sha256: hash(bytes), value: JSON.parse(bytes.toString()) };
  }
  json(join(directory, "provider-custody.json"), {
    source: readFileSync(join(directory, "source.sha"), "utf8").trim(),
    node: process.version,
    providers,
    manifests,
  });
  appendFileSync(
    process.env.GITHUB_ENV,
    `VIZE_TEST_CONTEXTUAL_PROP_NATIVE_TSGO=${native}\nVIZE_TEST_CONTENT_MAPPER_TSGO=${beta}\nVIZE_TEST_CONTENT_MAPPER_VUE=${realpathSync("reporter/node_modules/vue")}\nCORSA_PATH=${native}\n`,
  );
} else if (mode === "verify") {
  assert.equal(readFileSync(join(directory, "actual-hosts.status"), "utf8"), "0\n");
  for (const [name, count] of [
    ["generator", 6],
    ["batch", 1],
    ["actual-hosts", 1],
  ]) {
    assert(
      readFileSync(join(directory, `${name}.stdout`), "utf8").includes(
        `test result: ok. ${count} passed; 0 failed; 0 ignored;`,
      ),
      name,
    );
  }
  const root = resolve("source/target/vize-tests/contextual-prop-anchors-10661"),
    paths = [];
  function walk(p) {
    for (const e of readdirSync(p, { withFileTypes: true })) {
      const f = join(p, e.name);
      if (e.isDirectory()) walk(f);
      else if (e.isFile() && f.endsWith(".json")) paths.push(f);
    }
  }
  walk(root);
  const packets = [],
    counts = {};
  const known = JSON.parse(
    readFileSync(new URL("./contextual-prop-anchor-known-baseline.json", import.meta.url), "utf8"),
  );
  let knownCount = 0;
  for (const path of paths) {
    const bytes = readFileSync(path),
      p = JSON.parse(bytes.toString());
    if (p.schema !== "vize.contextual-prop-anchor-host") continue;
    assert.deepEqual(p.stderr, [], path);
    const stem = path.slice(0, -5);
    assert.deepEqual(readFileSync(`${stem}.stdout.raw`), Buffer.from(p.stdout), path);
    assert.deepEqual(readFileSync(`${stem}.stderr.raw`), Buffer.from(p.stderr), path);
    counts[p.host] = (counts[p.host] ?? 0) + 1;
    const recordPath = relative(root, path),
      baseline = known.records[recordPath];
    if (baseline) {
      assert.equal(p.status, baseline.status, recordPath);
      assert.deepEqual(p.stdout, baseline.stdout, recordPath);
      assert.deepEqual(p.stderr, baseline.stderr, recordPath);
      for (const [field, sha256] of Object.entries(baseline.inputSha256))
        assert.equal(
          hash(
            typeof p.inputs[field] === "string"
              ? Buffer.from(p.inputs[field])
              : Buffer.from(p.inputs[field]),
          ),
          sha256,
          `${recordPath}:${field}`,
        );
      knownCount++;
    }
    packets.push({
      path: recordPath,
      sha256: hash(bytes),
      host: p.host,
      status: p.status,
      stdoutBytes: p.stdout.length,
      classification: baseline
        ? known.classification
        : recordPath.split("/")[1] === "authored-ts2552-retained"
          ? "authored-ts2552-retained"
          : "complete-clean",
    });
  }
  assert.deepEqual(counts, { "mapper-beta": 40, "native-raw-702": 40, "native-raw-beta": 40 });
  assert.equal(packets.length, 120);
  assert.equal(knownCount, 12);
  assert.equal(packets.filter((p) => p.stdoutBytes === 0 && p.status === 0).length, 96);
  assert.equal(packets.filter((p) => p.stdoutBytes > 0).length, 24);
  json(join(directory, "verification.json"), {
    source: readFileSync(join(directory, "source.sha"), "utf8").trim(),
    counts,
    packets,
    knownUnresolvedControl: {
      source: known.source,
      run: known.run,
      packets: knownCount,
      wholeOutputsAndGeneratedProgramsUnchanged: true,
    },
    wholeStderrEmpty: true,
  });
} else throw new Error("unregistered replay mode");
