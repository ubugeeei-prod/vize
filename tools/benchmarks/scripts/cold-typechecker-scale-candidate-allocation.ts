/** Private, separately built allocation overlays; normal product source/images stay untouched. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { buildColdScaleBinary } from "./cold-typechecker-scale-build.ts";
import type { BodyPin } from "./cold-typechecker-scale-candidate-source.ts";

const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
export const allocationObserverSources = [
  {
    file: "paired-candidate-child-allocation.rs",
    bytes: 8801,
    sha256: "c46fa7401d4a6caf806f2fcd40b6fd386d0e5e6c4c8874ffc96439234ca1d462",
  },
  {
    file: "paired-candidate-route-memory.rs",
    bytes: 5670,
    sha256: "bb9087e000bdcfbc20fd32092ccaf60f10537cda7995437f19d7b82a9a76b5bb",
  },
] as const;
const cliPath = "crates/vize_canon/src/batch/executor/cli.rs";
const executionPath = "crates/vize/src/commands/check/runner/execution.rs";
const childPath = "crates/vize_canon/src/batch/executor/cli/child_allocation_observer.rs";
const routePath = "crates/vize/src/commands/check/runner/execution/route_memory_observer.rs";
const childCall = `let (output, observation_write) = child_allocation_observer::output(&mut command);
        if let Err(failure) = observation_write {
            eprintln!("VIZE_CHILD_ALLOCATION_OBSERVATION_FAILED stage={} error={}", failure.stage, failure.error);
        }`;
const routeCall = `    route_memory_observer::record(&input.package_routes, &input.program_root)
        .map_err(|error| cstr!("route-memory observation failed: {error}"))?;
`;

function replaceOne(body: Buffer, before: string, after: string) {
  const text = new TextDecoder("utf-8", { fatal: true }).decode(body);
  assert.equal(text.split(before).length, 2, "whole frozen source insertion must be unique");
  return Buffer.from(text.replace(before, after));
}
function verifyBody(body: Buffer, pin: BodyPin) {
  assert.equal(body.length, pin.bytes);
  assert.equal(hash(body), pin.sha256);
  assert.equal(
    createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
    pin.gitBlob,
  );
}

export function candidateAllocationBodies(input: {
  kind: "type-work" | "route-move";
  cli: Buffer;
  execution: Buffer;
  pins: { cli: BodyPin; execution: BodyPin };
  childObserver: Buffer;
  routeObserver: Buffer;
}) {
  assert(input.kind === "type-work" || input.kind === "route-move");
  verifyBody(input.cli, input.pins.cli);
  verifyBody(input.execution, input.pins.execution);
  for (const [index, body] of [input.childObserver, input.routeObserver].entries()) {
    assert.equal(body.length, allocationObserverSources[index].bytes);
    assert.equal(hash(body), allocationObserverSources[index].sha256);
  }
  for (const [body, name] of [
    [input.cli, "child_allocation_observer"],
    [input.execution, "route_memory_observer"],
  ] as const)
    assert(
      !body.includes(Buffer.from(name)),
      "existing observer namespace/call must not be reused",
    );
  let cli = replaceOne(input.cli, "mod checkers;", "mod child_allocation_observer;\nmod checkers;");
  cli = replaceOne(cli, "let output = command.output();", childCall);
  const bodies = [
    { path: cliPath, body: cli },
    { path: childPath, body: input.childObserver },
  ];
  if (input.kind === "route-move") {
    let execution = replaceOne(
      input.execution,
      "use std::{",
      "mod route_memory_observer;\n\nuse std::{",
    );
    const setter =
      /^    checker\.set_package_routes\(input\.package_routes(?:\.iter\(\)\.cloned\(\))?\);$/mu.exec(
        execution.toString("utf8"),
      );
    assert(setter, "only actual borrowed/owned setter seam is supported");
    execution = replaceOne(execution, setter[0], routeCall + setter[0]);
    bodies.push(
      { path: executionPath, body: execution },
      { path: routePath, body: input.routeObserver },
    );
  }
  return bodies;
}

/** Hosted-only preparation, after both normal images have been sealed. */
export function prepareCandidateAllocation(options: {
  sourceRoot: string;
  sourceSha: string;
  destination: string;
  output: string;
  target: string;
  role: "base" | "head";
  kind: "type-work" | "route-move";
  pins: { cli: BodyPin; execution: BodyPin };
}) {
  const git = (root: string, args: string[]) =>
    execFileSync("git", args, { cwd: root, maxBuffer: 8 * 1024 * 1024 })
      .toString()
      .trim();
  assert.match(options.sourceSha, /^[a-f0-9]{40}$/u);
  assert.equal(git(options.sourceRoot, ["rev-parse", "HEAD"]), options.sourceSha);
  assert.equal(git(options.sourceRoot, ["status", "--porcelain", "--untracked-files=all"]), "");
  for (const root of [options.destination, options.output, options.target]) {
    assert(path.isAbsolute(root));
    assert(
      fs.lstatSync(root, { throwIfNoEntry: false }) === undefined,
      "never reuse a source/build/receipt root",
    );
  }
  const readObserver = (file: string) =>
    fs.readFileSync(new URL("../observers/" + file, import.meta.url));
  const bodies = candidateAllocationBodies({
    kind: options.kind,
    cli: fs.readFileSync(path.join(options.sourceRoot, cliPath)),
    execution: fs.readFileSync(path.join(options.sourceRoot, executionPath)),
    pins: options.pins,
    childObserver: readObserver(allocationObserverSources[0].file),
    routeObserver: readObserver(allocationObserverSources[1].file),
  });
  git(options.sourceRoot, ["worktree", "add", "--detach", options.destination, options.sourceSha]);
  for (const { path: name, body } of bodies) {
    const file = path.join(options.destination, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    if (name === childPath || name === routePath) assert(!fs.existsSync(file));
    fs.writeFileSync(file, body);
  }
  git(options.destination, ["add", "--", ...bodies.map((row) => row.path)]);
  git(options.destination, ["diff", "--cached", "--check"]);
  git(options.destination, [
    "-c",
    "user.name=Vize benchmark",
    "-c",
    "user.email=benchmark@vize.dev",
    "commit",
    "-m",
    "test(bench): observe separate allocation windows",
  ]);
  const instrumentedSource = git(options.destination, ["rev-parse", "HEAD"]);
  const changed = git(options.destination, [
    "diff",
    "--name-only",
    options.sourceSha,
    instrumentedSource,
  ]).split("\n");
  assert.deepEqual(changed, bodies.map((row) => row.path).sort());
  fs.mkdirSync(options.output);
  fs.writeFileSync(
    path.join(options.output, "source.json"),
    JSON.stringify(
      {
        productSource: options.sourceSha,
        instrumentedSource,
        role: options.role,
        kind: options.kind,
        bodies: bodies.map(({ path, body }) => ({ path, bytes: body.length, sha256: hash(body) })),
        sourcePins: options.pins,
        normalImagesUnaffected: true,
        timingEligible: false,
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  return buildColdScaleBinary({
    sourceRoot: options.destination,
    targetDir: options.target,
    binary: path.join(options.output, "vize"),
    receipt: path.join(options.output, "build.json"),
    role: "allocation-" + options.role,
    allocation: true,
  });
}
