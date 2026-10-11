import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import type { Evidence } from "./vrt_artifacts.ts";
import type { LiteralTitle } from "./vrt_literal.ts";
import {
  persistedImagesAndReports,
  publicArtifact,
  wholeLiteralReports,
  type PublicSession,
} from "./vrt_trusted_ui.ts";

/** Report UUIDs belong to one real session registry, not to an individual Art permission. */
export async function foreignSessionReports(input: {
  session: PublicSession;
  other: PublicSession;
  root: string;
  origin: string;
  title: LiteralTitle;
  reports: { json: string; html: string };
  e: Evidence;
}) {
  const { session, other, root, origin, title, reports, e } = input;
  assert.notEqual(session.endpoint, other.endpoint);
  assert.notEqual(session.token, other.token);
  const stable = await persistedImagesAndReports(root),
    bytes = await Promise.all(stable.map((entry) => readFile(path.join(root, entry.path)))),
    physical = await wholeLiteralReports(root, title),
    observations = [];
  for (const kind of ["json", "html"] as const) {
    const route = reports[kind],
      type = kind === "json" ? "application/json" : "text/html";
    assert.match(route, /^\/artifacts\/[0-9a-f-]{36}$/);
    assert.ok(physical[kind]);
    const before = await publicArtifact(session, origin, route, type);
    assert.deepEqual(before, physical[kind]);
    await e.save(`trusted-public/report-isolation/${kind}/own-before.bin`, before);
    session.assertRunning();
    other.assertRunning();
    const denied = await fetch(`${other.endpoint}${route}`, {
      redirect: "error",
      signal: AbortSignal.timeout(30000),
      headers: { Origin: origin, Authorization: `Bearer ${other.token}` },
    });
    const body = Buffer.from(await denied.arrayBuffer());
    await e.save(`trusted-public/report-isolation/${kind}/foreign-response.bin`, body);
    assert.equal(denied.status, 404);
    assert.equal(denied.headers.get("Content-Type"), "application/json");
    const after = await publicArtifact(session, origin, route, type);
    await e.save(`trusted-public/report-isolation/${kind}/own-after.bin`, after);
    assert.deepEqual(after, before);
    assert.deepEqual(await persistedImagesAndReports(root), stable);
    assert.deepEqual(
      await Promise.all(stable.map((entry) => readFile(path.join(root, entry.path)))),
      bytes,
    );
    session.assertRunning();
    other.assertRunning();
    observations.push({
      kind,
      route,
      ownStatusBefore: 200,
      otherStatus: denied.status,
      ownStatusAfter: 200,
    });
  }
  return {
    phase: "real-distinct-session-report-isolation",
    observations,
    persistedWholeFilesUnchanged: stable,
    sessionRegistryIsolationOnly: true,
  };
}
