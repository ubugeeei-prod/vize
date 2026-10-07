import assert from "node:assert/strict";
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { recoverFixtureCheckout, validateHydration } from "./canonical-corpus-hydration.mjs";
import { verifyCommittedInventory } from "./canonical-corpus-inventory.mjs";
import {
  expectedOldErrorReasons,
  expectedOldErrorSkips,
  parseCorpusEvidence,
  validateCorpusEvidence,
} from "../fixtures/davinci-dom-corpus-workflow.mjs";
import {
  artifactRoot,
  captureCorpus,
  corpusPlan,
  observers,
  sameCorpus,
  sha256,
  validateFiles,
} from "./canonical-corpus-identity.mjs";

export const observerLogs = { dom: "dom-corpus.log", "ssr-pug": "ssr-pug.log", reach: "reach.log" };
const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));
const writeJson = (path, value) => writeFileSync(path, `${JSON.stringify(value)}\n`);
const shapes = ["dom_inline", "dom_module", "ssr", "vapor"];
const ansiSequence = new RegExp(`${String.fromCharCode(27)}\\[[0-?]*[ -/]*[@-~]`, "g");

function single(lines, prefix) {
  const found = lines.filter((line) => line.startsWith(prefix));
  assert.equal(found.length, 1, `Missing or duplicate observer counter: ${prefix}`);
  return found[0];
}

function counter(line, name) {
  const match = new RegExp(`(?:^| )${name}=(\\d+)(?: |$)`).exec(line);
  assert(match, `Missing observer counter ${name}`);
  return Number(match[1]);
}

export function validateObserverLog(observer, bytes, identity) {
  assert(observers.includes(observer), "Unknown canonical observer");
  const expectedFiles = identity.files;
  const expectedGitlinks = identity.gitlinks.length;
  assert(
    Number.isSafeInteger(expectedFiles) && expectedFiles > 0 && expectedGitlinks > 0,
    "Missing committed canonical log inventory",
  );
  const lines = bytes
    .toString("utf8")
    .replace(ansiSequence, "")
    .split(/\r?\n/)
    .map((line) => line.replace(/^\d{4}-\d\d-\d\dT[\d:.]+Z /, ""));
  const closure = lines.filter((line) => line.startsWith("davinci-differential corpus scope:"));
  assert.equal(closure.length, observer === "ssr-pug" ? 2 : 1, "Incomplete canonical scope");
  for (const line of closure) {
    assert(
      line.includes("scope=canonical closure_evidence=true"),
      "Observer used a partial or foreign corpus",
    );
    assert.equal(
      counter(line, "submodules"),
      expectedGitlinks,
      "Observer omitted committed gitlinks",
    );
  }
  const harness = lines.filter((line) => line.startsWith("test result:"));
  assert.equal(harness.length, observer === "ssr-pug" ? 2 : 1, "Incomplete observer harness");
  const minimumTests = observer === "dom" ? [4] : observer === "ssr-pug" ? [3, 1] : [17];
  for (const [index, line] of harness.entries()) {
    assert(
      /^test result: ok\. \d+ passed; 0 failed; 0 ignored; 0 measured;/.test(line),
      "Observer tests did not complete without skips",
    );
    assert(Number(/(\d+) passed/.exec(line)[1]) >= minimumTests[index], "Observer tests shrank");
  }
  if (observer === "dom") {
    const evidence = parseCorpusEvidence(lines.join("\n"));
    assert.equal(evidence.files, expectedFiles, "DOM did not sweep the whole corpus");
    assert.equal(evidence.parsed, expectedFiles, "DOM parse inventory changed");
    assert.equal(evidence.unreadable, 0, "Unreadable DOM inputs");
    assert(evidence.patchFactEntries > 0, "DOM proved no patch materialization");
    assert.equal(
      evidence.oldErrorSkips,
      expectedOldErrorSkips,
      "Original DOM skip allowlist changed",
    );
    assert.deepEqual(
      evidence.oldErrorReasons,
      expectedOldErrorReasons,
      "Original DOM reasons changed",
    );
    assert.equal(evidence.s2Refusals, 0, "DOM refused original inputs");
    assert.equal(evidence.divergences, 0, "DOM diverged");
  } else if (observer === "ssr-pug") {
    const emitter = single(lines, "davinci SSR corpus sweep: scope=canonical ");
    assert.equal(counter(emitter, "files"), expectedFiles, "SSR corpus shrank");
    assert.equal(counter(emitter, "parsed"), expectedFiles, "SSR parse inventory changed");
    const production = single(lines, "davinci SSR production sweep: scope=canonical ");
    for (const line of [emitter, production]) {
      assert(
        counter(line, "templates") > 0 && counter(line, "compared") > 0 && counter(line, "s4") > 0,
        "SSR observer proved no selected production comparisons",
      );
      assert.equal(counter(line, "rejected"), 0, "SSR rejected a plan");
      assert.equal(counter(line, "divergences"), 0, "SSR diverged");
    }
    const pug = single(lines, "davinci pug corpus sweep: ");
    assert.equal(counter(pug, "files"), expectedFiles, "Pug corpus shrank");
    assert(
      counter(pug, "pug") > 0 && counter(pug, "compiled_lanes") > 0,
      "Pug proved no compilation",
    );
    assert.equal(counter(pug, "refused"), 0, "Pug refused original inputs");
  } else {
    const summary = single(lines, "davinci production reach: scope=canonical ");
    assert.equal(counter(summary, "files"), expectedFiles, "Production reach corpus shrank");
    assert.equal(counter(summary, "unreadable"), 0, "Unreadable reach inputs");
    assert.equal(counter(summary, "parse_errors"), 0, "Reach parse failures");
    const canonical = lines.slice(lines.indexOf(summary));
    for (const shape of shapes) {
      const reach = single(canonical, `davinci production reach: shape=${shape} `);
      assert.equal(counter(reach, "rejected"), 0, "Production reach rejected a plan");
      assert.equal(counter(reach, "divergences"), 0, "Production reach diverged");
      const native = single(
        canonical,
        `davinci native-only acceptance: scope=canonical product=compiler target=${shape} `,
      );
      assert.equal(counter(native, "planned"), expectedFiles, "Native reach inventory changed");
      assert.equal(
        counter(native, "native_only") +
          counter(native, "legacy_backed") +
          counter(native, "unverified"),
        expectedFiles,
        "Native reach counters are incomplete",
      );
    }
  }
  return lines.filter(
    (line) => line.startsWith("davinci ") || line.startsWith("davinci-differential "),
  );
}

