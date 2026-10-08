import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import os from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import {
  selectSourceRecipe,
  verifyVersionCut,
  WORKFLOW,
  STEP,
} from "./typechecker-native-source-recipe.mjs";
import { rewriteVersionMetadata } from "./typechecker-native-release-metadata.mjs";

import {
  parse,
  hash,
  frozen,
  currentBytes,
  git,
  put,
  commit,
  fixture,
} from "./typechecker-native-source-recipe-fixture.mjs";

test("complete authenticated actual C6d/H339 metadata selects all original seven targets and full script", () => {
  assert.equal(frozen.originalCut, "6d26d84b4240e9356cf5078b6d277e181d311a42");
  assert.equal(frozen.originalHead, "3390cb6044d53d655a9d64e112d2618375cff5ca");
  fixture((f) => {
    const selected = selectSourceRecipe(f);
    assert.equal(selected.mode, "immutable-source-step");
    assert.equal(selected.changes.length, 37);
    const sourceWorkflow = frozen.files.find((row) => row.path === WORKFLOW);
    assert.equal(sourceWorkflow.before, sourceWorkflow.after);
    const step = parse(sourceWorkflow.before).jobs["native-phases"].steps.find(
      (s) => s.name === STEP,
    );
    assert.equal(selected.recipe, step.run);
    assert.equal(selected.recipeSha256, hash(step.run));
    const firstCargo = selected.recipe.split("\n").find((line) => line.includes("cargo test"));
    assert.deepEqual(
      [...firstCargo.matchAll(/--test ([a-z0-9_]+)/gu)].map((m) => m[1]),
      [
        "check_template_emit_cli",
        "check_vue_helper_scope_cli",
        "lint_unchecked_indexed_access_cli",
        "check_canon_define_model_modifiers_cli",
        "lsp_generic_prop_hover_cli",
        "lsp_data_aria_attributes_cli",
        "tsconfig_diamond",
      ],
    );
    assert.ok(selected.recipe.endsWith("-- --nocapture\n"));
    assert.equal(selected.unavailableCurrentCapture, "tsconfig-types-extends");
    assert.notEqual(
      f.source,
      frozen.originalHead,
      "Reconstructed source is not relabelled historical H",
    );
  });
});

test("ordinary source retains complete inline eight-target script and original downstream flags", () => {
  fixture((f) => {
    f.event.pull_request.head.ref = "fix/current-native";
    f.event.pull_request.body = "";
    assert.equal(selectSourceRecipe(f).mode, "current-inline");
    const sourceRun = parse(frozen.files.find((row) => row.path === WORKFLOW).before).jobs[
      "native-phases"
    ].steps.find((s) => s.name === STEP).run;
    const run = parse(currentBytes).jobs["native-phases"].steps.find((s) => s.name === STEP).run;
    const inline =
      "set -euo pipefail\n" +
      run
        .split("\nelse\n")[1]
        .replace(/\nfi\n$/u, "\n")
        .replace(/^  /gmu, "");
    assert.equal(
      inline,
      sourceRun.replace(" --config ", " --test check_tsconfig_types_extends_cli --config "),
    );
    assert.ok(run.includes('bash --noprofile --norc -e -o pipefail "$recipe"'));
    assert.ok(run.includes("GITHUB_WORKFLOW_SHA"));
    assert.equal([...run.matchAll(/--test check_tsconfig_types_extends_cli/gu)].length, 1);
    for (const line of sourceRun.split("\n").slice(4).filter(Boolean))
      assert.ok(run.includes(line), line);
  });
});

test("outside-checkout helper bytes must match immutable workflow SHA before current inline selection", () => {
  fixture((f) => {
    const helperNames = [
      "typechecker-native-source-recipe.mjs",
      "typechecker-native-release-metadata.mjs",
    ];
    const helperRoot = join(f.root, "tools/benchmarks/scripts");
    for (const name of helperNames)
      put(
        f.root,
        `tools/benchmarks/scripts/${name}`,
        readFileSync(new URL(name, import.meta.url), "utf8"),
      );
    put(f.root, "package.json", '{"type":"module"}\n');
    put(f.root, ".gitignore", "node_modules/\n");
    const source = commit(f.root);
    symlinkSync(
      fileURLToPath(new URL("../../../node_modules", import.meta.url)),
      join(f.root, "node_modules"),
    );
    const temp = mkdtempSync(join(os.tmpdir(), "source-recipe-bootstrap-"));
    try {
      for (const name of helperNames)
        writeFileSync(join(temp, name), readFileSync(join(helperRoot, name)));
      const event = structuredClone(f.event);
      event.pull_request.head.sha = source;
      event.pull_request.head.ref = "fix/current";
      event.pull_request.body = "";
      writeFileSync(join(temp, "event.json"), JSON.stringify(event));
      const env = {
        ...process.env,
        ...f.env,
        SOURCE_SHA: source,
        DRIVER_SHA: source,
        GITHUB_WORKFLOW_SHA: source,
        NATIVE_PHASE_SOURCE_ROOT: f.root,
        GITHUB_EVENT_PATH: join(temp, "event.json"),
        RUNNER_TEMP: temp,
      };
      const run = (overrides) =>
        spawnSync(process.execPath, [join(temp, helperNames[0])], {
          cwd: f.root,
          env: { ...env, ...overrides },
          encoding: "utf8",
        });
      const accepted = run({});
      assert.equal(accepted.status, 0, accepted.stderr);
      assert.equal(accepted.stdout, "");
      assert.equal(
        JSON.parse(readFileSync(join(temp, "native-source-recipe.json"))).mode,
        "current-inline",
      );
      assert.notEqual(run({ GITHUB_WORKFLOW_SHA: "partial" }).status, 0);
      for (const name of helperNames) {
        const bytes = readFileSync(join(temp, name));
        writeFileSync(join(temp, name), Buffer.concat([bytes, Buffer.from("\n// forged bytes\n")]));
        assert.notEqual(run({}).status, 0, name);
        writeFileSync(join(temp, name), bytes);
      }
    } finally {
      rmSync(temp, { recursive: true, force: true });
    }
  });
});

