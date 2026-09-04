import { fileURLToPath } from "node:url";

const fixture = (id) => `tests/_fixtures/_git/${id}`;

const BLACKSMITH_RUNNER = "blacksmith-32vcpu-ubuntu-2404";

function row(profile, suite, shard, task, fixtureIds, needsPlaywright, timeout, runner) {
  return {
    profile,
    suite,
    shard,
    task,
    fixtures: fixtureIds.map(fixture),
    needsPlaywright,
    timeout,
    runner: runner ?? BLACKSMITH_RUNNER,
    cacheKey: `app-e2e-${profile}-${suite}-${shard}`,
    worktreeId: `ci-app-e2e-${profile}-${suite}-${shard}`,
    artifactStem: `${profile}-${suite}-${shard}`,
  };
}

const checkFixtures = [
  "ant-design-vue",
  "directus",
  "element-plus",
  "elk",
  "frontend-phpcon-do-website",
  "hoppscotch",
  "misskey",
  "naive-ui",
  "npmx.dev",
  "nuxt-ui",
  "primevue",
  "reka-ui",
  "voicevox",
  "vue-vben-admin",
  "vuefes-2025",
  "vuetify",
];
const lintFixtures = checkFixtures.filter((id) => id !== "frontend-phpcon-do-website");
const readinessFixtures = ["elk", "misskey", "npmx.dev", "nuxt-ui", "reka-ui"];
const fullVrtTimeout = "15m";

export const fullAppE2eRows = [
  row("full", "dev", "elk", "test:dev:elk", ["elk"], true, "12m"),
  row("full", "dev", "misskey", "test:dev:misskey", ["misskey"], true, "12m"),
  row("full", "dev", "npmx", "test:dev:npmx", ["npmx.dev"], true, "12m"),
  row("full", "dev", "nuxt-ui", "test:dev:nuxt-ui", ["nuxt-ui"], true, "15m"),
  row("full", "dev", "vuefes", "test:dev:vuefes", ["vuefes-2025"], true, "12m"),
  row("full", "vrt", "elk", "test:vrt:elk", ["elk"], true, fullVrtTimeout),
  row(
    "full",
    "vrt",
    "frontend-phpcon",
    "test:vrt:frontend-phpcon",
    ["frontend-phpcon-do-website"],
    true,
    fullVrtTimeout,
  ),
  row("full", "vrt", "misskey", "test:vrt:misskey", ["misskey"], true, "20m"),
  row("full", "vrt", "npmx", "test:vrt:npmx", ["npmx.dev"], true, fullVrtTimeout),
  row("full", "vrt", "vuefes", "test:vrt:vuefes", ["vuefes-2025"], true, fullVrtTimeout),
  row("full", "preview", "elk", "test:preview:elk", ["elk"], false, "10m"),
  row("full", "preview", "misskey", "test:preview:misskey", ["misskey"], false, "10m"),
  row("full", "preview", "npmx", "test:preview:npmx", ["npmx.dev"], false, "10m"),
  row("full", "preview", "vuefes", "test:preview:vuefes", ["vuefes-2025"], false, "10m"),
  row(
    "full",
    "build",
    "all",
    "test:build",
    ["elk", "misskey", "npmx.dev", "vuefes-2025"],
    false,
    "10m",
  ),
  row("full", "check", "all", "test:check", checkFixtures, false, "75m"),
  row("full", "lint", "all", "test:lint", lintFixtures, false, "10m"),
];

export const readinessRows = [
  row("readiness", "readiness", "check", "test:readiness:check", readinessFixtures, false, "25m"),
  row(
    "readiness",
    "readiness",
    "check-vuefes",
    "test:readiness:check:vuefes",
    ["vuefes-2025"],
    false,
    "2m",
  ),
  row("readiness", "readiness", "lint", "test:readiness:lint", readinessFixtures, false, "20m"),
  row("readiness", "readiness", "build", "test:readiness:build", ["elk"], false, "3m"),
  row(
    "readiness",
    "readiness",
    "dev-misskey",
    "test:readiness:dev:misskey",
    ["misskey"],
    true,
    "8m",
  ),
  row(
    "readiness",
    "readiness",
    "dev-nuxt-ui",
    "test:readiness:dev:nuxt-ui",
    ["nuxt-ui"],
    true,
    "8m",
  ),
];

