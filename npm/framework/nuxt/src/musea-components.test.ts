import assert from "node:assert/strict";
import test from "node:test";

import {
  MUSEA_ART_COMPONENT_IGNORE,
  appendMuseaArtComponentIgnore,
  type MuseaNuxtComponentDir,
} from "./musea-components.ts";

void test("Musea component ignore converts string dirs to object dirs", () => {
  const dirs: MuseaNuxtComponentDir[] = ["~/components"];

  appendMuseaArtComponentIgnore(dirs);

  assert.deepEqual(dirs, [
    {
      path: "~/components",
      ignore: [MUSEA_ART_COMPONENT_IGNORE],
    },
  ]);
});

void test("Musea component ignore preserves existing dir options", () => {
  const dirs: MuseaNuxtComponentDir[] = [
    {
      path: "~/components",
      prefix: "Design",
      ignore: ["**/*.stories.vue"],
    },
  ];

  appendMuseaArtComponentIgnore(dirs);

  assert.deepEqual(dirs, [
    {
      path: "~/components",
      prefix: "Design",
      ignore: ["**/*.stories.vue", MUSEA_ART_COMPONENT_IGNORE],
    },
  ]);
});

void test("Musea component ignore does not duplicate the art pattern", () => {
  const dirs: MuseaNuxtComponentDir[] = [
    {
      path: "~/components",
      ignore: [MUSEA_ART_COMPONENT_IGNORE],
    },
  ];

  appendMuseaArtComponentIgnore(dirs);
  appendMuseaArtComponentIgnore(dirs);

  assert.deepEqual(dirs, [
    {
      path: "~/components",
      ignore: [MUSEA_ART_COMPONENT_IGNORE],
    },
  ]);
});
