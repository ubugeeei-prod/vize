import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import type { Node } from "yaml";

import { stagedWorkflowAction, verifyWorkflowStage } from "./workflow-stage.ts";
import { withoutGeneratedHeader } from "../../../tools/support/ci/generated-header.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const actionScript = /^\. "\$GITHUB_ACTION_PATH\/scripts\/([\w.-]+\.(?:sh|ps1))"$/u;

const workflowScript =
  /^\. "(?:\$\{\{ env\.VIZE_CI_FILES \}\}|\$VIZE_CI_FILES)\/([\w.-]+\.(?:sh|ps1))"$/u;

const shellFiles = new Map<string, Set<string>>();

function shellBody(file: string): string {
  const body = withoutGeneratedHeader(readFileSync(file, "utf8").replace(/\r\n?/gu, "\n"), "hash")
    .replace(/^#![^\n]*\n/u, "")
    .trimEnd();
  const files = shellFiles.get(body) ?? new Set<string>();
  files.add(file);
  shellFiles.set(body, files);
  return body;
}

/** Execute the file that supplied a step's source-contract text. */
export function automationShellPath(body: string, owner?: string): string {
  const files = [...(shellFiles.get(body.trimEnd()) ?? [])].filter(
    (file) => !owner || file.split(path.sep).includes(owner),
  );
  assert.equal(files.length, 1, "expected one checked-in shell program for the selected step");
  return files[0]!;
}

/** The pre-install benchmark contracts inspect text and run with Node alone. */
export function readWorkflowShell(owner: string): string {
  assert.match(owner, /^[a-z0-9-]+$/u);
  const workflow = withoutGeneratedHeader(
    readFileSync(path.join(root, ".github/workflows", owner + ".yml"), "utf8"),
    "hash",
  );
  return workflow.replace(
    /^( *)(- )?run: (.+)$/gmu,
    (line: string, indent: string, item: string | undefined, command: string) => {
      const script = workflowScript.exec(command);
      if (!script) return line;
      const program = shellBody(path.join(root, ".github/scripts/workflows", owner, script[1]!));
      const padding = indent + (item ?? "").replace("-", " ") + "  ";
      return (
        indent +
        (item ?? "") +
        "run: |\n" +
        program
          .split("\n")
          .map((row) => padding + row)
          .join("\n")
      );
    },
  );
}

/** Read the executable bodies for source-contract tests while retaining each step's metadata. */
export function readAutomationPrograms(...parts: (string | URL)[]): string {
  if (parts.at(-1) === "utf8") parts.pop();
  const file =
    parts[0] instanceof URL ? fileURLToPath(parts[0]) : path.resolve(root, ...(parts as string[]));
  const source = withoutGeneratedHeader(
    readFileSync(file, "utf8").replace(/\r\n?/gu, "\n"),
    "hash",
  );
  const { isMap, isScalar, isSeq, parseDocument } = createRequire(import.meta.url)(
    "yaml",
  ) as typeof import("yaml");
  const document = parseDocument(source);
  assert.deepEqual(document.errors, []);
  const edits = new Map<number, { end: number; value: string }>();

  const workflowOwner =
    path.dirname(file) === path.join(root, ".github/workflows")
      ? path.basename(file, path.extname(file))
      : undefined;
  if (workflowOwner) {
    const jobs = document.get("jobs", true);
    assert.ok(isMap(jobs));
    for (const pair of jobs.items) {
      const job = pair.value;
      if (!isMap(job)) continue;
      const steps = job.get("steps", true);
      if (isSeq(steps)) {
        const staged = steps.items.findIndex(
          (step) => isMap(step) && step.get("uses") === stagedWorkflowAction,
        );
        if (staged >= 0) {
          const checkout = steps.items[0],
            action = steps.items[1];
          assert.equal(staged, 1);
          assert.ok(isMap(checkout) && isMap(action));
          verifyWorkflowStage(workflowOwner, checkout.toJS(document), action.toJS(document));
          assert.ok(checkout.range && action.range);
          const start = source.lastIndexOf("\n", checkout.range[0]) + 1;
          const end = action.range[2];
          edits.set(start, { end, value: "" });
        }
      }
      const env = job.get("env", true);
      if (isMap(env) && env.has("VIZE_CI_FILES")) {
        const dirs = new Set([
          "${{ github.workspace }}/.github/scripts/workflows/" + workflowOwner,
        ]);
        if (isSeq(steps))
          for (const child of steps.items) {
            if (
              !isMap(child) ||
              child.get("uses") !== "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"
            )
              continue;
            const opts = child.get("with", true);
            if (
              !isMap(opts) ||
              opts.has("repository") ||
              opts.has("sparse-checkout") ||
              child.has("if")
            )
              continue;
            const dir = opts.get("path");
            if (
              typeof dir === "string" &&
              dir.split("/").every((part) => /^[a-zA-Z0-9_-]+$/u.test(part)) &&
              [undefined, "${{ github.sha }}", "${{ github.workflow_sha }}"].includes(
                opts.get("ref") as string | undefined,
              )
            )
              dirs.add(
                "${{ github.workspace }}/" + dir + "/.github/scripts/workflows/" + workflowOwner,
              );
          }
        assert.ok(
          dirs.has(String(env.get("VIZE_CI_FILES"))),
          "workflow scripts must belong to the checked-out revision",
        );
        const entry = env.items.find(
          (pair) => isScalar(pair.key) && pair.key.value === "VIZE_CI_FILES",
        )!;
        const node =
          env.items.length === 1
            ? job.items.find((pair) => isScalar(pair.key) && pair.key.value === "env")!.key
            : entry.key;
        assert.ok(isScalar(node) && isScalar(entry.value) && node.range && entry.value.range);
        if (env.flow && env.items.length > 1) {
          const index = env.items.indexOf(entry),
            next = env.items[index + 1]?.key,
            previous = env.items[index - 1]?.value;
          if (isScalar(next) && next.range)
            edits.set(node.range[0], { end: next.range[0], value: "" });
          else {
            assert.ok(isScalar(previous) && previous.range);
            edits.set(previous.range[1], { end: entry.value.range[1], value: "" });
          }
        } else {
          const start = source.lastIndexOf("\n", node.range[0]) + 1;
          const end = env.items.length === 1 ? env.range![2] : entry.value.range[2];
          edits.set(start, { end, value: "" });
        }
      }
    }
  }
  function visit(node: Node | null): void {
    if (isMap(node)) {
      for (const pair of node.items) {
        const child = pair.value as Node | null;
        if (
          isScalar(pair.key) &&
          pair.key.value === "run" &&
          isScalar(child) &&
          typeof child.value === "string"
        ) {
          const action = actionScript.exec(child.value);
          const workflow = workflowScript.exec(child.value);
          if (!action && !workflow) continue;
          assert.ok(child.range && pair.key.range);
          const script = action
            ? path.join(path.dirname(file), "scripts", action[1]!)
            : path.join(
                root,
                ".github/scripts/workflows",
                path.basename(file, path.extname(file)),
                workflow![1]!,
              );
          const body = shellBody(script);
          const column = pair.key.range[0] - source.lastIndexOf("\n", pair.key.range[0]) - 1;
          const padding = " ".repeat(column + 2);
          const text = body
            .split("\n")
            .map((line) => (line ? padding + line : ""))
            .join("\n");
          // Flow mappings cannot contain YAML block scalars.
          const value = node.flow
            ? JSON.stringify(body + (body.includes("\n") ? "\n" : ""))
            : (body.includes("\n") ? "|\n" : "|-\n") + text;
          edits.set(child.range[0], { end: child.range[1], value });
        } else {
          visit(child);
        }
      }
    } else if (isSeq(node)) {
      for (const child of node.items) visit(child as Node | null);
    }
  }
  visit(document.contents);
  let result = source;
  for (const [start, { end, value }] of [...edits].sort(([a], [b]) => b - a)) {
    result = result.slice(0, start) + value + result.slice(end);
  }
  return result;
}

/** Locate the module the capture step passes to Node. */
export function actionNodeProgramPath(owner: string, command: string | undefined): string {
  const matches = [
    ...(command ?? "").matchAll(
      /\bnode "\$GITHUB_ACTION_PATH\/scripts\/([\w.-]+\.ts)" "\$capture"/gu,
    ),
  ];
  assert.equal(matches.length, 1, "the capture step must run one named Node module");
  return path.join(root, ".github/actions", owner, "scripts", matches[0]![1]!);
}

/** Read the named program or filter reached by the actual workflow step. */
export function workflowProgram(owner: string, command: string, program: string): string {
  assert.match(owner, /^[a-z0-9-]+$/u);
  assert.match(program, /^[a-z0-9-]+\.(?:ts|cjs|jq)$/u);
  assert.ok(
    command.includes('"$VIZE_CI_FILES/' + program + '"') ||
      command.includes('"${{ env.VIZE_CI_FILES }}/' + program + '"'),
    owner + " does not invoke " + program,
  );
  return withoutGeneratedHeader(
    readFileSync(path.join(root, ".github/scripts/workflows", owner, program), "utf8"),
    "hash",
  );
}
