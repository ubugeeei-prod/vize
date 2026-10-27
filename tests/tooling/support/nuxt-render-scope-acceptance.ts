import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../../differential/build-receipt.ts";
import {
  copyGenuineNuxtWithNestedKit,
  observeRuntime,
  requireSupported,
  sha256,
  writeInputs,
  verifyRegistryAuthority,
  readManifest,
} from "./nuxt-render-scope-runtime.ts";
import {
  body,
  duplicates,
  exact,
  profile,
  run,
  scopeCounters,
  valid,
  withoutOriginalTwoGroups,
} from "./nuxt-render-scope-packets.ts";
import type { ProcessPacket, Profile } from "./nuxt-render-scope-packets.ts";

type CorpusCase = {
  id: string;
  role: string;
  files: Record<string, string>;
  expectedDuplicateIds: string[];
};
type Spec = CorpusCase & {
  exclusive?: boolean;
  scopeExpected?: boolean;
  patterns?: string[];
  setup?:
    | "ignored"
    | "project-rc"
    | "ancestor-rc"
    | "xdg-rc"
    | "env"
    | "node-env"
    | "decoy-kit"
    | "nested-kit";
};
type CaseReport = {
  id: string;
  role: string;
  exclusive: boolean;
  cwd: string;
  inputs: { path: string; source: string; sha256: string }[];
  environment: { home: string | undefined; xdg: string; removedKeys: string[]; override?: string };
  runtime?: ReturnType<typeof observeRuntime>;
  ambient?: { path: string; source: string; sha256: string };
  setup?: ProcessPacket;
  processes: Record<string, ProcessPacket>;
  sourceProfile?: Profile;
  beforeProfile?: Profile;
  counters?: Record<string, number>;
  after?: { path: string; sha256: string }[];
  matched: boolean;
  error?: string;
};
const argv = process.argv.slice(2);
function option(name: string): string {
  const index = argv.indexOf(name);
  const value = index >= 0 ? argv[index + 1] : undefined;
  assert(value, "Missing " + name);
  return path.resolve(value);
}
const root = path.resolve(fileURLToPath(new URL("../../../", import.meta.url)));
const runtime = option("--runtime");
const beforeFile = option("--before");
const output = option("--output");
const beforeBytes = fs.readFileSync(beforeFile);
const before = JSON.parse(beforeBytes.toString("utf8")) as {
  qualified: false;
  failures: unknown[];
  provenance: unknown;
  cases: { on: ProcessPacket }[];
};
assert.equal(before.qualified, false);
assert.deepEqual(before.failures, []);
assert.equal(before.cases.length, 13);
const publicCli = before.cases[0].on.command;
const identity = expectedBuildIdentity(root);
const receipt = JSON.parse(
  fs.readFileSync(path.join(root, identity.binaryPath + ".differential-build.json"), "utf8"),
);
validateBuildReceipt(receipt, identity);
const sourceCli = path.join(root, identity.binaryPath);
assert.match(process.env.VIZE_NUXT_SCOPE_AUTHORED_HEAD ?? "", /^[a-f0-9]{40}$/);
const corpusBytes = fs.readFileSync(
  new URL("../../_fixtures/cross-file/nuxt-exclusive-id-10919.json", import.meta.url),
);
assert.equal(
  sha256(corpusBytes),
  "2681f03c914880af56bdd419e326cf8867ab4f2909241272153a51fd6478cb32",
);
const corpus = JSON.parse(corpusBytes.toString("utf8")) as {
  originalFiles: Record<string, string>;
  cases: CorpusCase[];
};
fs.mkdirSync(output, { recursive: true });
const runtimePackages = observeRuntime(runtime);
requireSupported(runtimePackages);
const unsupportedKit = fs.realpathSync(path.join(runtime, "node_modules/nuxt-kit-unsupported"));
const unsupportedManifest = JSON.parse(
  fs.readFileSync(path.join(unsupportedKit, "package.json"), "utf8"),
) as { name: string; version: string };
assert.deepEqual(unsupportedManifest.name, "@nuxt/kit");
assert.equal(unsupportedManifest.version, "4.6.2");
const installLock = fs.readFileSync(path.join(runtime, "package-lock.json"));
const aliasPackage = readManifest(
  path.join(unsupportedKit, "package.json"),
  path.join(runtime, "package.json"),
);
const registryAuthority = verifyRegistryAuthority(
  runtime,
  path.join(path.dirname(output), "nuxt-runtime"),
  [...runtimePackages, aliasPackage],
);

