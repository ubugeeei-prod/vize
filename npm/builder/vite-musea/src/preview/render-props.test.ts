import assert from "node:assert/strict";
import test from "node:test";
import { runInNewContext } from "node:vm";

import { reactive, shallowRef } from "vue";

import { generatePreviewModule } from "./index.ts";
import type { ArtFileInfo } from "../types/art.ts";
import type { MuseaVueVersion } from "../types/plugin.ts";

const art: ArtFileInfo = {
  path: "/repo/components/Probe.art.vue",
  metadata: { title: "Probe", tags: [], status: "ready" },
  variants: [{ name: "default", template: "<Probe />", isDefault: true, skipVrt: false }],
  hasScriptSetup: false,
  hasScript: false,
  styleCount: 0,
};

const origin = "http://musea.test";

/** Run the generated preview module and return what it hands to Vue after a set-props message. */
function renderAfterSetProps(
  vueVersion: MuseaVueVersion,
  component: unknown,
  payload: { props?: unknown; attrs?: unknown },
): string {
  const handlers = new Map<string, (event: unknown) => void>();
  const errors: unknown[] = [];
  const element = () => ({ addEventListener() {}, appendChild() {}, innerHTML: "", className: "" });
  const h = (_target: unknown, data: unknown) => data;
  let render: (() => unknown) | undefined;
  const createApp = (root: { setup?: () => () => unknown }) => ({
    mount() {
      render = root.setup?.();
    },
    unmount() {},
  });
  class Vue {
    options: { render: (create: typeof h) => unknown };
    constructor(options: { render: (create: typeof h) => unknown }) {
      this.options = options;
    }
    $mount() {
      render = () => this.options.render(h);
    }
    $destroy() {}
  }
  runInNewContext(
    generatePreviewModule(art, "Default", "default", [], null, vueVersion).replace(
      /^import .*$/gm,
      "",
    ),
    {
      window: {
        location: { origin },
        parent: { postMessage() {} },
        addEventListener: (type: string, handler: (event: unknown) => void) =>
          handlers.set(type, handler),
      },
      document: { getElementById: element, createElement: element },
      console: { log() {}, error: (...args: unknown[]) => errors.push(args) },
      artModule: { Default: component, __component__: component },
      createApp,
      Vue,
      reactive,
      shallowRef,
      h,
    },
  );
  handlers.get("message")!({ origin, data: { type: "musea:set-props", payload } });
  assert.deepEqual(errors, []);
  assert.ok(render);
  return JSON.stringify(render());
}

const declared = {
  props: ["label", "onLabel", "once", "dotted.name"],
  mixins: [{ props: { "from-mixin": String } }],
  extends: { props: ["only"] },
};
const kept =
  '"label":"Edited","onLabel":"On","once":true,"dotted.name":1,"from-mixin":"m","only":"o","data-state":"open","__proto__":{"own":true}';
const payload = {
  props: JSON.parse(
    `{${kept},".innerHTML":"<img>","^onclick":"x",".srcdoc":"<p>","innerHTML":"<img>","OuterHTML":"<p>","srcdoc":"<p>","onclick":"x","onClick":"x","ONERROR":"x","on:click":"x","undeclared.name":2,"spaced name":3}`,
  ),
  attrs: JSON.parse(
    '{"aria-label":"Close","class":"wide",".outerHTML":"<p>","^srcdoc":"<p>","onmouseover":"x","SrcDoc":"<p>"}',
  ),
};

void test("the Vue 3 preview renders declared props and plainly named attributes only", () => {
  assert.equal(
    renderAfterSetProps(3, declared, payload),
    `{"aria-label":"Close","class":"wide",${kept}}`,
  );
});

void test("the Vue 2 preview renders declared props and plainly named attributes only", () => {
  const expected = `{"props":{${kept}},"attrs":{"aria-label":"Close","class":"wide"},"scopedSlots":{}}`;
  assert.equal(renderAfterSetProps(2, declared, payload), expected);
  const constructor = Object.assign(function Probe() {}, {
    options: {
      props: { label: {}, onLabel: {}, once: {}, "dotted.name": {}, fromMixin: {}, only: {} },
    },
  });
  assert.equal(renderAfterSetProps(2, constructor, payload), expected);
});

void test("a preview without declared props keeps identifier and kebab-case names", () => {
  assert.equal(
    renderAfterSetProps(
      3,
      {},
      {
        props: JSON.parse(
          '{"label":"a","data-state":"b","_private":"c","$scope":"d","ラベル":"e","onLabel":"f","once":"g",".label":"h","^label":"i","1st":"j"}',
        ),
      },
    ),
    '{"label":"a","data-state":"b","_private":"c","$scope":"d","ラベル":"e"}',
  );
});
