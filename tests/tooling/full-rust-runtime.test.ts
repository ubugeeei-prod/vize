import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse } from "yaml";
import { provisionFullRustRuntime } from "../../tools/support/compat/github/full-rust-runtime.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const variables = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
];
const name = `@typescript/typescript-${process.platform}-${process.arch}`;
const version = "7.0.2";
const script = fileURLToPath(
  new URL("../../tools/support/compat/github/full-rust-runtime.ts", import.meta.url),
);
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

function fixture() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-full-rust-runtime-")));
  const environment: NodeJS.ProcessEnv & { GITHUB_ENV: string } = {
    ...process.env,
    GITHUB_ENV: path.join(root, "github-env"),
    GITHUB_SHA: "a".repeat(40),
    GITHUB_RUN_ID: "123",
    GITHUB_RUN_ATTEMPT: "2",
    VIZE_TEST_REQUIRE_TSGO: "1",
  };
  for (const key of variables) delete environment[key];
  delete environment.VIZE_TEST_DISABLE_TSGO;
  const directory = path.join(root, "node_modules", name);
  const executable = path.join(directory, "lib", process.platform === "win32" ? "tsc.exe" : "tsc");
  const receiptPath = path.join(root, "target/nextest/full/native-runtime-provider.json");
  fs.mkdirSync(path.dirname(executable), { recursive: true });
  fs.writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify({ optionalDependencies: { [name]: "catalog:corsa-runtime" } }),
  );
  fs.writeFileSync(
    path.join(root, "pnpm-workspace.yaml"),
    JSON.stringify({ catalogs: { "corsa-runtime": { [name]: version } } }),
  );
  fs.writeFileSync(
    path.join(root, "pnpm-lock.yaml"),
    '---\nlockfileVersion: "9.0"\nimporters: {".": {packageManagerDependencies: {pnpm: {version: "12.1.0"}}}}\n---\n' +
      JSON.stringify({
        catalogs: { "corsa-runtime": { [name]: { specifier: version, version } } },
        importers: {
          ".": {
            optionalDependencies: { [name]: { specifier: "catalog:corsa-runtime", version } },
          },
        },
      }),
  );
  const metadata = path.join(directory, "package.json");
  fs.writeFileSync(metadata, JSON.stringify({ name, version }));
  // Carrier control only: this executable is not native compiler qualification.
  fs.writeFileSync(
    executable,
    `#!/bin/sh\n[ "$#" = 1 ] && [ "$1" = --version ] || exit 9\nprintf 'Version ${version}\\n'\n`,
    { mode: 0o755 },
  );
  fs.writeFileSync(environment.GITHUB_ENV, "EXISTING=retained\n");
  return {
    root,
    environment,
    directory,
    executable,
    metadata,
    receiptPath,
    readReceipt: () => JSON.parse(fs.readFileSync(receiptPath, "utf8")) as Record<string, unknown>,
    cleanup: () => fs.rmSync(root, { recursive: true, force: true }),
  };
}

test("locked provider reaches an external test child with its actual image/version receipt", () => {
  const f = fixture();
  const parent = { ...process.env };
  try {
    const input = { ...f.environment };
    provisionFullRustRuntime(f.root, f.environment);
    assert.deepEqual(f.environment, input);
    assert.deepEqual({ ...process.env }, parent);
    const exported = fs.readFileSync(f.environment.GITHUB_ENV, "utf8");
    assert.equal(exported, `EXISTING=retained\nCORSA_PATH=${f.executable}\n`);
    const receipt = f.readReceipt();
    assert.equal(receipt.selection, "checkout-locked-platform-package");
    assert.equal(receipt.sourceSha, f.environment.GITHUB_SHA);
    assert.deepEqual(receipt.execution, { runId: "123", runAttempt: "2" });
    assert.deepEqual(receipt.package, {
      name,
      version,
      directory: f.directory,
      metadataSha256: digest(fs.readFileSync(f.metadata)),
    });
    assert.deepEqual(receipt.executable, {
      path: f.executable,
      sha256: digest(fs.readFileSync(f.executable)),
      bytes: fs.statSync(f.executable).size,
      version: {
        args: ["--version"],
        stdout: `Version ${version}\n`,
        stderr: "",
        exitCode: 0,
        signal: null,
      },
    });
    const external = fs.mkdtempSync(path.join(os.tmpdir(), "vize-full-rust-consumer-"));
    try {
      const child = spawnSync(
        process.execPath,
        [
          "-e",
          "process.stdout.write(JSON.stringify([process.env.CORSA_PATH, process.env.VIZE_TEST_REQUIRE_TSGO, process.env.VIZE_TEST_DISABLE_TSGO]))",
        ],
        {
          cwd: external,
          encoding: "utf8",
          env: {
            ...f.environment,
            CORSA_PATH: exported.trim().split("\n").at(-1)?.slice("CORSA_PATH=".length),
          },
        },
      );
      assert.equal(child.status, 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), [f.executable, "1", null]);
    } finally {
      fs.rmSync(external, { recursive: true, force: true });
    }
  } finally {
    f.cleanup();
  }
});

