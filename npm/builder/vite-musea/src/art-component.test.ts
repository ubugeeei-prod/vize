import assert from "node:assert/strict";
import test from "node:test";
import { inferArtComponent, resolveArtComponent } from "./art-component.ts";
import { buildVariantSfcSource } from "./art-variant-sfc.ts";
import type { ArtFileInfo } from "./types/art.ts";

void test("inline art uses the host component's filename binding", () => {
  const art: ArtFileInfo = {
    path: "/project/Button.vue",
    componentPath: "/project/Button.vue",
    isInline: true,
    metadata: { title: "Button", tags: [], status: "ready" },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
  const component = resolveArtComponent(art, art.path, null);
  assert.equal(component.componentBindingName, component.componentTagName);
  const source = buildVariantSfcSource(
    art,
    "<Button>Default Button</Button>",
    "Default",
    component,
  );
  assert.match(source, /import Button from "\/project\/Button\.vue"/u);
  assert.match(source, /<Button>Default Button<\/Button>/u);
  assert.doesNotMatch(source, /__MuseaComponent/u);
});

void test("standalone art infers one imported component shared by every variant", () => {
  const inferred = inferArtComponent(
    {
      variants: [
        { name: "Default", template: "<Counter />", isDefault: true, skipVrt: false },
        {
          name: "WithIcon",
          template: "<Counter><Icon /></Counter>",
          isDefault: false,
          skipVrt: false,
        },
      ],
    },
    ['import Counter from "./Counter.vue"', 'import Icon from "./Icon.vue"'],
  );
  assert.deepEqual(inferred, {
    localName: "Counter",
    source: "./Counter.vue",
    statement: 'import Counter from "./Counter.vue"',
  });
});

void test("standalone art diagnoses multiple target candidates", () => {
  assert.throws(
    () =>
      inferArtComponent(
        {
          variants: [
            {
              name: "Composition",
              template: "<Counter /><Meter />",
              isDefault: true,
              skipVrt: false,
            },
          ],
        },
        ['import Counter from "./Counter.vue"', 'import Meter from "./Meter.vue"'],
      ),
    /cannot infer one target component \(Counter, Meter\).*defineArt\(Target\)/u,
  );
});
