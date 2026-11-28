import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

// docs/davinci/decisions/2026-11-17-creusot.md: the Creusot proof crate
// includes shipped kernels by path, CI replays every committed certificate,
// the product build never sees Creusot, and every trusted item is listed.

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const crateDir = path.join(repoRoot, "tests", "formal", "creusot");
const workflowFile = path.join(repoRoot, ".github", "workflows", "07-proofs-creusot.yml");
const inventoryFile = path.join(
  repoRoot,
  "docs",
  "davinci",
  "decisions",
  "2026-11-17-creusot-inventory.md",
);

type Step = {
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
  if?: unknown;
  "continue-on-error"?: unknown;
  "working-directory"?: string;
};
type Workflow = {
  on: {
    pull_request: { paths: string[] };
    push: { branches: string[]; paths: string[] };
    workflow_dispatch: unknown;
  };
  jobs: Record<
    string,
    { name?: string; steps: Step[]; if?: unknown; "continue-on-error"?: unknown }
  >;
};

/** Each proof package's library root, which sits next to its kernel. */
function proofRoots(): string[] {
  const workspace = fs.readFileSync(path.join(crateDir, "Cargo.toml"), "utf8");
  const members = [
    ...(workspace.match(/^members = \[([^\]]*)\]$/mu)?.[1] ?? "").matchAll(/"([^"]+)"/gu),
  ];
  assert.ok(members.length > 0, "proof workspace members");
  return members.map((member) => {
    const manifest = fs.readFileSync(path.join(crateDir, member[1]!, "Cargo.toml"), "utf8");
    const lib = manifest.match(/^\[lib\]\npath = "([^"]+)"$/mu)?.[1];
    assert.ok(lib, `${member[1]} names its library root`);
    return path.resolve(crateDir, member[1]!, lib);
  });
}

/** The repository-relative roots and the kernel modules they declare. */
function includedKernels(): string[] {
  const kernels = new Set<string>();
  for (const root of proofRoots()) {
    kernels.add(path.relative(repoRoot, root).split(path.sep).join("/"));
    for (const match of fs.readFileSync(root, "utf8").matchAll(/^\s*(?:pub\s+)?mod (\w+);$/gmu)) {
      const target = path.join(path.dirname(root), `${match[1]!}.rs`);
      assert.ok(fs.existsSync(target), `${root} declares missing ${match[1]}`);
      kernels.add(path.relative(repoRoot, target).split(path.sep).join("/"));
    }
    // A kernel the product itself `include!`s is included verbatim the same way.
    for (const match of fs.readFileSync(root, "utf8").matchAll(/^include!\("(\w+\.rs)"\);$/gmu)) {
      const target = path.join(path.dirname(root), match[1]!);
      assert.ok(fs.existsSync(target), `${root} includes missing ${match[1]}`);
      kernels.add(path.relative(repoRoot, target).split(path.sep).join("/"));
    }
  }
  return [...kernels].sort();
}

test("the Creusot workflow replays every proof whenever a proved kernel changes", () => {
  const workflow = parseYaml(fs.readFileSync(workflowFile, "utf8")) as Workflow;
  const kernels = includedKernels();
  assert.ok(kernels.length > 0);
  const own = [
    ".github/workflows/07-proofs-creusot.yml",
    "tools/nix/creusot/**",
    "tests/formal/creusot/**",
  ];
  const expected = [...own, ...kernels];
  assert.deepEqual([...workflow.on.pull_request.paths].sort(), [...expected].sort());
  assert.deepEqual([...workflow.on.push.paths].sort(), [...expected].sort());
  assert.deepEqual(workflow.on.push.branches, ["main"]);
  assert.ok(Object.hasOwn(workflow.on, "workflow_dispatch"));

  assert.deepEqual(Object.keys(workflow.jobs).sort(), ["editor-projection-discovery", "prove"]);
  const job = workflow.jobs.prove!;
  assert.equal(job.name, "Creusot proofs");
  assert.equal(job.if, undefined, "full replay always runs, including discovery dispatches");
  assert.ok(job["continue-on-error"] === undefined || job["continue-on-error"] === false);
  const runs = job.steps.flatMap((step) => {
    assert.equal(step.if, undefined);
    assert.ok(step["continue-on-error"] === undefined || step["continue-on-error"] === false);
    return step.run ? [[step["working-directory"] ?? ".", step.run.trim()]] : [];
  });
  assert.deepEqual(runs.at(-1), [
    "tests/formal/creusot",
    "nix develop 'path:../../../tools/nix/creusot' --command cargo creusot --replay -- --locked",
  ]);
});