test("every nonempty explicit runtime choice, including invalid choices, is preserved without validation", () => {
  for (const key of variables) {
    const f = fixture();
    try {
      fs.rmSync(path.join(f.root, "node_modules"), { recursive: true });
      f.environment[key] = `/missing/${key}\nunchanged`;
      f.environment.UNRELATED = "keep";
      const before = { ...f.environment };
      provisionFullRustRuntime(f.root, f.environment);
      assert.deepEqual(f.environment, before);
      assert.equal(fs.readFileSync(f.environment.GITHUB_ENV, "utf8"), "EXISTING=retained\n");
      assert.equal(f.readReceipt().selection, "inherited-explicit-runtime");
      assert.deepEqual(f.readReceipt().overrides, { [key]: before[key] });
      assert.equal(f.readReceipt().executable, undefined);
    } finally {
      f.cleanup();
    }
  }
});

test("the root-selected package may use the ordinary linked package-store layout", () => {
  const f = fixture();
  try {
    const selected = path.join(f.root, "node_modules/.pnpm/owned-provider/node_modules", name);
    fs.mkdirSync(path.dirname(selected), { recursive: true });
    fs.renameSync(f.directory, selected);
    fs.symlinkSync(selected, f.directory);
    provisionFullRustRuntime(f.root, f.environment);
    assert.deepEqual(f.readReceipt().package, {
      name,
      version,
      directory: selected,
      metadataSha256: digest(fs.readFileSync(path.join(selected, "package.json"))),
    });
    assert.equal(
      fs.readFileSync(f.environment.GITHUB_ENV, "utf8"),
      `EXISTING=retained\nCORSA_PATH=${fs.realpathSync(f.executable)}\n`,
    );
  } finally {
    f.cleanup();
  }
});

test("the actual committed manifest, catalog and multi-document lock select the owned provider", () => {
  const f = fixture();
  try {
    for (const file of ["package.json", "pnpm-workspace.yaml", "pnpm-lock.yaml"])
      fs.writeFileSync(path.join(f.root, file), readRepoFile(file));
    provisionFullRustRuntime(f.root, f.environment);
    assert.equal(f.readReceipt().selection, "checkout-locked-platform-package");
  } finally {
    f.cleanup();
  }
});

test("multiple explicit choices retain precedence inputs; empty values permit the locked default", () => {
  for (const populated of [true, false]) {
    const f = fixture();
    try {
      for (const key of variables) f.environment[key] = populated ? `/chosen/${key}` : "";
      const before = { ...f.environment };
      provisionFullRustRuntime(f.root, f.environment);
      assert.deepEqual(f.environment, before);
      if (populated)
        assert.deepEqual(
          f.readReceipt().overrides,
          Object.fromEntries(variables.map((key) => [key, before[key]])),
        );
      else assert.equal(f.readReceipt().selection, "checkout-locked-platform-package");
    } finally {
      f.cleanup();
    }
  }
});