test("forged identity, protocol, permission, pin and source context never fall back to current", () => {
  fixture((f) => {
    const controls = [
      (x) => {
        x.env.GITHUB_REPOSITORY = "foreign/repo";
      },
      (x) => {
        x.event.pull_request.head.repo.full_name = "fork/repo";
      },
      (x) => {
        x.event.pull_request.base.ref = "other";
      },
      (x) => {
        x.event.pull_request.head.sha = f.cut;
      },
      (x) => {
        x.event.pull_request.draft = false;
      },
      (x) => {
        x.event.pull_request.merged = true;
      },
      (x) => {
        x.event.pull_request.state = "closed";
      },
      (x) => {
        x.permission.role_name = "write";
      },
      (x) => {
        x.pin = f.source;
      },
      (x) => {
        x.env.MAIN_SOURCE_SHA = f.source;
      },
      (x) => {
        x.env.GITHUB_WORKFLOW_SHA = "partial";
      },
      (x) => {
        x.env.GITHUB_WORKFLOW_REF = `foreign/repo/${WORKFLOW}@refs/heads/main`;
      },
      (x) => {
        x.env.GITHUB_EVENT_NAME = "workflow_dispatch";
      },
      (x) => {
        x.event.pull_request.body += "\n<!-- vize-release-pin: immutable-v1 -->";
      },
      (x) => {
        x.event.pull_request.body = x.event.pull_request.body.replace(f.cut, f.source);
      },
      (x) => {
        x.event.pull_request.body = "";
      },
      (x) => {
        x.event.pull_request.head.ref = "release/v0.435.1";
      },
    ];
    for (const control of controls) {
      const x = {
        ...f,
        env: { ...f.env },
        event: structuredClone(f.event),
        permission: { ...f.permission },
      };
      control(x);
      assert.throws(() => selectSourceRecipe(x));
    }
    const extraParent = git(
      f.root,
      "commit-tree",
      git(f.root, "rev-parse", `${f.source}^{tree}`),
      "-p",
      f.source,
      "-p",
      f.cut,
      "-m",
      "foreign merge",
    );
    git(f.root, "checkout", "--detach", extraParent);
    assert.throws(() =>
      selectSourceRecipe({
        ...f,
        env: { ...f.env, SOURCE_SHA: extraParent, DRIVER_SHA: extraParent },
      }),
    );
  });
});

test("partial version, production, dependencies, flags, target deletion and modes are rejected", () => {
  fixture((f) => {
    for (const [path, change] of [
      ["npm/cli/package.json", (bytes) => bytes.replace("0.436.0", "0.435.1")],
      ["pnpm-lock.yaml", (bytes) => bytes + "\n# foreign dependency mutation\n"],
      ["crates/vize/src/recipe-control.rs", (bytes) => bytes + "// foreign product\n"],
      [WORKFLOW, (bytes) => bytes.replace("--locked", "--offline")],
      [WORKFLOW, (bytes) => bytes.replace(" --test tsconfig_diamond", "")],
    ]) {
      git(f.root, "checkout", "--detach", f.source);
      put(f.root, path, change(readFileSync(join(f.root, path), "utf8")));
      const altered = commit(f.root);
      assert.throws(() => verifyVersionCut(f.root, f.cut, altered, "0.435.1", "0.436.0"), path);
    }
    git(f.root, "checkout", "--detach", f.source);
    chmodSync(join(f.root, "Cargo.toml"), 0o755);
    assert.throws(() => verifyVersionCut(f.root, f.cut, commit(f.root), "0.435.1", "0.436.0"));
  });
});

test("exact rewrite preserves benchmark, foreign dependency and non-version bytes", () => {
  const old = "1.2.3",
    next = "1.2.4";
  assert.equal(
    rewriteVersionMetadata("crates/foreign/Cargo.toml", 'version = "1.2.3"\n', old, next),
    'version = "1.2.3"\n',
  );
  assert.equal(
    rewriteVersionMetadata(
      "Cargo.lock",
      '[[package]]\nname = "foreign"\nversion = "1.2.3"\n',
      old,
      next,
    ),
    '[[package]]\nname = "foreign"\nversion = "1.2.3"\n',
  );
  assert.equal(
    rewriteVersionMetadata(
      "README.md",
      "1.2.3\n<!-- benchmark:readme:start -->\n1.2.3\n<!-- benchmark:readme:end -->\n1.2.3\n",
      old,
      next,
    ),
    "1.2.4\n<!-- benchmark:readme:start -->\n1.2.3\n<!-- benchmark:readme:end -->\n1.2.4\n",
  );
});
