import assert from "node:assert/strict";
import type { TestContext } from "node:test";
import { verifyPublication as verify } from "../../../tools/support/release/public_acceptance/registry.ts";
import {
  marketplaceQuery,
  marketplaceQueryUrl,
  marketplaceQueryAccept,
} from "../../../tools/support/release/marketplace_query.ts";
import { publicFixture, sha256 } from "./release-public-acceptance-fixtures.ts";

/** Complete inert archive and channel receipt law; no public availability claim. */
export async function publicArchiveReceiptLaw(t: TestContext) {
  const f = publicFixture(t);
  const result = await verify(f.plan, f.options);
  assert.equal(result.success, true);
  assert.equal(result.source.cut, f.cut);
  assert.equal(result.source.head, f.head);
  assert.equal(result.npm.length, f.plan.npm.length);
  assert.equal(result.crates.length, f.plan.crates.length);
  for (const item of [...result.npm, ...result.crates])
    assert.equal(item.sha256, sha256(f.responses.get(item.url) as Buffer));
  assert.equal(result.marketplace.exactVersionAvailable, true);
  assert.deepEqual(result.marketplace.response, f.responses.get(f.marketplaceUrl));
  assert.equal(
    result.marketplace.responseSha256,
    sha256(JSON.stringify(f.responses.get(f.marketplaceUrl))),
  );
  assert.equal(result.openVsx.status, "available");
  assert.equal(
    result.githubAssets[0].sha256,
    sha256(f.responses.get(result.githubAssets[0].url) as Buffer),
  );
  assert.match(result.evidence, /no signature or installed-product execution claim/);
  for (const call of f.calls) {
    assert.equal(call.init?.credentials, "omit");
    assert.equal(new Headers(call.init?.headers).has("authorization"), false);
    if (call.url === marketplaceQueryUrl) {
      assert.equal(call.init?.method, "POST");
      assert.ok(typeof call.init?.body === "string");
      assert.deepEqual(
        JSON.parse(call.init.body),
        marketplaceQuery(`${f.plan.editor.publisher}.${f.plan.editor.name}`),
      );
      assert.equal(new Headers(call.init?.headers).get("accept"), marketplaceQueryAccept);
    } else assert.ok(!call.init?.method || call.init.method === "GET");
    assert.doesNotMatch(call.url, /\/latest(?:[/?]|$)/);
  }
}