const report = {
  schema: "vize.nuxt-render.scope-source-acceptance.v1",
  role: "actual supported Nuxt source candidate; independent public release acceptance remains required",
  qualified: false,
  publicFixAccepted: false,
  sourceQualificationPassed: false,
  authoredHead: process.env.VIZE_NUXT_SCOPE_AUTHORED_HEAD,
  integrationSource: identity,
  receipt,
  originalBefore: { sha256: sha256(beforeBytes), provenance: before.provenance },
  originalCorpusSha256: sha256(corpusBytes),
  node: process.version,
  platform: process.platform,
  runtime: {
    packages: runtimePackages,
    lockSha256: sha256(installLock),
    registryAuthority,
    unsupportedKit: unsupportedManifest,
  },
  cases: [] as { id: string; file: string; sha256: string; matched: boolean; error?: string }[],
  failures: [] as { id: string; message: string }[],
};
function persist(): void {
  fs.writeFileSync(path.join(output, "index.json"), JSON.stringify(report, null, 2) + "\n");
}
const overview =
  "<script setup>import Login from './login.vue';import Signup from './signup.vue'</script><template><Login/><Signup/></template>\n";
const originals = corpus.originalFiles;
function changed(id: string, files: Record<string, string>, setup?: Spec["setup"]): Spec {
  return {
    id,
    role: "unknown-or-co-render-must-retain",
    files: { ...originals, ...files },
    expectedDuplicateIds: ["shell", "auth-page"],
    setup,
  };
}
const specs: Spec[] = [
  ...corpus.cases.map((entry, index) => ({
    ...entry,
    exclusive: index === 0,
    scopeExpected: index === 0 || entry.id === "layout-and-page",
  })),
  changed("unselected-co-rendering-importer", { "src/pages/overview.vue": overview }),
  changed(
    "ignored-co-rendering-importer",
    { "src/pages/overview.vue": overview, ".gitignore": "src/pages/overview.vue\n" },
    "ignored",
  ),
  changed("page-transition", {
    "src/pages/login.vue":
      "<script setup>definePageMeta({layout:'bare',pageTransition:{mode:'in-out'}})</script><template><section id=\"auth-page\">login</section></template>\n",
  }),
  changed("repeated-layout-slot", {
    "src/layouts/bare.vue": '<template><div id="shell"><slot/><slot/></div></template>\n',
  }),
  changed("looped-layout-slot", {
    "src/layouts/bare.vue":
      '<template><div id="shell"><template v-for="item in [1,2]"><slot/></template></div></template>\n',
  }),
  changed("shadowed-framework-component", {
    "src/components/NuxtPage.vue": "<template><slot/><slot/></template>\n",
  }),
  changed("inline-event-carrier", {
    "src/layouts/bare.vue":
      '<template><div id="shell" onclick="document.body.appendChild(this.cloneNode(true))"><slot/></div></template>\n',
  }),
  ...(["project-rc", "ancestor-rc", "xdg-rc", "env", "node-env", "nested-kit"] as const).map(
    (setup) => changed(setup, {}, setup),
  ),
  { ...changed("top-level-kit-decoy", {}, "decoy-kit"), exclusive: true },
];
specs.find((entry) => entry.id === "unselected-co-rendering-importer")!.patterns = [
  "src/app.vue",
  "src/layouts/bare.vue",
  "src/layouts/default.vue",
  "src/pages/login.vue",
  "src/pages/signup.vue",
];
const removedKeys = Object.keys(process.env).filter(
  (key) =>
    key.startsWith("NUXT_") ||
    key.startsWith("C12_") ||
    key.startsWith("DOTENV_") ||
    key.startsWith("JITI_") ||
    [
      "NODE_OPTIONS",
      "NODE_PATH",
      "NODE_PRESERVE_SYMLINKS",
      "NODE_PRESERVE_SYMLINKS_MAIN",
      "FORCE_COLOR",
    ].includes(key),
);
for (const spec of specs) {
  const cwd = path.join(runtime, "cases", spec.id);
  const xdg = path.join(runtime, "controlled-xdg", spec.id);
  fs.mkdirSync(xdg, { recursive: true });
  const inputFiles = {
    ...spec.files,
    "package.json":
      JSON.stringify({ name: "nuxt-scope-" + spec.id, version: "0.0.0", type: "module" }) + "\n",
  };
  writeInputs(cwd, inputFiles);
  const env: NodeJS.ProcessEnv = { ...process.env, NO_COLOR: "1", VIZE_LOG: "warn" };
  for (const key of removedKeys) delete env[key];
  env.XDG_CONFIG_HOME = xdg;
  env.NUXT_TELEMETRY_DISABLED = "1";
  const row: CaseReport = {
    id: spec.id,
    role: spec.role,
    exclusive: !!spec.exclusive,
    cwd,
    inputs: Object.entries(inputFiles).map(([name, source]) => ({
      path: name,
      source,
      sha256: sha256(source),
    })),
    environment: { home: env.HOME, xdg, removedKeys },
    processes: {},
    matched: false,
  };
  const packetFile = path.join(output, spec.id + ".json");
  const saveCase = (): void => fs.writeFileSync(packetFile, JSON.stringify(row, null, 2) + "\n");
  let ambientPath: string | undefined;
  try {
    if (spec.setup === "ignored") {
      row.setup = run("git", ["init", "--quiet"], cwd, env);
      assert.equal(row.setup.status, 0);
    }
    if (spec.setup === "project-rc" || spec.setup === "ancestor-rc" || spec.setup === "xdg-rc") {
      ambientPath =
        spec.setup === "project-rc"
          ? path.join(cwd, ".nuxtrc")
          : spec.setup === "ancestor-rc"
            ? path.join(path.dirname(cwd), ".nuxtrc")
            : path.join(xdg, ".nuxtrc");
      assert(!fs.existsSync(ambientPath), "Never overwrite an existing ambient input");
      const source = 'app.pageTransition={"mode":"in-out"}\n';
      fs.writeFileSync(ambientPath, source);
      row.ambient = { path: ambientPath, source, sha256: sha256(source) };
    }
    if (spec.setup === "env" || spec.setup === "node-env") {
      const key = spec.setup === "env" ? "NUXT_APP_BASE_URL" : "NODE_PRESERVE_SYMLINKS";
      env[key] = "1";
      row.environment.override = key;
    }
    if (spec.setup === "decoy-kit") {
      const target = path.join(cwd, "node_modules/@nuxt/kit");
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.symlinkSync(unsupportedKit, target, "dir");
    }
    if (spec.setup === "nested-kit")
      copyGenuineNuxtWithNestedKit(cwd, runtimePackages, unsupportedKit);
    row.runtime = observeRuntime(cwd);
    if (spec.setup === "nested-kit") {
      assert.equal(row.runtime[1].version, "4.6.2");
    } else {
      requireSupported(row.runtime);
    }
    const patterns = spec.patterns ?? ["src/**/*.vue"];
    const flags = [
      "lint",
      "--no-config",
      "--locale",
      "en",
      "--help-level",
      "none",
      "--format",
      "json",
    ];
    const on = [...flags, "--cross-file", ...patterns];
    const off = [...flags, ...patterns];
    const plain = ["lint", "-f", "plain", "--no-config", "--cross-file", ...patterns];
    const plainOff = ["lint", "-f", "plain", "--no-config", ...patterns];
    const sourceProfilePath = path.join(output, spec.id + ".profile.json");
    const beforeProfilePath = path.join(output, spec.id + ".before-profile.json");
    // Capture every subprocess and whole doctor packet before evaluating the oracle.
    row.processes.beforeOn = run(publicCli, [...on, "--profile-json", beforeProfilePath], cwd, env);
    row.processes.sourceOn = run(sourceCli, [...on, "--profile-json", sourceProfilePath], cwd, env);
    row.processes.beforeOff = run(publicCli, off, cwd, env);
    row.processes.sourceOff = run(sourceCli, off, cwd, env);
    row.processes.beforePlain = run(publicCli, plain, cwd, env);
    row.processes.sourcePlain = run(sourceCli, plain, cwd, env);
    if (spec.exclusive) row.processes.beforePlainOff = run(publicCli, plainOff, cwd, env);
    if (spec.exclusive)
      row.processes.genuineNuxtConfig = run(
        process.execPath,
        [
          fileURLToPath(new URL("./nuxt-render-scope-runtime.ts", import.meta.url)),
          "--probe-effective-config",
          cwd,
        ],
        cwd,
        env,
      );
    const doctor = ["doctor", "src", "--format", "json", "--exit-zero"];
    row.processes.beforeDoctor = run(publicCli, doctor, cwd, env);
    row.processes.sourceDoctor = run(sourceCli, doctor, cwd, env);
    row.after = row.inputs.map((input) => ({
      path: input.path,
      sha256: sha256(fs.readFileSync(path.join(cwd, input.path))),
    }));
    saveCase();
    for (const packet of Object.values(row.processes)) valid(packet);
    const beforeBody = body(row.processes.beforeOn);
    const sourceBody = body(row.processes.sourceOn);
    for (const id of spec.expectedDuplicateIds)
      assert(
        duplicates(beforeBody).some((message) =>
          message.message.includes("Element ID '" + id + "'"),
        ),
        spec.id + "/public warning " + id,
      );
    exact(row.processes.sourceOff, row.processes.beforeOff);
    if (spec.exclusive) {
      assert.equal(
        row.processes.genuineNuxtConfig.status,
        0,
        row.processes.genuineNuxtConfig.stderr,
      );
      assert.equal(
        JSON.parse(row.processes.genuineNuxtConfig.stdout).schema,
        "vize.nuxt-render.genuine-effective-config.v1",
      );
      const expected = withoutOriginalTwoGroups(beforeBody);
      assert.deepEqual(sourceBody, expected, "Only the two original groups may change");
      assert.deepEqual(
        expected,
        body(row.processes.beforeOff),
        "No unrelated CF diagnostics in this supported positive",
      );
      exact(row.processes.sourcePlain, row.processes.beforePlainOff!);
      assert.equal(row.processes.sourceOn.status, row.processes.beforeOff.status);
      assert.equal(row.processes.sourceOn.stderr, row.processes.beforeOn.stderr);
    } else {
      exact(row.processes.sourceOn, row.processes.beforeOn);
      exact(row.processes.sourcePlain, row.processes.beforePlain);
    }
    assert.equal(row.processes.beforeDoctor.status, 0);
    assert.equal(row.processes.sourceDoctor.status, 0);
    assert.deepEqual(
      JSON.parse(row.processes.sourceDoctor.stdout),
      JSON.parse(row.processes.beforeDoctor.stdout),
    );
    assert.equal(row.processes.sourceDoctor.stderr, row.processes.beforeDoctor.stderr);
    if (spec.id.includes("co-rendering-importer")) {
      assert(
        !sourceBody.some((file) => file.file.endsWith("overview.vue")),
        "The importer must actually be omitted",
      );
    }
    row.sourceProfile = profile(sourceProfilePath);
    row.beforeProfile = profile(beforeProfilePath);
    for (const key of [
      "cli.lint.cross_file.sfc.effect_script_parse_inputs",
      "croquis_cf.dependencies.finalize.calls",
      "croquis_cf.dependencies.finalize.files",
    ]) {
      const original = row.beforeProfile.counters.find((counter) => counter.key === key);
      const candidate = row.sourceProfile.counters.find((counter) => counter.key === key);
      assert(original && candidate, key + "/actual counter must be present");
      assert.deepEqual(candidate, original, key + "/unchanged existing source-operation count");
    }
    row.counters = scopeCounters(row.sourceProfile);
    if (spec.id === "non-nuxt-lookalike-directories") {
      assert.equal(row.counters.invocations, undefined);
    } else {
      assert.equal(row.counters.invocations, 1);
      assert.equal(row.counters.proven, (spec.scopeExpected ?? spec.exclusive) ? 1 : 0);
      assert(row.counters.read_dir_requests <= 4);
      assert(row.counters.package_lookup_requests <= 5 * 64);
      assert(row.counters.metadata_requests <= 512);
      assert(row.counters.manifest_read_requests <= 6);
      assert(row.counters.manifest_bytes <= row.counters.manifest_read_requests * 64 * 1024);
      if (spec.exclusive) {
        const manifests = new Map(row.runtime.map((pkg) => [pkg.manifest, pkg.bytes]));
        const projectBytes = fs.readFileSync(path.join(cwd, "package.json")).length;
        assert.equal(row.counters.manifest_read_requests, manifests.size + 1);
        assert.equal(row.counters.manifest_json_parse_requests, manifests.size + 1);
        assert.equal(
          row.counters.manifest_bytes,
          [...manifests.values()].reduce((a, b) => a + b, projectBytes),
        );
        assert.equal(row.counters.read_dir_requests, 4);
        const entries = ["", "src", "src/pages", "src/layouts"].reduce(
          (sum, directory) => sum + fs.readdirSync(path.join(cwd, directory)).length,
          0,
        );
        assert.equal(row.counters.directory_entries, entries);
        assert.equal(row.counters.file_type_requests, entries);
        for (const [name, original] of Object.entries(originals)) {
          assert.equal(sha256(fs.readFileSync(path.join(cwd, name))), sha256(original));
        }
      }
    }
    assert.deepEqual(
      row.after,
      row.inputs.map(({ path, sha256 }) => ({ path, sha256 })),
    );
    row.matched = true;
  } catch (error) {
    row.error = error instanceof Error ? error.message : String(error);
    report.failures.push({ id: spec.id, message: row.error });
  } finally {
    if (ambientPath) fs.rmSync(ambientPath);
    saveCase();
    report.cases.push({
      id: spec.id,
      file: path.basename(packetFile),
      sha256: sha256(fs.readFileSync(packetFile)),
      matched: row.matched,
      error: row.error,
    });
    persist();
  }
}
report.sourceQualificationPassed = report.failures.length === 0;
persist();
console.log(
  JSON.stringify({
    qualified: false,
    sourceQualificationPassed: report.sourceQualificationPassed,
    cases: report.cases.length,
    failures: report.failures,
  }),
);
if (!report.sourceQualificationPassed) process.exitCode = 1;
