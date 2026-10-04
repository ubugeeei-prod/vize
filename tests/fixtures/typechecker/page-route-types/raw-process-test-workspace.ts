import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import type { TestContext } from "node:test";

import type { PinnedFixtureWorkspace } from "../../../_helpers/realworld-patch.ts";
import { pageRouteObserver } from "./cli-observer.ts";
import type { RecordObservation } from "./raw-process.ts";
import { PAGE_PATH, PLAYGROUND, ROUTES_PATH } from "./support.ts";

export function temporary(t: TestContext): string {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-router-raw-process-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}

/** Only owned source files and Node stubs; no upstream archive, CLI or provider preparation. */
export function fakeFixture(t: TestContext) {
  const workspaceDir = temporary(t);
  const fixture: PinnedFixtureWorkspace = {
    entry: {
      id: "owned-node-control",
      displayName: "owned Node control",
      fixturePath: workspaceDir,
      revision: "0".repeat(40),
    },
    upstreamDir: workspaceDir,
    workspaceDir,
    resolve: (file) => path.join(workspaceDir, file),
    read: (file) => fs.readFileSync(path.join(workspaceDir, file), "utf8"),
    write(file, source) {
      const target = path.join(workspaceDir, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, source);
    },
    applyExactPatch() {
      throw new Error("owned fake fixture never uses upstream patching");
    },
  };
  const source = "<script setup lang='ts'>type T = ReturnType<typeof useRoute></script>\r\n";
  const routes = "// owned map authority; no actual provider\nexport {};\n";
  fixture.write(PAGE_PATH, source);
  fixture.write(ROUTES_PATH, routes);
  fixture.write("tsconfig.json", `${JSON.stringify({ include: [PAGE_PATH, ROUTES_PATH] })}\r\n`);
  fixture.write("vize.config.json", '{ "owned": true }\r\n');
  const configuration = [
    "tsconfig.json",
    "vize.config.json",
    `${PLAYGROUND}/compiler-options.json`,
  ];
  fixture.write(configuration[2], "owned inherited configuration\r\n");
  const before = new Map(
    [PAGE_PATH, ROUTES_PATH, ...configuration].map(
      (file) => [file, fs.readFileSync(fixture.resolve(file))] as const,
    ),
  );
  function assertRestored() {
    for (const [file, bytes] of before)
      assert.deepEqual(fs.readFileSync(fixture.resolve(file)), bytes);
    assert.equal(fs.existsSync(fixture.resolve("project-root-map.d.ts")), false);
  }
  function stub(body: string) {
    const file = fixture.resolve("owned-cli-stub.cjs");
    fs.writeFileSync(file, `#!${process.execPath}\n${body}\n`, { mode: 0o755 });
    return file;
  }
  function observer(
    capture: { record: RecordObservation },
    cli: string,
    reference = "owned-reference-unused",
  ) {
    return pageRouteObserver(
      t,
      fixture,
      capture.record,
      cli,
      "owned-corsa-unused",
      reference,
      source,
      routes,
    );
  }
  return { fixture, source, stub, assertRestored, observer };
}