export function validateObserverArtifact(observer, directory) {
  const receipt = readJson(join(directory, "observer.json"));
  assert.equal(receipt.schema, "vize.canonical-corpus-observer", "Foreign observer receipt");
  assert.equal(receipt.version, 1, "Unknown observer receipt version");
  assert.equal(receipt.observer, observer, "Wrong canonical observer");
  assert.equal(receipt.outcome, "success", "Canonical observer did not succeed");
  const identity = readJson(join(directory, "identity.json"));
  sameCorpus(receipt.identity, identity);
  assert.equal(
    validateHydration(directory, identity),
    receipt.hydrationSha256,
    "Canonical hydration receipt was replaced",
  );
  const files = readJson(join(directory, "files.json"));
  const proof = readJson(join(directory, "committed-corpus.json"));
  assert.equal(
    sha256(JSON.stringify(proof)),
    identity.committedSha256,
    "Committed tree proof was replaced",
  );
  const committedFiles = verifyCommittedInventory(proof, identity.gitlinks);
  assert.equal(committedFiles.length, identity.files, "Committed corpus count changed");
  assert.equal(
    validateFiles(files, committedFiles),
    identity.filesSha256,
    "Canonical file manifest was replaced",
  );
  const bytes = readFileSync(join(directory, observerLogs[observer]));
  assert.equal(sha256(bytes), receipt.logSha256, "Observer log was replaced");
  assert.deepEqual(
    validateObserverLog(observer, bytes, identity),
    receipt.counters,
    "Observer counters changed",
  );
  assert.equal(
    sha256(readFileSync(join(directory, "selected-gitlinks.txt"))),
    receipt.selectedSha256,
    "Selected gitlink evidence was replaced",
  );
  assert.equal(
    sha256(readFileSync(join(directory, "submodule-status.txt"))),
    receipt.statusSha256,
    "Submodule status evidence was replaced",
  );
  assert.deepEqual(
    readFileSync(join(directory, "selected-gitlinks.txt"), "utf8").trim().split("\n"),
    identity.gitlinks.map((row) => row.path),
    "Selected gitlink evidence is incomplete",
  );
  const status = readFileSync(join(directory, "submodule-status.txt"), "utf8")
    .trimEnd()
    .split("\n");
  assert.equal(status.length, identity.gitlinks.length, "Submodule status evidence is incomplete");
  for (const [index, line] of status.entries()) {
    const prefix = ` ${identity.gitlinks[index].sha} ${identity.gitlinks[index].path}`;
    assert(
      line.startsWith(prefix) && /^(?: \([^\r\n]*\))?$/.test(line.slice(prefix.length)),
      "Submodule status differs from the committed gitlink",
    );
  }
  if (observer === "dom")
    assert.deepEqual(
      validateCorpusEvidence(directory).failures,
      [],
      "Original DOM evidence gate failed",
    );
  return receipt;
}

export function recordObserver(observer, outcome, cwd, env = process.env) {
  assert(observers.includes(observer), "Unknown canonical observer");
  assert.equal(outcome, "success", "Required canonical observer did not succeed");
  const directory = join(cwd, artifactRoot);
  const identity = readJson(join(directory, "identity.json"));
  sameCorpus(identity, captureCorpus(cwd, env).identity);
  const bytes = readFileSync(join(directory, observerLogs[observer]));
  writeJson(join(directory, "observer.json"), {
    schema: "vize.canonical-corpus-observer",
    version: 1,
    observer,
    outcome,
    identity,
    hydrationSha256: validateHydration(directory, identity),
    logSha256: sha256(bytes),
    selectedSha256: sha256(readFileSync(join(directory, "selected-gitlinks.txt"))),
    statusSha256: sha256(readFileSync(join(directory, "submodule-status.txt"))),
    counters: validateObserverLog(observer, bytes, identity),
  });
  return validateObserverArtifact(observer, directory);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, observer] = process.argv.slice(2);
  const cwd = process.cwd();
  mkdirSync(artifactRoot, { recursive: true });
  if (mode === "plan") {
    const plan = corpusPlan(cwd);
    assert(process.env.GITHUB_OUTPUT, "Missing canonical plan output");
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `cache-key=${plan.cacheKey}\ncache-trusted=${plan.cacheTrusted}\n`,
    );
  } else if (mode === "rehydrate") {
    recoverFixtureCheckout(cwd, corpusPlan(cwd));
  } else if (mode === "snapshot") {
    const capture = captureCorpus(cwd);
    writeJson(join(artifactRoot, "identity.json"), capture.identity);
    writeJson(join(artifactRoot, "files.json"), capture.files);
    writeJson(join(artifactRoot, "committed-corpus.json"), capture.proof);
  } else if (mode === "record") {
    recordObserver(observer, process.env.OBSERVER_OUTCOME, cwd);
  } else {
    throw new Error("Expected canonical plan|rehydrate|snapshot|record OBSERVER");
  }
}
