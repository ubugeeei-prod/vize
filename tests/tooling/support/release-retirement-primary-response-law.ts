import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import type { PublicationPlan } from "../../../tools/support/release/public_acceptance/plan.ts";
import { verifyRetirementAbsence } from "../../../tools/support/release/retirement_absence.ts";
import { marketplaceExtension } from "../../../tools/support/release/marketplace_query.ts";

export type Reply = {
  status: number;
  body: unknown;
  rawBody?: string | Uint8Array;
  headers?: Record<string, string>;
  url?: string;
};

/** Replay complete primary bytes while retaining the original provider JSON types. */
export async function retirementPrimaryResponseLaw(
  plan: PublicationPlan,
  replies: Map<string, Reply>,
  primaryTargets: string[],
  fetch: typeof globalThis.fetch,
) {
  const { publisher, name } = plan.editor;
  const primary = JSON.parse(
    fs.readFileSync(
      new URL("../../_fixtures/release/registry-provider-responses.json", import.meta.url),
      "utf8",
    ),
  ) as {
    npm: { raw: string; sha256: string; bytes: number };
    openvsx: { raw: string; sha256: string; bytes: number };
    marketplace: { raw: string; sha256: string; bytes: number };
    latestOnlyControl: { raw: string; sha256: string; bytes: number };
  };
  const originalReplies = primaryTargets.map((url) => replies.get(url)!);
  for (const [index, row] of [primary.npm, primary.openvsx, primary.marketplace].entries()) {
    assert.equal(Buffer.byteLength(row.raw), row.bytes);
    assert.equal(createHash("sha256").update(row.raw).digest("hex"), row.sha256);
    replies.set(primaryTargets[index], {
      status: index === 2 ? 200 : 404,
      body: JSON.parse(row.raw) as unknown,
      rawBody: row.raw,
    });
  }
  try {
    const replay = await verifyRetirementAbsence(plan, { fetch });
    for (const [index, row] of [primary.npm, primary.openvsx, primary.marketplace].entries()) {
      const observation = replay.observations.find((value) => value.url === primaryTargets[index])!;
      assert.deepEqual(observation.response, JSON.parse(row.raw));
      assert.equal(observation.responseSha256, row.sha256);
      assert.equal(observation.responseBytes, row.bytes);
    }
    assert.equal(Buffer.byteLength(primary.latestOnlyControl.raw), primary.latestOnlyControl.bytes);
    assert.equal(
      createHash("sha256").update(primary.latestOnlyControl.raw).digest("hex"),
      primary.latestOnlyControl.sha256,
    );
    const extension = marketplaceExtension(
      `${publisher}.${name}`,
      JSON.parse(primary.latestOnlyControl.raw) as unknown,
    );
    assert.equal(
      (extension.versions as unknown[]).length,
      1,
      "unmarked latest-only response alone cannot prove the request flags; exact query custody is required",
    );
  } finally {
    for (const [index, url] of primaryTargets.entries()) replies.set(url, originalReplies[index]);
  }
}