const profiles = new Map([
  ["full", fullAppE2eRows],
  ["readiness", readinessRows],
]);
const fullSuites = ["dev", "vrt", "preview", "check", "lint", "build", "all"];

export function validateAppE2eRows(rows) {
  if (!Array.isArray(rows) || rows.length === 0) throw new Error("App E2E plan must not be empty");
  const uniqueFields = ["identity", "task", "cacheKey", "worktreeId", "artifactStem"];
  const seen = new Map(uniqueFields.map((field) => [field, new Set()]));
  for (const current of rows) {
    if (current == null || typeof current !== "object")
      throw new Error("Plan rows must be objects");
    const identity = `${current.profile}:${current.suite}:${current.shard}`;
    const expectedPrefix = `${current.profile}-${current.suite}-${current.shard}`;
    if (!profiles.has(current.profile))
      throw new Error(`Unknown App E2E profile: ${current.profile}`);
    if (current.profile === "full" && !fullSuites.includes(current.suite)) {
      throw new Error(`Unknown full App E2E suite: ${current.suite}`);
    }
    if (current.profile === "readiness" && current.suite !== "readiness") {
      throw new Error(`Readiness row has invalid suite: ${current.suite}`);
    }
    if (!/^[a-z0-9][a-z0-9-]*$/.test(current.shard))
      throw new Error(`Invalid shard: ${current.shard}`);
    if (!/^test:[a-z0-9:-]+$/.test(current.task)) throw new Error(`Invalid task: ${current.task}`);
    if (!/^\d+m$/.test(current.timeout)) throw new Error(`Invalid timeout: ${current.timeout}`);
    if (typeof current.needsPlaywright !== "boolean") {
      throw new Error(`${identity} needsPlaywright must be boolean`);
    }
    const expectedBrowser =
      (current.profile === "full" && (current.suite === "dev" || current.suite === "vrt")) ||
      (current.profile === "readiness" && current.shard.startsWith("dev-"));
    if (current.needsPlaywright !== expectedBrowser) {
      throw new Error(`${identity} Playwright requirement drifted`);
    }
    if (current.runner !== BLACKSMITH_RUNNER) {
      throw new Error(`${identity} runner drifted: expected ${BLACKSMITH_RUNNER}`);
    }
    if (!Array.isArray(current.fixtures) || current.fixtures.length === 0) {
      throw new Error(`${identity} must hydrate at least one fixture`);
    }
    if (new Set(current.fixtures).size !== current.fixtures.length) {
      throw new Error(`${identity} repeats a fixture`);
    }
    if (current.fixtures.some((path) => !/^tests\/_fixtures\/_git\/[a-z0-9.-]+$/.test(path))) {
      throw new Error(`${identity} contains an invalid fixture path`);
    }
    if (
      current.cacheKey !== `app-e2e-${expectedPrefix}` ||
      current.artifactStem !== expectedPrefix
    ) {
      throw new Error(`${identity} mutable identity drifted`);
    }
    if (current.worktreeId !== `ci-app-e2e-${expectedPrefix}`) {
      throw new Error(`${identity} worktree drifted`);
    }
    for (const [field, value] of [
      ["identity", identity],
      ["task", current.task],
      ["cacheKey", current.cacheKey],
      ["worktreeId", current.worktreeId],
      ["artifactStem", current.artifactStem],
    ]) {
      if (seen.get(field).has(value)) throw new Error(`Duplicate ${field}: ${value}`);
      seen.get(field).add(value);
    }
  }
  return rows;
}

validateAppE2eRows([...fullAppE2eRows, ...readinessRows]);

export function planAppE2eRows(profile, suite = "all") {
  const rows = profiles.get(profile);
  if (rows == null) throw new Error(`Unknown App E2E profile: ${profile}`);
  if (profile === "readiness") {
    if (suite !== "all" && suite !== "readiness")
      throw new Error(`Unknown readiness suite: ${suite}`);
    return rows.map((current) => structuredClone(current));
  }
  if (!fullSuites.includes(suite)) throw new Error(`Unknown full App E2E suite: ${suite}`);
  const selected = suite === "all" ? rows : rows.filter((current) => current.suite === suite);
  if (selected.length === 0) throw new Error(`App E2E suite selected no rows: ${suite}`);
  return selected.map((current) => structuredClone(current));
}

