import { runStarListingOriginalTest } from "../../tools/support/compat/github/empty-vue-star-listing-source.mjs";

// The complete committed current law precedes this unchanged literal source law.
await runStarListingOriginalTest(
  new URL("../../", import.meta.url),
  "tests/tooling/warm-type-backed-empty-vue-star-listing.test.ts",
);
