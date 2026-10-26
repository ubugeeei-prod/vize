import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { output } from "./prepare-consumer.mjs";

export async function buildRspack(rspack, configuration, label) {
  const compiler = rspack(configuration);
  let stats;
  try {
    stats = await new Promise((resolve, reject) =>
      compiler.run((error, result) => (error ? reject(error) : resolve(result))),
    );
  } finally {
    await new Promise((resolve, reject) =>
      compiler.close((error) => (error ? reject(error) : resolve())),
    );
  }
  const report = stats.toJson({
    all: false,
    errors: true,
    warnings: true,
    timings: true,
    assets: true,
  });
  await fs.writeFile(
    path.join(output, `${label}-stats.json`),
    JSON.stringify(report, null, 2) + "\n",
  );
  assert.deepEqual(report.errors, []);
  return report;
}

/** Isolate the pinned host's unchanged performance policy from all SFC code. */
export async function verifyDebugAdvisories(rspack, root, bundle, warnings) {
  const entry = path.join(root, "host-authority.js");
  await fs.writeFile(entry, "export {};\n");
  const configuration = {
    context: root,
    mode: "production",
    entry,
    output: { path: path.join(root, "host-authority"), filename: "bundle.js" },
    optimization: { minimize: false },
  };
  const small = await buildRspack(rspack, configuration, "rspack-host-small-authority");
  assert.deepEqual(small.warnings, []);
  const replay = await buildRspack(
    rspack,
    {
      ...configuration,
      plugins: [
        {
          apply(compiler) {
            compiler.hooks.thisCompilation.tap("ReplayUnminifiedAsset", (compilation) => {
              compilation.hooks.processAssets.tap(
                {
                  name: "ReplayUnminifiedAsset",
                  stage: rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT - 1,
                },
                () => {
                  compilation.updateAsset("bundle.js", new rspack.sources.RawSource(bundle));
                },
              );
            });
          },
        },
      ],
    },
    "rspack-host-unminified-asset-authority",
  );
  assert.deepEqual(
    replay.warnings.map(({ code }) => code),
    ["assets over size limit warning", "no async chunks warning"],
  );
  assert.deepEqual(
    warnings,
    replay.warnings,
    "the full debug advisory packet must equal the independent pinned host's packet",
  );
}
