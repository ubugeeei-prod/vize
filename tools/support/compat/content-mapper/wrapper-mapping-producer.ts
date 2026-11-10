/** The #11675 producer adds only diagnostic-owned delimiter atoms. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { CHECK_MODE_PRODUCER_SUCCESSORS } from "./check-mode-successor.ts";

export function assertWrapperMappingProducer(producer: Uint8Array): void {
  const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
  assert.equal(
    digest(producer),
    "a2e22c9ee38c9eb77150b805e8e5e09f086f05041940eaaaf0dfea51e1aec6a9",
    "unqualified #11675 wrapper mapping producer",
  );
  let historical = Buffer.from(producer).toString("utf8");
  for (const [successor, predecessor] of CHECK_MODE_PRODUCER_SUCCESSORS) {
    assert.equal(historical.split(successor).length, 2);
    historical = historical.replace(successor, predecessor);
  }
  assert.equal(
    digest(historical),
    "271dfc983bf196cd118b521e7ac401824b19c423d2cacfa32dcfec29bdec922b",
    "certified check-mode wrapper successor",
  );
  for (const [successor, expected, message] of [
    [
      "    span_features::retain_component_prop_hover_owner(generated, &mut spans);\n",
      "9c3faec15ca7c73c4a8a3f0fd1fcae7bb6f9d70c8e71a7046994d373e8b4e038",
      "certified component-hover owner wrapper successor",
    ],
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