test("missing, foreign, nested, mismatched and nonexecutable providers fail without any fallback export", () => {
  const mutations: Array<(f: ReturnType<typeof fixture>) => void> = [
    (f) => fs.rmSync(f.directory, { recursive: true }),
    (f) => fs.writeFileSync(f.metadata, JSON.stringify({ name: "foreign", version })),
    (f) => fs.writeFileSync(f.metadata, JSON.stringify({ name, version: "7.1.0-beta" })),
    (f) =>
      fs.writeFileSync(
        path.join(f.root, "package.json"),
        JSON.stringify({ optionalDependencies: { [name]: "7.0.2" } }),
      ),
    (f) =>
      fs.writeFileSync(
        path.join(f.root, "pnpm-workspace.yaml"),
        JSON.stringify({ catalogs: { "corsa-runtime": { [name]: "^7.0.2" } } }),
      ),
    (f) =>
      fs.writeFileSync(
        path.join(f.root, "pnpm-lock.yaml"),
        JSON.stringify({
          catalogs: { "corsa-runtime": { [name]: { specifier: version, version: "7.1.0-beta" } } },
        }),
      ),
    (f) => {
      const file = path.join(f.root, "pnpm-lock.yaml");
      const project = fs.readFileSync(file, "utf8").split("---\n").at(-1);
      fs.appendFileSync(file, `\n---\n${project}`);
    },
    (f) => fs.appendFileSync(path.join(f.root, "pnpm-lock.yaml"), "\n---\nbroken: [\n"),
    (f) =>
      fs.writeFileSync(
        path.join(f.root, "pnpm-lock.yaml"),
        JSON.stringify({
          catalogs: { "corsa-runtime": { [name]: { specifier: version, version } } },
          importers: {
            ".": {
              optionalDependencies: {
                [name]: { specifier: "catalog:corsa-runtime", version: "7.1.0-beta" },
              },
            },
          },
        }),
      ),
    (f) => {
      fs.rmSync(f.executable);
      fs.mkdirSync(f.executable);
    },
    (f) => fs.chmodSync(f.executable, 0o644),
    (f) => fs.writeFileSync(f.executable, "#!/bin/sh\nprintf 'Version 7.1.0-beta\\n'\n"),
    (f) => fs.writeFileSync(f.executable, "#!/bin/sh\nexit 7\n"),
    (f) => fs.writeFileSync(f.executable, `#!/bin/sh\nprintf 'Version ${version} \\n'\n`),
    (f) => fs.writeFileSync(f.executable, `#!/bin/sh\nprintf '\\nVersion ${version}\\n'\n`),
    (f) =>
      fs.writeFileSync(
        f.executable,
        `#!/bin/sh\nprintf 'Version ${version}\\n'\nprintf 'warning\\n' >&2\n`,
      ),
    (f) => fs.writeFileSync(f.executable, "#!/bin/sh\nkill -TERM $$\n"),
    (f) =>
      fs.writeFileSync(
        f.executable,
        `#!/bin/sh\nprintf 'Version ${version}\\n'\nprintf '# changed\\n' >> "$0"\n`,
      ),
    (f) => {
      const nested = path.join(f.root, "tests/node_modules", name);
      fs.mkdirSync(path.dirname(nested), { recursive: true });
      fs.renameSync(f.directory, nested);
    },
    (f) => {
      const foreign = path.join(f.root, "foreign");
      fs.copyFileSync(f.executable, foreign);
      fs.rmSync(f.executable);
      fs.symlinkSync(foreign, f.executable);
    },
  ];
  for (const [index, mutate] of mutations.entries()) {
    const f = fixture();
    try {
      const fallback = path.join(f.root, "node_modules/.bin/tsgo");
      fs.mkdirSync(path.dirname(fallback), { recursive: true });
      fs.copyFileSync(f.executable, fallback);
      mutate(f);
      assert.throws(
        () => provisionFullRustRuntime(f.root, f.environment),
        Error,
        `mutation ${index}`,
      );
      assert.equal(
        fs.existsSync(f.receiptPath),
        false,
        `mutation ${index} must not receive provider credit`,
      );
      assert.equal(fs.readFileSync(f.environment.GITHUB_ENV, "utf8"), "EXISTING=retained\n");
    } finally {
      f.cleanup();
    }
  }
});

test("a newline in the canonical provider path cannot inject GITHUB_ENV", () => {
  const f = fixture();
  const moved = `${f.root}\nINJECTED=value`;
  try {
    fs.renameSync(f.root, moved);
    const environment = { ...f.environment, GITHUB_ENV: path.join(moved, "github-env") };
    assert.throws(() => provisionFullRustRuntime(moved, environment), /cannot be written safely/);
    assert.equal(fs.readFileSync(environment.GITHUB_ENV, "utf8"), "EXISTING=retained\n");
  } finally {
    fs.rmSync(moved, { recursive: true, force: true });
    f.cleanup();
  }
});

