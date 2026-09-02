import path from "node:path";
import { test } from "node:test";

import { assertS0AliasConsumer, repoRoot } from "./support/davinci-stage-dependencies.ts";

test("Glyph formatter imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_glyph",
    label: "Glyph formatter",
    directory: path.join(repoRoot, "crates", "vize_glyph"),
  });
});
