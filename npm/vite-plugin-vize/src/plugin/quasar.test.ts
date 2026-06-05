import assert from "node:assert/strict";
import path from "node:path";

import { patchQuasarBridge } from "./quasar.ts";

const sourcePath = path.resolve(process.cwd(), "target", "vize-tests", "quasar", "App.vue");
const virtualId = `\0${sourcePath}.ts`;
const queriedClientVirtualId = `${sourcePath}.ts?vue&vize`;
const queriedSsrVirtualId = `\0vize-ssr:${sourcePath}.ts?vue&type=template`;
const legacyVirtualId = `\0vize:${sourcePath}.ts`;

{
  let receivedId = "";

  const plugins = [
    {
      name: "vite:quasar:script",
      transform(code: string, id: string) {
        receivedId = id;
        return id.includes(".vue")
          ? {
              code: `import { QBtn } from "quasar";\n${code.replace(/_resolveComponent\("QBtn"\)/g, "QBtn")}`,
              map: null,
            }
          : null;
      },
    },
  ];

  patchQuasarBridge(plugins);
  const result = plugins[0]!.transform!(
    `const _component_QBtn = _resolveComponent("QBtn");`,
    virtualId,
  );

  assert.equal(receivedId, sourcePath);
  assert.ok(result && typeof result === "object");
  assert.match(result.code, /import \{ QBtn \} from "quasar"/);
  assert.doesNotMatch(result.code, /_resolveComponent\("QBtn"\)/);
}

{
  const receivedIds: string[] = [];

  const plugins = [
    {
      name: "vite:quasar:script",
      transform(_code: string, id: string) {
        receivedIds.push(id);
        return null;
      },
    },
  ];

  patchQuasarBridge(plugins);
  plugins[0]!.transform!("export default {}", queriedClientVirtualId);
  plugins[0]!.transform!("export default {}", queriedSsrVirtualId);
  plugins[0]!.transform!("export default {}", legacyVirtualId);

  assert.deepEqual(receivedIds, [
    `${sourcePath}?vue&vize`,
    `${sourcePath}?vue&type=template`,
    sourcePath,
  ]);
}

{
  let receivedId = "";

  const plugins = [
    {
      name: "vite:quasar:script",
      transform(_code: string, id: string) {
        receivedId = id;
        return null;
      },
    },
  ];

  patchQuasarBridge(plugins);
  plugins[0]!.transform!("export default {}", "/project/src/plain.ts");

  assert.equal(receivedId, "/project/src/plain.ts");
}

{
  let callCount = 0;

  const plugins = [
    {
      name: "vite:quasar:script",
      transform() {
        callCount += 1;
        return null;
      },
    },
  ];

  patchQuasarBridge(plugins);
  patchQuasarBridge(plugins);
  plugins[0]!.transform!("export default {}", virtualId);

  assert.equal(callCount, 1);
}

{
  let receivedId = "";

  const plugins = [
    {
      name: "vite:other",
      transform(_code: string, id: string) {
        receivedId = id;
        return null;
      },
    },
  ];

  patchQuasarBridge(plugins);
  plugins[0]!.transform!("export default {}", virtualId);

  assert.equal(receivedId, virtualId);
}

console.log("✅ vite-plugin-vize Quasar bridge tests passed!");
