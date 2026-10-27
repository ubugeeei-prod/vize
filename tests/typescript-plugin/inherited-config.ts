import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { startTsserver } from "./tsserver-client.mjs";
import fixtureLoader from "./project-fixture.cjs";
import { createOrdinaryHostFrontier } from "./ordinary-host-frontier.ts";
import type { NativeApi, TsserverClient } from "./project-metadata.ts";

export function createInheritedConfig(
  consumer: string,
  src: string,
  output: string,
  typescriptVersion: string,
) {
  const { files, expected, pack } = fixtureLoader.readProjectFixture();
  const base = path.join(consumer, "inherited.json"),
    root = path.join(consumer, "tsconfig.json");
  const sourceFile = path.join(src, "Inherited.vue");
  const source = files.get("src/Inherited.vue.txt");
  const control = path.join(src, "inherited-control.ts");
  const controlSource = files.get("src/control.ts.txt");
  const before = JSON.parse(files.get("inherited-before.json.txt"));
  const after = JSON.parse(files.get("inherited-after.json.txt"));
  fs.writeFileSync(
    root,
    files.get(`tsconfig-${typescriptVersion.startsWith("6.") ? "6.0" : "5.7"}.json.txt`),
  );
  fs.writeFileSync(base, files.get("inherited-before.json.txt"));
  fs.writeFileSync(sourceFile, source);
  fs.writeFileSync(control, controlSource);
  const digest = (data: string | Buffer) => createHash("sha256").update(data).digest("hex");
  const fixed = [root, sourceFile, control].map((file) => ({
    file,
    bytes: fs.readFileSync(file),
    sha256: digest(fs.readFileSync(file)),
  }));
  fs.writeFileSync(
    path.join(output, "inherited-inputs.json"),
    JSON.stringify(
      {
        root: fs.readFileSync(root, "utf8"),
        source,
        controlSource,
        before,
        after,
        fixture: pack,
        fixed: fixed.map(({ file, sha256 }) => ({ file, sha256 })),
      },
      null,
      2,
    ),
  );
  return { root, base, sourceFile, source, control, controlSource, before, after, fixed, expected };
}

