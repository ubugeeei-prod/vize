/** --quiet removes progress; --show-virtual-ts still emits exact source dumps. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { hash } from "./canon-instance-directives-io.ts";
import { packet, isRecord } from "./canon-instance-directives-model.ts";
import {
  applyCurrentOverloadPreamble,
  applyIntegratedSourcePreambleTransitions,
  applyPreviousOverloadPreamble,
  applySourceOwnedPreambleTransitions,
} from "./canon-slot-provider-preamble.ts";
import { applyCurrentSlotPreamble } from "./canon-slot-provider-current-preamble.ts";
import { applyDiscriminatedJsxUnionIn } from "../../support/compat/content-mapper/discriminated-jsx-union-transition.ts";
import {
  applyCanonLeftoversIn,
  applyStrictExtraKeysIn,
  beforeCanonLeftovers,
} from "../../support/compat/content-mapper/strict-extra-keys-transition.ts";
import { applyEmitMemberInferIn } from "../../support/compat/content-mapper/emit-member-infer-transition.ts";
import { applyInstanceInputKeysIn } from "../../support/compat/content-mapper/instance-input-keys-transition.ts";
import { applyUnownedOwnedKeysIn } from "../../support/compat/content-mapper/unowned-owned-keys-transition.ts";
import { applyPropValueOrIn } from "../../support/compat/content-mapper/prop-value-or-transition.ts";
import { applyListenerContractIn } from "../../support/compat/content-mapper/listener-contract-transition.ts";
import { applySharedTemplateRefUnwrapIn } from "../../support/compat/content-mapper/shared-template-ref-unwrap-transition.ts";
import { applySharedUsageHelpersIn } from "../../support/compat/content-mapper/shared-usage-helpers-transition.ts";

export const canonLeftoverPreamblePaths = [
  "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
  "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-emit-props.d.ts.txt",
] as const;

export function applyCanonLeftoverPreamble(previous: string) {
  return canonLeftoverPreamblePaths.reduce(
    (current, file) => applyCanonLeftoversIn(file, current),
    previous,
  );
}

export function slotCliPacket(
  process: { stdout: Buffer; stderr: Buffer; status: number | null },
  project: string,
) {
  const observed = packet(process.stdout, process.status, project);
  const bytes = fs.readFileSync(
    path.resolve(
      "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/slot-provider-payload-update-11027.json",
    ),
  );
  assert.equal(hash(bytes), "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c");
  const source: unknown = JSON.parse(bytes.toString("utf8"));
  assert.ok(isRecord(source) && isRecord(source.logicalPreamble));
  const helper = source.logicalPreamble;
  assert.equal(typeof helper.afterWholeUtf8, "string");
  const text = helper.afterWholeUtf8 as string;
  assert.equal(Buffer.byteLength(text), helper.afterBytes);
  assert.equal(hash(text), helper.afterSha256);
  // Preserve the historical slot evidence. Advance current expectations only
  // through the authenticated source-owned helper transitions.
  const prefix = "type $OverloadUnionRecursive<";
  const historical = text.split("\n").filter((line) => line.startsWith(prefix));
  const emitOverloadsPath =
    "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt";
  const emitOverloads = fs.readFileSync(emitOverloadsPath, "utf8");
  const declarations = beforeCanonLeftovers(emitOverloadsPath, emitOverloads)
    .split("\n")
    .filter(Boolean);
  const current = emitOverloads.split("\n").filter((line) => line.startsWith(prefix));
  assert.equal(historical.length, 1);
  assert.equal(current.length, 1);
  const previous = historical[0],
    replacement = current[0];
  assert.ok(previous !== undefined && replacement !== undefined);
  assert.equal(text.split(previous).length, 2);
  for (const declaration of declarations)
    if (declaration !== replacement) assert.equal(text.split(declaration).length, 2);
  const certifiedPreamble = applyCurrentSlotPreamble(
    applySourceOwnedPreambleTransitions(applyPreviousOverloadPreamble(text)),
  );
  const currentPreamble = applySharedUsageHelpersIn(
    applySharedTemplateRefUnwrapIn(
      applyListenerContractIn(
        applyPropValueOrIn(
          applyUnownedOwnedKeysIn(
            applyInstanceInputKeysIn(
              applyCanonLeftoverPreamble(
                applyEmitMemberInferIn(
                  applyStrictExtraKeysIn(
                    applyDiscriminatedJsxUnionIn(
                      applyIntegratedSourcePreambleTransitions(
                        applyCurrentOverloadPreamble(certifiedPreamble),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
  assert.equal(currentPreamble.split(replacement).length, 2);
  const files = observed.result.files.filter((file) => typeof file.virtualTs === "string");
  assert.ok(files.length > 0, "all actual virtual file bodies must remain in JSON");
  const expected =
    `\n=== __vize_helpers.d.ts ===\n${currentPreamble}\n` +
    files
      .map((file) => `\n=== ${path.resolve(project, file.file)} ===\n${file.virtualTs}\n`)
      .join("");
  assert.equal(
    process.stderr.equals(Buffer.from(expected)),
    true,
    "complete CLI stderr must equal its source-backed helper and JSON virtual-file dumps",
  );
  return observed;
}