test("certificate discovery has a separate manual check and never substitutes for replay", () => {
  const workflow = parseYaml(fs.readFileSync(workflowFile, "utf8")) as Workflow;
  const job = workflow.jobs["editor-projection-discovery"]!;
  assert.equal(job.name, "Discover editor projection certificates");
  assert.equal(
    job.if,
    "${{ github.event_name == 'workflow_dispatch' && inputs.discover_editor_projection }}",
  );
  assert.ok(job["continue-on-error"] === undefined || job["continue-on-error"] === false);
  for (const step of job.steps) {
    assert.ok(step["continue-on-error"] === undefined || step["continue-on-error"] === false);
    if (step.uses?.startsWith("actions/upload-artifact@")) {
      assert.equal(step.if, "always()");
      assert.equal(
        step.with?.path,
        "tests/formal/creusot/verif/vize_incremental_editor_projection_proofs_rlib/",
      );
    } else {
      assert.equal(step.if, undefined);
    }
  }
  const search = job.steps.filter((step) => step.run?.includes("cargo creusot"));
  assert.deepEqual(
    search.map((step) => [step["working-directory"], step.run]),
    [
      [
        "tests/formal/creusot",
        "nix develop 'path:../../../tools/nix/creusot' --command cargo creusot -- --locked -p vize_incremental_editor_projection_proofs",
      ],
    ],
  );
  assert.equal(workflow.jobs.prove!.if, undefined);
});

test("the product workspace never depends on Creusot", () => {
  const lock = fs.readFileSync(path.join(repoRoot, "Cargo.lock"), "utf8");
  assert.doesNotMatch(lock, /name = "creusot-std/u);
  const manifest = fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
  assert.match(manifest, /^exclude = \[[^\]]*"tests\/formal\/creusot"[^\]]*\]$/mu);
  assert.match(manifest, /check-cfg = \[[^\]]*'cfg\(creusot\)'[^\]]*\]/u);
  // Proof roots are never part of a product module tree; shipped kernels are.
  const roots = new Set(
    proofRoots().map((root) => path.relative(repoRoot, root).split(path.sep).join("/")),
  );
  for (const kernel of includedKernels().filter((kernel) => !roots.has(kernel))) {
    const source = fs.readFileSync(path.join(repoRoot, kernel), "utf8");
    // Creusot items reach a kernel only behind `cfg(creusot)`.
    for (const line of source.split("\n")) {
      if (/\bcreusot_std\b/u.test(line)) {
        assert.match(line, /^\s*(use creusot_std::|\/\/)/u, `${kernel}: ${line.trim()}`);
      }
    }
    const uses = [...source.matchAll(/^(.*)\n\s*use creusot_std::/gmu)];
    for (const use of uses) assert.equal(use[1]!.trim(), "#[cfg(creusot)]", kernel);
  }
});

test("every trusted item in the proofs is listed in the inventory", () => {
  const sources = [
    ...proofRoots(),
    ...includedKernels().map((kernel) => path.join(repoRoot, kernel)),
  ];
  // A trusted attribute, bare or under `cfg_attr(creusot, ...)`, and the item
  // it marks: the first following line that is not an attribute.
  const trusted = sources.flatMap((file) => {
    const lines = fs.readFileSync(file, "utf8").split("\n");
    const relative = path.relative(repoRoot, file).split(path.sep).join("/");
    return lines.flatMap((line, index) => {
      if (!/^\s*#\[(?:cfg_attr\(creusot,\s*)?trusted\b/u.test(line)) return [];
      const item = lines.slice(index + 1).find((next) => !/^\s*#\[/u.test(next));
      return [`${relative}: ${item?.trim() ?? ""}`];
    });
  });
  const inventory = fs.readFileSync(inventoryFile, "utf8");
  const section = inventory.slice(inventory.indexOf("## Trusted items"));
  const listed = [...section.matchAll(/^- `([^`]+)`/gmu)].map((match) => match[1]!);
  assert.deepEqual(trusted.sort(), listed.sort());
});
