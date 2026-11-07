/** The #11675 producer adds only diagnostic-owned delimiter atoms. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export function assertWrapperMappingProducer(producer: Uint8Array): void {
  const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
  assert.equal(
    digest(producer),
    "9c3faec15ca7c73c4a8a3f0fd1fcae7bb6f9d70c8e71a7046994d373e8b4e038",
    "unqualified #11675 wrapper mapping producer",
  );
  let historical = Buffer.from(producer).toString("utf8");
  for (const [successor, expected, message] of [
    [
      "    span_features::retain_template_hover_owner(generated, &mut spans);\n",
      "6a060e4d0c5d16cd3071f76ca35558abcecde9fcd5458346d54287eacdef03fd",
      "certified template-hover owner wrapper successor",
    ],
    [
      "            template_use_navigation: true,\n",
      "d51c0acbed93e899ec5b634825bd3d7d2ffb2d47908f5bbc575e68feb909a937",
      "certified template-use projection wrapper successor",
    ],
  ] as const) {
    assert.equal(historical.split(successor).length, 2);
    historical = historical.replace(successor, "");
    assert.equal(digest(historical), expected, message);
  }
  const localOwnershipSuccessor = "            local_component_ownership_helpers: false,\n";
  assert.equal(historical.split(localOwnershipSuccessor).length, 2);
  historical = historical.replace(localOwnershipSuccessor, "");
  assert.equal(
    digest(historical),
    "f75096741294e3f99e5fedf6accad2daf49ee402c72af2bc684b20b0f4310e20",
    "certified local-ownership wrapper successor",
  );
  for (const addition of [
    "mod component_export;\n",
    "    component_export::insert_spans(&mut spans, mappings, metadata);\n",
  ]) {
    assert.equal(historical.split(addition).length, 2);
    historical = historical.replace(addition, "");
  }
  assert.equal(
    digest(historical),
    "25abafce233364f0b11f71202d32f87290ef7d231f939063938f8649e3448b67",
    "certified component-export wrapper successor",
  );
}
