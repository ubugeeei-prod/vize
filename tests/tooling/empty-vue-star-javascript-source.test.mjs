import { runStarListingOriginalTest } from "../../tools/support/compat/github/empty-vue-star-listing-source.mjs";

// The complete committed current law precedes this unchanged literal source law.
await runStarListingOriginalTest(
  new URL("../../", import.meta.url),
  "tests/tooling/empty-vue-star-javascript-source.test.mjs",
);