export function findAppE2eRow(profile, suite, shard) {
  const matches = planAppE2eRows(profile, profile === "readiness" ? "all" : suite).filter(
    (current) => current.suite === suite && current.shard === shard,
  );
  if (matches.length !== 1) throw new Error(`Unknown App E2E row: ${profile}:${suite}:${shard}`);
  return matches[0];
}

export function validateAppE2eTarget(suite, targetSha, runHeadSha) {
  if (targetSha === "") {
    if (suite === "all") throw new Error("target_sha is required when suite=all");
    return null;
  }
  if (!/^[0-9a-f]{40}$/.test(targetSha)) {
    throw new Error("target_sha must be a full lowercase 40-character commit SHA");
  }
  if (runHeadSha !== targetSha) {
    throw new Error(`dispatch ref must resolve to target_sha ${targetSha}; got ${runHeadSha}`);
  }
  return targetSha;
}

export function createAppE2ePlanEvidence(profile, suite, targetSha, sourceHeadSha = null) {
  if (!/^[0-9a-f]{40}$/.test(targetSha))
    throw new Error("Plan evidence requires an exact target SHA");
  if (sourceHeadSha != null && !/^[0-9a-f]{40}$/.test(sourceHeadSha)) {
    throw new Error("Plan evidence source head must be an exact SHA");
  }
  const rows = planAppE2eRows(profile, suite);
  return {
    schema: "vize.appE2ePlanEvidence",
    version: 1,
    profile,
    suite,
    targetSha,
    sourceHeadSha,
    rowCount: rows.length,
    rows,
  };
}

function parseArgs(argv) {
  const args = {
    profile: null,
    suite: "all",
    shard: null,
    field: "matrix",
    targetSha: null,
    runHeadSha: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const value = () => {
      if (argv[index + 1] == null) throw new Error(`${argv[index]} requires a value`);
      return argv[++index];
    };
    if (argv[index] === "--profile") args.profile = value();
    else if (argv[index] === "--suite") args.suite = value();
    else if (argv[index] === "--shard") args.shard = value();
    else if (argv[index] === "--field") args.field = value();
    else if (argv[index] === "--validate-target") args.targetSha = value();
    else if (argv[index] === "--run-head-sha") args.runHeadSha = value();
    else throw new Error(`Unknown argument: ${argv[index]}`);
  }
  if (args.profile == null) throw new Error("--profile is required");
  return args;
}

function main(argv) {
  const args = parseArgs(argv);
  if (args.targetSha != null) {
    if (args.runHeadSha == null)
      throw new Error("--run-head-sha is required with --validate-target");
    validateAppE2eTarget(args.suite, args.targetSha, args.runHeadSha);
    return;
  }
  if (args.shard == null) {
    const rows = planAppE2eRows(args.profile, args.suite);
    if (args.field === "matrix") process.stdout.write(`${JSON.stringify({ include: rows })}\n`);
    else if (args.field === "count") process.stdout.write(`${rows.length}\n`);
    else if (args.field === "evidence") {
      process.stdout.write(
        `${JSON.stringify(
          createAppE2ePlanEvidence(
            args.profile,
            args.suite,
            process.env.E2E_TARGET_SHA ?? "",
            process.env.E2E_SOURCE_HEAD_SHA || null,
          ),
          null,
          2,
        )}\n`,
      );
    } else throw new Error("Matrix planning only supports matrix or count fields");
    return;
  }
  const current = findAppE2eRow(args.profile, args.suite, args.shard);
  const fields = {
    fixtures: current.fixtures.join("\n"),
    task: current.task,
    timeout: current.timeout,
    "needs-playwright": String(current.needsPlaywright),
    "cache-key": current.cacheKey,
    "worktree-id": current.worktreeId,
    "artifact-stem": current.artifactStem,
  };
  if (!(args.field in fields)) throw new Error(`Unknown row field: ${args.field}`);
  process.stdout.write(`${fields[args.field]}\n`);
}

if (process.argv[1] != null && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
