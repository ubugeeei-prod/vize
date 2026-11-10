import assert from "node:assert/strict";

import { listenerWrapperReference } from "./listener-wrapper-reference.js";
import { publicInstanceReference } from "./public-instance-reference.js";

const prefix =
  "The expected type comes from property 'fooBar' which is declared here on type 'ComponentPropsForTemplate<";
const tail =
  ", VueTemplatePublicAttrsType & { string?: unknown; contextmenu?: unknown; onAbort?: unknown; onAnimationcancel?: unknown; onAnimationend?: unknown; ";
const before = `${prefix}__Child_CheckProps_0${tail}... 677 more ...; webpreferences?: unknown; } & { ...; }>'`;
const after = `${prefix}__VCheckProps0${tail}onAnimationiteration?: unknown; ... 676 more ...; webpreferences?: unknown; } & { ...; }>'`;

// The archived packets keep their bytes. Canon now spells the whole-props
// target `__VCheckProps0`; the shorter name leaves room for one more printed
// member before TypeScript truncates the same type. Only that related message
// of the feature-boundary diagnostic moves.
export function compactContractNameReference(packets) {
  const out = structuredClone(packets);
  const matches = out.filter((row) => row.name === "feature-boundary-diagnostics");
  assert.equal(matches.length, 1, "one closed feature-boundary packet");
  const related = matches[0].result
    .flatMap((diagnostic) => diagnostic.relatedInformation ?? [])
    .filter((row) => row.messageText === before);
  assert.equal(related.length, 1, "one archived whole-props target message");
  related[0].messageText = after;
  return out;
}

/** Every reviewed successor of the archived packets, oldest first. */
export function currentPacketReference(packets) {
  return compactContractNameReference(listenerWrapperReference(publicInstanceReference(packets)));
}
