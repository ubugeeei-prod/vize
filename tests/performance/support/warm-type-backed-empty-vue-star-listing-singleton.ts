import {
  emptyVueStarBodyPairs,
  emptyVueStarMetadataPairs,
} from "./warm-type-backed-empty-vue-star-host.ts";
import {
  emptyVueStarListingPath,
  emptyVueStarListingBeforeSHA256,
  emptyVueStarListingAfterSHA256,
} from "./warm-type-backed-empty-vue-star-listing.ts";

const retainedHelpers = [
  [
    "tests/performance/support/warm-type-backed-empty-vue-star-host.ts",
    "24ada83ae6f7e648f88502d627a798f66bd81a1f0e9ef6c3cf0ce7e7c909102f",
  ],
  [
    "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts",
    "3dc6a16c6e9869302858df257d21e2e2b03acdc13606460bd7849d9d9ce28fc2",
  ],
] as const;
const listingHelper = "tests/performance/support/warm-type-backed-empty-vue-star-listing.ts";
const listingHelperSHA = "6e7941072788c2fad4e678d5edd6d4abb5b8e3b87ad53b85c198a6e78d8af801";

/** The existing migration is already present; qualify only this whole listing correction. */
export function qualifyEmptyVueStarListingSingleton(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: (side: "before" | "after", file: string) => string | null,
) {
  if (
    production.length !== 1 ||
    production[0] !== emptyVueStarListingPath ||
    alreadyQualified.has(emptyVueStarListingPath)
  )
    return null;
  for (const [file, expected] of retainedHelpers)
    if (digest("before", file) !== expected || digest("after", file) !== expected) return null;
  if (
    digest("before", listingHelper) !== null ||
    digest("after", listingHelper) !== listingHelperSHA
  )
    return null;
  const pairs = [...emptyVueStarBodyPairs, ...emptyVueStarMetadataPairs];
  for (const [file, , current] of pairs) {
    const before = file === emptyVueStarListingPath ? emptyVueStarListingBeforeSHA256 : current;
    const after = file === emptyVueStarListingPath ? emptyVueStarListingAfterSHA256 : current;
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  }
  return {
    authority: "exact-reviewed-current-star-listing-singleton",
    files: [emptyVueStarListingPath],
    completeBodyPairs: 1,
    retainedCurrentBodyPairs: pairs.length - 1,
    retainedHelperBodies: retainedHelpers.length,
    runtimeCredit: false,
    performanceCredit: false,
  };
}
