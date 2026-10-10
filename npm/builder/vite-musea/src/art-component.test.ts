import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { resolveArtComponent, expandSelfTag } from "./art-component.ts";
import { buildVariantSfcSource } from "./art-variant-sfc.ts";
import type { ArtFileInfo } from "./types/art.ts";

void test("inline Self uses the binding actually imported into its synthetic SFC", () => {
  const fixture = JSON.parse(
    readFileSync(
      new URL(
        "../../../../tests/_fixtures/differential/musea/inline-self-binding.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as { filename: string; template: string; variant: string };
  const art: ArtFileInfo = {
    path: fixture.filename,
    componentPath: fixture.filename,
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
    expandSelfTag(fixture.template, component.componentTagName),
    fixture.variant,
    component,
  );
  assert.match(source, /import MuseaComponent from "\/project\/Button\.vue"/u);
  assert.match(source, /<MuseaComponent>Default Button<\/MuseaComponent>/u);
  assert.doesNotMatch(source, /__MuseaComponent/u);
});
