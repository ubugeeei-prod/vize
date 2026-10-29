import assert from "node:assert/strict";

type ManifestCase = {
  id: string;
  dialectCoverage?: unknown;
  [key: string]: unknown;
};

const promotions = new Map<string, unknown>([
  [
    "formatter/sfc/plain-style-nested-comments",
    {
      state: "source-inspected",
      registryRef: null,
      evidence: [
        {
          kind: "source-inspection",
          dialects: ["vue3-sfc", "html-template", "css-scoped"],
          selector: "default Vue version with --no-config",
        },
      ],
    },
  ],
  [
    "formatter/sfc/split-v-pre-indentation",
    {
      state: "source-inspected",
      registryRef: null,
      evidence: [
        {
          kind: "source-inspection",
          dialects: ["vue3-sfc", "html-template", "v-pre"],
          selector: "default Vue version with --no-config",
        },
      ],
    },
  ],
]);

export function assertPreservedFormatterCase(
  current: ManifestCase | undefined,
  original: ManifestCase,
) {
  const promotion = promotions.get(original.id);
  if (!promotion) {
    assert.deepEqual(current, original, "original formatter cases changed");
    return;
  }
  assert(current, `${original.id}: live formatter case is required`);
  const { dialectCoverage, ...currentCustody } = current;
  const { dialectCoverage: _originalCoverage, ...originalCustody } = original;
  assert.deepEqual(currentCustody, originalCustody, "original formatter case custody changed");
  assert.deepEqual(dialectCoverage, promotion, "dialect measurement promotion changed");
}
