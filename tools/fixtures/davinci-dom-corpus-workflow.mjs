import { spawn, spawnSync } from "node:child_process";
import { createWriteStream, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { appendFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { expectedComparisonCount } from "../davinci/lib/corpus-baseline-artifact.mjs";
import { loadManifest } from "../davinci/lib/corpus-baseline-contract.mjs";

export const artifactDir = "real-project-davinci-dom-corpus";
export const corpusRoot = "tests/_fixtures/_git";
export const expectedGitlinks = 146;
export const expectedDomOutputComparisons = 144;

const ansiEscapePattern = new RegExp(`${String.fromCharCode(27)}\\[[0-?]*[ -/]*[@-~]`, "g");

function stripAnsi(value) {
  return value.replace(ansiEscapePattern, "");
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    ...options,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed with exit code ${result.status}`);
  }
  return result.stdout ?? "";
}

function errorMessage(error) {
  const message = error instanceof Error ? error.message : String(error);
  return message.replace(/\s+/g, " ").trim();
}

export function parseFixtureGitlinks(indexOutput) {
  return indexOutput
    .split(/\r?\n/)
    .flatMap((line) => {
      const match = /^160000 [0-9a-f]{40} 0\t(.+)$/.exec(line);
      return match ? [match[1]] : [];
    })
    .sort((left, right) => left.localeCompare(right));
}

function selectedGitlinks(runCommand) {
  return parseFixtureGitlinks(runCommand("git", ["ls-files", "--stage", "--", corpusRoot]));
}

export function verdictFor(outcome, mode) {
  return mode === "record-only" && outcome === "failure" ? "success" : outcome;
}

export function corpusEvidenceLines(logText) {
  return logText
    .split(/\r?\n/)
    .filter((line) =>
      /davinci-differential corpus scope|davinci DOM corpus sweep/.test(stripAnsi(line)),
    );
}

export function parseCorpusEvidence(logText) {
  const evidence = {
    canonicalScope: false,
    closureEvidence: false,
    submodules: 0,
    files: 0,
    unreadable: 0,
    parsed: 0,
    templates: 0,
    compared: 0,
    oldErrorSkips: 0,
    s2Refusals: 0,
    divergences: 0,
  };
  for (const line of corpusEvidenceLines(logText).map(stripAnsi)) {
    const scope = /scope=canonical closure_evidence=(true|false) submodules=(\d+)/.exec(line);
    if (scope) {
      evidence.canonicalScope = true;
      evidence.closureEvidence = scope[1] === "true";
      evidence.submodules = Number(scope[2]);
      continue;
    }
    const sweep =
      /files=(\d+) unreadable=(\d+) parsed=(\d+) templates=(\d+) compared=(\d+) old_error_skips=(\d+) s2_refusals=(\d+) divergences=(\d+)/.exec(
        line,
      );
    if (sweep) {
      evidence.files = Number(sweep[1]);
      evidence.unreadable = Number(sweep[2]);
      evidence.parsed = Number(sweep[3]);
      evidence.templates = Number(sweep[4]);
      evidence.compared = Number(sweep[5]);
      evidence.oldErrorSkips = Number(sweep[6]);
      evidence.s2Refusals = Number(sweep[7]);
      evidence.divergences = Number(sweep[8]);
    }
  }
  return evidence;
}

export function validateCorpusEvidence(artifact = artifactDir) {
  const manifestDomOutputComparisons = expectedComparisonCount(loadManifest(), ["compiler"]);
  const selected = readOptional(`${artifact}/selected-gitlinks.txt`)
    .trim()
    .split(/\r?\n/)
    .filter(Boolean);
  const status = readOptional(`${artifact}/submodule-status.txt`)
    .trim()
    .split(/\r?\n/)
    .filter(Boolean);
  const evidence = parseCorpusEvidence(readOptional(`${artifact}/dom-corpus.log`));
  const failures = [];
  if (manifestDomOutputComparisons !== expectedDomOutputComparisons) {
    failures.push(
      `manifest DOM-output comparisons ${manifestDomOutputComparisons} != ${expectedDomOutputComparisons}`,
    );
  }
  if (selected.length !== expectedGitlinks) {
    failures.push(`selected gitlinks ${selected.length} != ${expectedGitlinks}`);
  }
  if (status.length !== expectedGitlinks) {
    failures.push(`submodule status rows ${status.length} != ${expectedGitlinks}`);
  }
  if (!evidence.canonicalScope || !evidence.closureEvidence) {
    failures.push("corpus log is missing canonical closure evidence");
  }
  if (evidence.submodules !== expectedGitlinks) {
    failures.push(`corpus log submodules ${evidence.submodules} != ${expectedGitlinks}`);
  }
  if (evidence.files === 0 || evidence.templates === 0 || evidence.compared === 0) {
    failures.push("corpus log proves no DOM-output comparisons");
  }
  if (evidence.unreadable !== 0 || evidence.oldErrorSkips !== 0) {
    failures.push(
      `corpus log skipped inputs: unreadable=${evidence.unreadable} old_error_skips=${evidence.oldErrorSkips}`,
    );
  }
  if (evidence.s2Refusals !== 0 || evidence.divergences !== 0) {
    failures.push(
      `corpus log is not clean: s2_refusals=${evidence.s2Refusals} divergences=${evidence.divergences}`,
    );
  }
  return {
    manifestDomOutputComparisons,
    selectedGitlinks: selected.length,
    submoduleStatusRows: status.length,
    evidence,
    failures,
  };
}

function hydrateFixtureSerially(fixturePath, runCommand) {
  try {
    runCommand("git", [
      "submodule",
      "update",
      "--init",
      "--checkout",
      "--depth",
      "1",
      "--jobs",
      "1",
      "--",
      fixturePath,
    ]);
  } catch (error) {
    console.warn(
      `::warning title=Davinci corpus hydrate full fallback::${fixturePath}: ${errorMessage(
        error,
      )}`,
    );
    runCommand("git", [
      "submodule",
      "update",
      "--init",
      "--checkout",
      "--force",
      "--",
      fixturePath,
    ]);
  }
}

export function hydrateCorpus({ artifact = artifactDir, runCommand = run } = {}) {
  mkdirSync(artifact, { recursive: true });
  const fixturePaths = selectedGitlinks(runCommand);
  if (fixturePaths.length !== expectedGitlinks) {
    console.error(
      `::error title=Unexpected fixture gitlinks::expected ${expectedGitlinks}, got ${fixturePaths.length}`,
    );
    return 1;
  }
  writeFileSync(`${artifact}/selected-gitlinks.txt`, `${fixturePaths.join("\n")}\n`);
  try {
    runCommand("git", [
      "submodule",
      "update",
      "--init",
      "--checkout",
      "--depth",
      "1",
      "--jobs",
      "8",
      "--",
      ...fixturePaths,
    ]);
  } catch (error) {
    console.warn(`::warning title=Davinci corpus hydrate serial fallback::${errorMessage(error)}`);
    for (const fixturePath of fixturePaths) {
      hydrateFixtureSerially(fixturePath, runCommand);
    }
  }
  const status = runCommand("git", ["submodule", "status", "--", corpusRoot]);
  writeFileSync(`${artifact}/submodule-status.txt`, status);
  return 0;
}

export async function runCorpus() {
  mkdirSync(artifactDir, { recursive: true });
  const log = createWriteStream(`${artifactDir}/dom-corpus.log`, { flags: "w" });
  const child = spawn(
    "cargo",
    [
      "test",
      "-p",
      "vize_s1_to_s2",
      "--features",
      "davinci-differential",
      "--test",
      "davinci_dom_corpus",
      "--",
      "--nocapture",
    ],
    {
      env: {
        ...process.env,
        VIZE_DAVINCI_DIFFERENTIAL_CORPUS: corpusRoot,
      },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  child.stdout.on("data", (chunk) => {
    process.stdout.write(chunk);
    log.write(chunk);
  });
  child.stderr.on("data", (chunk) => {
    process.stderr.write(chunk);
    log.write(chunk);
  });
  return await new Promise((resolvePromise) => {
    child.on("close", (code) => {
      log.end();
      resolvePromise(code ?? 1);
    });
    child.on("error", (error) => {
      log.end();
      console.error(error instanceof Error ? error.message : String(error));
      resolvePromise(1);
    });
  });
}

export async function finalizeCorpus(environment = process.env) {
  mkdirSync(artifactDir, { recursive: true });
  const mode = environment.VIZE_DAVINCI_DOM_CORPUS_MODE ?? "enforce";
  const outcome = environment.VIZE_DAVINCI_DOM_CORPUS_OUTCOME ?? "failure";
  let verdict = verdictFor(outcome, mode);
  const validation = validateCorpusEvidence();
  if (outcome === "success" && validation.failures.length > 0) {
    verdict = "failure";
  }
  writeFileSync(
    `${artifactDir}/summary.json`,
    `${JSON.stringify({ mode, outcome, verdict, ...validation })}\n`,
  );
  await appendCorpusSummary(mode, outcome, verdict, environment.GITHUB_STEP_SUMMARY);
  if (verdict !== "success") {
    for (const failure of validation.failures) {
      console.error(`::error title=Invalid Davinci S2 DOM corpus evidence::${failure}`);
    }
    console.error(`::error title=Davinci S2 DOM corpus failed::mode=${mode} verdict=${verdict}`);
    return 1;
  }
  return 0;
}

async function appendCorpusSummary(mode, outcome, verdict, summaryPath) {
  if (!summaryPath) return;
  const validation = validateCorpusEvidence();
  const logText = readOptional(`${artifactDir}/dom-corpus.log`);
  const evidence = corpusEvidenceLines(logText);
  await appendFile(
    summaryPath,
    [
      "## Davinci S2 DOM Corpus",
      "",
      `- mode: \`${mode}\``,
      `- outcome: \`${outcome}\``,
      `- verdict: \`${verdict}\``,
      `- manifest DOM-output comparisons: \`${validation.manifestDomOutputComparisons}\``,
      `- gitlinks: \`${validation.selectedGitlinks}\``,
      `- submodule status rows: \`${validation.submoduleStatusRows}\``,
      `- compared templates: \`${validation.evidence.compared}\``,
      "",
      ...evidence,
      "",
    ].join("\n"),
  );
}

function readOptional(path) {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return "";
  }
}

async function main() {
  const command = process.argv[2];
  if (command === "hydrate") return hydrateCorpus();
  if (command === "run") return await runCorpus();
  if (command === "finalize") return await finalizeCorpus();
  console.error("usage: davinci-dom-corpus-workflow.mjs hydrate|run|finalize");
  return 1;
}

const isMain = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  process.exitCode = await main();
}
