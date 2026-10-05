import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { isAbsolute, join, relative } from "node:path";

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const injected = "apps/web-antd/src/__vize_batch_incremental_oracle__.vue";

// This observer runs outside the unchanged integration test's timing windows.
export function prepareSourceCustody({ fixture, capture, output, driver, authority }) {
  const git = (...args) =>
    execFileSync("git", ["-C", fixture, ...args], { maxBuffer: 64 * 1024 * 1024 });
  const fixtureState = () => ({
    head: git("rev-parse", "HEAD").toString().trim(),
    porcelainBase64: git("status", "--porcelain=v1", "-z", "--untracked-files=all").toString(
      "base64",
    ),
  });
  const originalState = fixtureState();
  assert.equal(originalState.head, authority.revision);
  const generations = ["cold", "brokenWarm", "repairedWarm"].map((phase) => {
    const bytes = readFileSync(join(capture, phase + ".json"));
    const packet = JSON.parse(bytes);
    assert.deepEqual(packet.fixture, authority);
    assert.equal(packet.wholeGeneration.observed.mode, "native-bulk");
    assert.equal(packet.wholeGeneration.observed.bulk.release, "acknowledged");
    const sources = packet.wholeGeneration.generatedFiles.map((file) => file.original);
    assert.equal(sources.length, 681);
    assert.equal(new Set(sources).size, 681);
    assert.equal(sources.filter((path) => path.endsWith(".vue")).length, 500);
    assert.equal(sources.filter((path) => path.endsWith(".ts")).length, 181);
    assert.equal(packet.vuePaths.length, 500);
    assert.deepEqual(
      [...packet.vuePaths].sort(),
      sources.filter((path) => path.endsWith(".vue")).sort(),
    );
    return {
      phase,
      captureSha256: digest(bytes),
      sources,
      vuePaths: packet.vuePaths,
      injectedSource: packet.injectedSource,
      generatedConfig: packet.wholeGeneration.generatedConfig,
    };
  });
  for (const generation of generations) {
    assert.deepEqual(generation.sources, generations[0].sources);
    assert.deepEqual(generation.vuePaths, generations[0].vuePaths);
  }
  const literal = (name) => {
    const match = driver.match(new RegExp("const " + name + ': &str = r#"([\\s\\S]*?)"#;'));
    assert(match, "qualified fixture driver has no literal " + name);
    return Buffer.from(match[1]);
  };
  const clean = literal("CLEAN_SOURCE");
  const broken = literal("BROKEN_SOURCE");
  assert.equal(generations[0].injectedSource, clean.toString());
  assert.equal(generations[1].injectedSource, broken.toString());
  assert.equal(generations[2].injectedSource, clean.toString());
  const tree = new Map(
    git("ls-tree", "-r", "-z", "HEAD")
      .toString()
      .split("\0")
      .filter(Boolean)
      .map((row) => {
        const tab = row.indexOf("\t");
        const [mode, kind, blob] = row.slice(0, tab).split(" ");
        assert.equal(kind, "blob");
        return [row.slice(tab + 1), { mode, blob }];
      }),
  );
  const selected = generations[0].sources.map((path) => {
    const name = relative(fixture, path);
    assert(isAbsolute(path) && name && !name.startsWith("../") && !isAbsolute(name));
    assert.equal(join(fixture, name), path, "foreign or normalized source identity");
    return name;
  });
  assert(selected.includes(injected));
  // Include every authored tracked config/manifest, without interpreting extends.
  const names = [
    ...new Set([
      ...selected.filter((name) => name !== injected),
      ...[...tree.keys()].filter((name) => /\.(?:json|jsonc|ya?ml)$/.test(name)),
    ]),
  ].sort();
  const originals = names.map((path) => {
    const entry = tree.get(path);
    assert(entry && ["100644", "100755"].includes(entry.mode), "unowned body " + path);
    const bytes = readFileSync(join(fixture, path));
    const blob = createHash("sha1")
      .update("blob " + bytes.length + "\0")
      .update(bytes)
      .digest("hex");
    return {
      path,
      expectedGitBlob: entry.blob,
      gitBlob: blob,
      matches: blob === entry.blob,
      sha256: digest(bytes),
      bytes: bytes.length,
      base64: bytes.toString("base64"),
    };
  });
  const catalog = {
    fixtureState: originalState,
    stateScope:
      "Git porcelain detects tracked/untracked membership changes; ignored paths are not enumerated. No full ignored-directory selection census.",
    selectedSources: selected,
    originals,
    generations,
    injected: {
      path: injected,
      clean: { bytes: clean.length, sha256: digest(clean), base64: clean.toString("base64") },
      broken: { bytes: broken.length, sha256: digest(broken), base64: broken.toString("base64") },
      restoredState: "absent before and after each integration process",
    },
  };
  const catalogBytes = Buffer.from(JSON.stringify(catalog, null, 2) + "\n");
  const catalogPath = join(output, "source-catalog.json");
  writeFileSync(catalogPath, catalogBytes);
  assert(
    originals.every((row) => row.matches),
    "physical source/config differs from immutable fixture",
  );
  const observations = [];
  return {
    catalogPath,
    catalogSha256: digest(catalogBytes),
    verify(phase) {
      const rows = originals.map((expected) => {
        try {
          const bytes = readFileSync(join(fixture, expected.path));
          const sha256 = digest(bytes);
          return {
            path: expected.path,
            sha256,
            matches: sha256 === expected.sha256,
            ...(sha256 === expected.sha256 ? {} : { changedBase64: bytes.toString("base64") }),
          };
        } catch (error) {
          return { path: expected.path, matches: false, error: error.message };
        }
      });
      const injectedAbsent = !existsSync(join(fixture, injected));
      const state = fixtureState();
      const observation = { phase, rows, injectedAbsent, fixtureState: state };
      if (!injectedAbsent)
        observation.unrestoredInjectedBase64 = readFileSync(join(fixture, injected)).toString(
          "base64",
        );
      observations.push(observation);
      writeFileSync(
        join(output, "source-validation.json"),
        JSON.stringify(observations, null, 2) + "\n",
      );
      assert(injectedAbsent, phase + ": integration did not restore its owned injected file");
      assert(
        rows.every((row) => row.matches),
        phase + ": original source/config bodies changed",
      );
      assert.deepEqual(
        state,
        originalState,
        phase + ": fixture revision or visible membership changed",
      );
    },
  };
}
