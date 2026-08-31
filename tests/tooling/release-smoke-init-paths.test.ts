import assert from "node:assert/strict";
import { test } from "node:test";

import { reportedDiagnostics } from "../../legacy-tools/npm/smoke-release-init-project.mjs";

test("reported diagnostics are normalized relative to the fresh project", () => {
  const diagnostics = ["error:1:1 [TS2322] Type mismatch."];
  assert.deepEqual(
    reportedDiagnostics(
      {
        files: [
          {
            file: "C:\\Users\\runneradmin\\AppData\\Local\\Temp\\vize\\fresh\\app\\src\\App.vue",
            diagnostics,
          },
          { file: "src\\components\\HelloWorld.vue", diagnostics },
          { file: "src/clean.vue", diagnostics: [] },
        ],
      },
      "C:\\Users\\runneradmin\\AppData\\Local\\Temp\\vize\\fresh\\app",
    ),
    [
      { file: "src/App.vue", diagnostics },
      { file: "src/components/HelloWorld.vue", diagnostics },
    ],
  );
  assert.deepEqual(
    reportedDiagnostics(
      {
        files: [{ file: "/tmp/vize/fresh/app/src/App.vue", diagnostics }],
      },
      "/tmp/vize/fresh/app",
    ),
    [{ file: "src/App.vue", diagnostics }],
  );
  assert.deepEqual(
    reportedDiagnostics(
      {
        files: [
          {
            file: "../../../../../../../runneradmin/AppData/Local/Temp/vize/fresh/app/src/App.vue",
            diagnostics,
          },
        ],
      },
      "C:\\Users\\runneradmin\\AppData\\Local\\Temp\\vize\\fresh\\app",
    ),
    [{ file: "src/App.vue", diagnostics }],
  );
  assert.deepEqual(
    reportedDiagnostics(
      {
        files: [
          {
            file: [
              "C:/Users/runneradmin/AppData/Local/Temp",
              "vize-release-smoke-LsASc9/fresh/vite-vue-ts-npm/src/App.vue",
            ].join("/"),
            diagnostics,
          },
          {
            file: [
              "C:/Users/runneradmin/AppData/Local/Temp",
              "vize-release-smoke-LsASc9/fresh/vite-vue-ts-npm/src/components/HelloWorld.vue",
            ].join("/"),
            diagnostics,
          },
        ],
      },
      "C:\\a\\_temp\\vize-release-smoke-LsASc9\\fresh\\vite-vue-ts-npm",
    ),
    [
      { file: "src/App.vue", diagnostics },
      { file: "src/components/HelloWorld.vue", diagnostics },
    ],
  );
  assert.deepEqual(
    reportedDiagnostics(
      {
        files: [
          {
            file: "C:/Users/runneradmin/AppData/Local/Temp/fresh/vite-vue-ts-npm/src/App.vue",
            diagnostics,
          },
        ],
      },
      "C:\\a\\_temp\\fresh\\vite-vue-ts-npm",
    ),
    [
      {
        file: "C:/Users/runneradmin/AppData/Local/Temp/fresh/vite-vue-ts-npm/src/App.vue",
        diagnostics,
      },
    ],
  );
  assert.deepEqual(
    reportedDiagnostics(
      {
        files: [{ file: "../other/src/App.vue", diagnostics }],
      },
      "/tmp/vize/fresh/app",
    ),
    [{ file: "../other/src/App.vue", diagnostics }],
  );
});