test("an existing provider receipt is retained and prevents a new environment export", () => {
  const f = fixture();
  try {
    fs.mkdirSync(path.dirname(f.receiptPath), { recursive: true });
    fs.writeFileSync(f.receiptPath, "preserved earlier provider evidence\n");
    assert.throws(() => provisionFullRustRuntime(f.root, f.environment), /EEXIST/);
    assert.equal(fs.readFileSync(f.receiptPath, "utf8"), "preserved earlier provider evidence\n");
    assert.equal(fs.readFileSync(f.environment.GITHUB_ENV, "utf8"), "EXISTING=retained\n");
  } finally {
    f.cleanup();
  }
});

test("actual workflow carrier precedes both full profiles and its receipt is uploaded", () => {
  type Step = { name?: string; run?: string; uses?: string; with?: { path?: string } };
  const workflow = parse(readRepoFile(".github/workflows/full-qualification.yml")) as {
    jobs: Record<string, { steps: Step[] }>;
  };
  const steps = workflow.jobs["full-rust-workspace"].steps;
  const carrier = steps.findIndex(
    (step) => step.name === "Provision the locked Full Rust native runtime",
  );
  assert.ok(carrier > steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@")));
  assert.ok(
    carrier <
      steps.findIndex(
        (step) => step.name === "Run the original test profile for same-source comparison",
      ),
  );
  assert.ok(
    carrier <
      steps.findIndex((step) => step.name === "Run and inventory every workspace test and doctest"),
  );
  assert.equal(steps[carrier].run, "node tools/support/compat/github/full-rust-runtime.ts");
  assert.equal(
    steps.find((step) => step.name === "Run the original test profile for same-source comparison")
      ?.run,
    "node tools/support/compat/github/run-rust-workspace-tests.ts --cargo-profile test --output target/nextest/full/workspace-reference-timing.json",
  );
  assert.equal(
    steps.find((step) => step.name === "Run and inventory every workspace test and doctest")?.run,
    "node tools/support/compat/github/run-rust-workspace-tests.ts --cargo-profile ci --capture-typechecker-workspace",
  );
  const upload = parse(
    readRepoFile(".github/actions/upload-rust-full-workspace-results/action.yml"),
  ) as { runs: { steps: Step[] } };
  assert.equal(upload.runs.steps[0].with?.path, "target/nextest/full/");
  const f = fixture();
  try {
    const result = spawnSync(process.execPath, [script], {
      cwd: f.root,
      env: f.environment,
      encoding: "utf8",
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(f.readReceipt().selection, "checkout-locked-platform-package");
    assert.equal(
      fs.readFileSync(f.environment.GITHUB_ENV, "utf8"),
      `EXISTING=retained\nCORSA_PATH=${f.executable}\n`,
    );
  } finally {
    f.cleanup();
  }
});

test("source coverage uses the same carrier before full native tests and retains its receipt", () => {
  type Step = {
    name?: string;
    run?: string;
    if?: string;
    uses?: string;
    with?: { path?: string; name?: string };
  };
  const workflow = parse(readRepoFile(".github/workflows/check.yml")) as {
    jobs: Record<string, { steps: Step[] }>;
  };
  const steps = workflow.jobs["source-coverage"].steps;
  const carrier = steps.findIndex(
    (step) => step.name === "Provision the locked Full Rust native runtime",
  );
  assert.ok(carrier > steps.findIndex((step) => step.name === "Install JS dependencies"));
  assert.ok(carrier < steps.findIndex((step) => step.name === "Source coverage report"));
  assert.equal(steps[carrier].run, "node tools/support/compat/github/full-rust-runtime.ts");
  assert.equal(
    steps.find((step) => step.name === "Source coverage report")?.run,
    "vp run --workspace-root coverage:source",
  );
  const retained = steps.find(
    (step) => step.name === "Retain source coverage native runtime provider",
  );
  assert.equal(retained?.if, "${{ always() }}");
  assert.equal(retained?.with?.path, "target/nextest/full/native-runtime-provider.json");
  assert.equal(
    retained?.with?.name,
    "source-coverage-native-runtime-${{ github.sha }}-${{ github.run_id }}-${{ github.run_attempt }}",
  );
});
