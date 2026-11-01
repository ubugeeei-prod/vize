import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { candidateAllocationBodies } from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-allocation.ts";
import type { BodyPin } from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-source.ts";

const pin = (body: Buffer): BodyPin => ({
  bytes: body.length,
  sha256: createHash("sha256").update(body).digest("hex"),
  gitBlob: createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
});
function fixture(owned: boolean) {
  const cli = Buffer.from(
    "// full CLI fixture\nmod checkers;\nfn output() {\n        let output = command.output();\n        cache_receipt(&output);\n        output\n}\n",
  );
  const execution = Buffer.from(
    `// full execution fixture\nuse std::{path::PathBuf};\nfn prepare() {\n    checker.set_package_routes(input.package_routes${owned ? "" : ".iter().cloned()"});\n}\n`,
  );
  const read = (name: string) =>
    fs.readFileSync(new URL("../../tools/benchmarks/observers/" + name, import.meta.url));
  return {
    kind: "route-move" as const,
    cli,
    execution,
    pins: { cli: pin(cli), execution: pin(execution) },
    childObserver: read("paired-candidate-child-allocation.rs"),
    routeObserver: read("paired-candidate-route-memory.rs"),
  };
}
test("allocation overlay preserves both borrowed and owned setter bodies and exact original result consumers", () => {
  for (const owned of [false, true]) {
    const x = fixture(owned),
      saved = [Buffer.from(x.cli), Buffer.from(x.execution)];
    const result = candidateAllocationBodies(x);
    assert.equal(result.length, 4);
    const cli = result[0].body.toString(),
      execution = result[2].body.toString();
    assert(cli.includes("cache_receipt(&output);\n        output"));
    assert(cli.includes("if let Err(failure) = observation_write"));
    assert(!cli.includes("observation_write?"));
    const originalCli = cli.replace("mod child_allocation_observer;\n", "").replace(
      `let (output, observation_write) = child_allocation_observer::output(&mut command);
        if let Err(failure) = observation_write {
            eprintln!("VIZE_CHILD_ALLOCATION_OBSERVATION_FAILED stage={} error={}", failure.stage, failure.error);
        }`,
      "let output = command.output();",
    );
    assert(Buffer.from(originalCli).equals(x.cli), "entire original CLI body restored");
    const setter = `checker.set_package_routes(input.package_routes${owned ? "" : ".iter().cloned()"});`;
    assert(execution.includes(setter));
    assert(execution.indexOf("route_memory_observer::record") < execution.indexOf(setter));
    const originalExecution = execution.replace("mod route_memory_observer;\n\n", "").replace(
      `    route_memory_observer::record(&input.package_routes, &input.program_root)
        .map_err(|error| cstr!("route-memory observation failed: {error}"))?;
`,
      "",
    );
    assert(
      Buffer.from(originalExecution).equals(x.execution),
      "entire original execution body restored",
    );
    assert(result[1].body.equals(x.childObserver) && result[3].body.equals(x.routeObserver));
    assert(x.cli.equals(saved[0]) && x.execution.equals(saved[1]));
    const typeWork = candidateAllocationBodies({ ...x, kind: "type-work" });
    assert.equal(typeWork.length, 2);
    assert(!typeWork.some((row) => row.path.includes("execution")));
  }
});
test("allocation overlay refuses stale source, duplicate/missing seams, changed observer and namespace reuse", () => {
  for (const alter of [
    (x: ReturnType<typeof fixture>) => {
      x.cli = Buffer.concat([x.cli, Buffer.from("changed")]);
    },
    (x: ReturnType<typeof fixture>) => {
      x.cli = Buffer.concat([x.cli, Buffer.from("let output = command.output();")]);
      x.pins.cli = pin(x.cli);
    },
    (x: ReturnType<typeof fixture>) => {
      x.cli = Buffer.from("mod checkers;\nno supported call");
      x.pins.cli = pin(x.cli);
    },
    (x: ReturnType<typeof fixture>) => {
      x.routeObserver = Buffer.from("different observer");
    },
    (x: ReturnType<typeof fixture>) => {
      x.execution = Buffer.from(
        x.execution.toString().replace("set_package_routes", "another_setter"),
      );
      x.pins.execution = pin(x.execution);
    },
    (x: ReturnType<typeof fixture>) => {
      x.execution = Buffer.concat([x.execution, Buffer.from("mod route_memory_observer;")]);
      x.pins.execution = pin(x.execution);
    },
  ]) {
    const x = fixture(false);
    alter(x);
    assert.throws(() => candidateAllocationBodies(x));
  }
});