export async function checkInheritedConfig(
  client: TsserverClient,
  fixture: ReturnType<typeof createInheritedConfig>,
  native: NativeApi,
  tsserver: string,
  consumer: string,
  output: string,
  results: Record<string, unknown>,
) {
  const packet: Record<string, any> = {
    source: fixture.source,
    beforeProjection: null,
    afterProjection: null,
    before: null,
    after: null,
    fresh: null,
    controlBefore: null,
    controlAfter: null,
    metadata: [],
    cases: [],
  };
  const save = () =>
    fs.writeFileSync(
      path.join(output, "inherited-observations.json"),
      JSON.stringify(packet, null, 2),
    );
  const project = (source: string) =>
    native.generateVueLanguageCode(source, {
      filename: fixture.sourceFile,
      configFileName: fixture.root,
      checkJavaScript: false,
      preserveUnusedDiagnostics: true,
    });
  const request = async (server: TsserverClient, key: string, command: string, file: string) => {
    const response = await server.send(command, { file });
    packet[key] = response;
    results["inherited-" + key] = response;
    save();
    assert.equal(response.success, true, JSON.stringify(response));
    return response;
  };
  await client.send(
    "open",
    { file: fixture.sourceFile, fileContent: fixture.source, scriptKindName: "TS" },
    false,
  );
  await client.send("open", { file: fixture.control, fileContent: fixture.controlSource }, false);
  packet.beforeProjection = JSON.parse(project(fixture.source));
  packet.metadata.push(native.getVueLanguageProjectMetadata(fixture.root));
  save();
  await request(client, "before", "semanticDiagnosticsSync", fixture.sourceFile);
  await request(client, "controlBefore", "semanticDiagnosticsSync", fixture.control);
  fs.writeFileSync(fixture.base, JSON.stringify(fixture.after));
  for (const file of fixture.fixed)
    assert.deepEqual(fs.readFileSync(file.file), file.bytes, "only the inherited config changes");
  packet.afterProjection = JSON.parse(project(fixture.source));
  packet.metadata.push(native.getVueLanguageProjectMetadata(fixture.root));
  save();
  await client.send("reloadProjects", {}, false);
  await request(client, "controlAfter", "semanticDiagnosticsSync", fixture.control);
  await request(client, "after", "semanticDiagnosticsSync", fixture.sourceFile);
  const freshDirectory = path.join(output, "inherited-fresh-server");
  fs.mkdirSync(freshDirectory);
  const fresh = startTsserver(tsserver, consumer, freshDirectory);
  try {
    await fresh.send("configure", {
      extraFileExtensions: [
        { extension: ".vue", isMixedContent: true, scriptKind: 7 },
        { extension: ".art", isMixedContent: true, scriptKind: 7 },
      ],
    });
    await fresh.send(
      "open",
      { file: fixture.sourceFile, fileContent: fixture.source, scriptKindName: "TS" },
      false,
    );
    await request(fresh, "fresh", "semanticDiagnosticsSync", fixture.sourceFile);
  } finally {
    await fresh.stop();
    save();
  }
  assert.deepEqual(packet.before.body, [], "unchecked unknown components are initially admitted");
  assert.deepEqual(packet.controlBefore.body, []);
  assert.deepEqual(packet.fresh.body, fixture.expected.strict);
  assert.deepEqual(packet.controlAfter.body, fixture.expected.controlAfter);
  assert(
    packet.controlAfter.body.some((diagnostic: { code: number }) => diagnostic.code === 7029),
    "the real TypeScript project consumed the changed extends file",
  );
  assert.notEqual(
    packet.beforeProjection.text,
    packet.afterProjection.text,
    "the native producer independently consumes effective inherited Vue options",
  );
  assert(
    packet.fresh.body.length > 0,
    "a fresh actual tsserver enforces the changed strict template option",
  );
  assert.deepEqual(
    packet.after.body,
    packet.fresh.body,
    "the warm server must refresh the same full Vue diagnostic packet after project reload",
  );
  const packageRoot = path.join(consumer, "node_modules/vize-config-priority");
  fs.mkdirSync(packageRoot);
  fs.writeFileSync(path.join(packageRoot, "package.json"), '{"tsconfig":"preferred"}');
  fs.writeFileSync(path.join(packageRoot, "tsconfig.json"), JSON.stringify(fixture.before));
  const preferred = path.join(packageRoot, "preferred.json");
  const hostFrontier = await createOrdinaryHostFrontier(fixture, consumer, tsserver, output);
  const states = [
    [
      "package-fallback",
      () => fs.writeFileSync(fixture.base, '{"extends":"vize-config-priority"}'),
      fixture.before,
    ],
    [
      "preferred-created",
      () => fs.writeFileSync(preferred, JSON.stringify(fixture.after)),
      fixture.after,
    ],
    ["preferred-removed", () => fs.unlinkSync(preferred), fixture.before],
    ["malformed-parent", () => fs.writeFileSync(preferred, "{ invalid JSON"), null],
    [
      "parent-repaired",
      () => fs.writeFileSync(preferred, JSON.stringify(fixture.after)),
      fixture.after,
    ],
    [
      "ordinary-options-only",
      () =>
        fs.writeFileSync(
          preferred,
          JSON.stringify({ ...fixture.after, compilerOptions: fixture.before.compilerOptions }),
        ),
      { ...fixture.after, compilerOptions: fixture.before.compilerOptions },
    ],
  ];
  try {
    for (const [name, edit, options] of states) {
      edit();
      for (const file of fixture.fixed)
        assert.deepEqual(
          fs.readFileSync(file.file),
          file.bytes,
          "root config and both authored files remain byte-identical",
        );
      const observation: Record<string, any> = {
        name,
        options,
        base: fs.readFileSync(fixture.base, "utf8"),
        preferred: fs.existsSync(preferred) ? fs.readFileSync(preferred, "utf8") : null,
      };
      packet.cases.push(observation);
      try {
        observation.metadata = native.getVueLanguageProjectMetadata(fixture.root);
        packet.metadata.push(observation.metadata);
        observation.projection = JSON.parse(project(fixture.source));
      } catch (cause) {
        observation.nativeError = (cause as Error).message;
      }
      save();
      await client.send("reloadProjects", {}, false);
      observation.control = await client.send("semanticDiagnosticsSync", { file: fixture.control });
      observation.warm = await client.send("semanticDiagnosticsSync", { file: fixture.sourceFile });
      results[`inherited-${name}`] = observation.warm;
      save();
      const directory = path.join(output, `inherited-fresh-${name}`);
      fs.mkdirSync(directory);
      const next = startTsserver(tsserver, consumer, directory);
      try {
        await next.send("configure", {
          extraFileExtensions: [{ extension: ".vue", isMixedContent: true, scriptKind: 7 }],
        });
        await next.send(
          "open",
          { file: fixture.sourceFile, fileContent: fixture.source, scriptKindName: "TS" },
          false,
        );
        observation.fresh = await next.send("semanticDiagnosticsSync", {
          file: fixture.sourceFile,
        });
        await next.send(
          "open",
          { file: fixture.control, fileContent: fixture.controlSource },
          false,
        );
        observation.freshControl = await next.send("semanticDiagnosticsSync", {
          file: fixture.control,
        });
      } finally {
        await next.stop();
        save();
      }
      assert.equal(observation.warm.success, true);
      assert.equal(observation.fresh.success, true);
      assert.deepEqual(
        observation.warm.body,
        observation.fresh.body,
        `complete warm/fresh diagnostics for ${name}`,
      );
      if (options) {
        assert.equal(observation.nativeError, undefined);
        assert.deepEqual(
          observation.warm.body,
          options.vueCompilerOptions.strictTemplates ? packet.fresh.body : [],
        );
        if (name === "package-fallback")
          assert(observation.metadata.watchedFiles.includes(preferred));
      } else {
        assert.match(observation.nativeError, /Invalid tsconfig/);
        assert(
          observation.warm.body.some((diagnostic: { text: string }) =>
            diagnostic.text.includes("Invalid tsconfig"),
          ),
        );
      }
      // The ordinary package-option frontier is a separately retained failure,
      // compared to unmodified workers, outside the Vue qualification role.
      await hostFrontier.observe(
        name,
        observation.control,
        observation.freshControl,
        options
          ? options.compilerOptions.noFallthroughCasesInSwitch
            ? fixture.expected.controlAfter
            : fixture.expected.controlBefore
          : undefined,
      );
    }
    const repaired = packet.cases.find((item: { name: string }) => item.name === "parent-repaired");
    const ordinary = packet.cases.find(
      (item: { name: string }) => item.name === "ordinary-options-only",
    );
    assert.equal(ordinary.metadata.configIdentity, repaired.metadata.configIdentity);
    assert.deepEqual(
      ordinary.projection,
      repaired.projection,
      "ordinary TS options do not change the whole native projection",
    );
    fs.writeFileSync(fixture.base, JSON.stringify(fixture.after));
    save();
  } finally {
    await hostFrontier.stop();
    save();
  }
  return {
    role: "vue-language-project-configuration",
    qualified: true,
    ordinaryHostFrontier: hostFrontier.summary(),
  };
}

export function checkConfigurationLog(output: string) {
  const observations = JSON.parse(
    fs.readFileSync(path.join(output, "inherited-observations.json"), "utf8"),
  );
  const marker = "[vize] TypeScript project configuration ";
  const configurations = fs
    .readFileSync(path.join(output, "tsserver.log"), "utf8")
    .split("\n")
    .filter((line) => line.includes(marker))
    .map((line) => JSON.parse(line.slice(line.indexOf(marker) + marker.length)));
  fs.writeFileSync(
    path.join(output, "configuration-watches.json"),
    JSON.stringify(configurations, null, 2),
  );
  for (const metadata of observations.metadata) {
    const actual = configurations.find((value) => value.configIdentity === metadata.configIdentity);
    assert(actual, `the real project registered canonical snapshot ${metadata.configIdentity}`);
    assert.deepEqual(actual.nativeWatchedFiles, metadata.watchedFiles);
    assert.deepEqual(new Set(actual.registeredFiles), new Set(metadata.watchedFiles));
    assert.equal(
      actual.registeredFiles.length,
      metadata.watchedFiles.length,
      "exactly one live watch per canonical dependency",
    );
  }
}

export function checkAutoTargetMetadata(native: NativeApi, consumer: string, output: string) {
  const root = path.join(consumer, "自動設定");
  fs.mkdirSync(root);
  fs.writeFileSync(path.join(root, "package.json"), '{"name":"auto-target-fixture"}');
  const config = path.join(root, "tsconfig.json");
  const source = '<script setup lang="ts"></script><template><div x-small /></template>';
  const capture = () => ({
    metadata: native.getVueLanguageProjectMetadata(config),
    raw: native.generateVueLanguageCode(source, {
      filename: path.join(root, "Source.vue"),
      configFileName: config,
    }),
  });
  const preferred = path.join(root, "node_modules/vue/package.json");
  const installed = path.join(consumer, "node_modules/vue/package.json");
  const observations: Record<string, any> = {
    source,
    installed: JSON.parse(fs.readFileSync(installed, "utf8")),
  };
  const save = () =>
    fs.writeFileSync(
      path.join(output, "auto-target-metadata.json"),
      JSON.stringify(observations, null, 2),
    );
  fs.writeFileSync(config, '{"vueCompilerOptions":{"target":"auto","checkUnknownProps":true}}');
  try {
    observations.before = capture();
    save();
    assert(
      observations.before.metadata.watchedFiles.includes(preferred),
      "the canonical resolver retains the nearer absent Vue candidate",
    );
    assert(observations.before.metadata.watchedFiles.includes(installed));
    assert.deepEqual(capture(), observations.before);
    fs.mkdirSync(path.dirname(preferred), { recursive: true });
    fs.writeFileSync(preferred, '{"version":"2.7.16"}');
    observations.created = capture();
    save();
    assert.notEqual(
      observations.created.metadata.configIdentity,
      observations.before.metadata.configIdentity,
    );
    assert.notEqual(observations.created.raw, observations.before.raw);
    assert(!observations.created.metadata.watchedFiles.includes(installed));
    fs.unlinkSync(preferred);
    observations.removed = capture();
    save();
    assert.deepEqual(
      observations.removed,
      observations.before,
      "removal restores the entire original identity, closure and projection",
    );
    fs.writeFileSync(preferred, '{"version":"unsupported"}');
    assert.throws(capture, /unsupported installed Vue version/);
    fs.writeFileSync(preferred, '{"version":"2.7.16"}');
    observations.repaired = capture();
    save();
    assert.deepEqual(observations.repaired, observations.created);
    for (const options of [{ target: 3 }, {}]) {
      fs.writeFileSync(config, JSON.stringify({ vueCompilerOptions: options }));
      const metadata = native.getVueLanguageProjectMetadata(config);
      assert.deepEqual(
        metadata.watchedFiles,
        [config],
        "non-auto targets perform no package lookup",
      );
    }
  } finally {
    save();
  }
}
