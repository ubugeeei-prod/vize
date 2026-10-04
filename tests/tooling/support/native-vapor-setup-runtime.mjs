// Complete native/stock inline defaults under separate genuine dev/prod runtimes.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { captureVaporProcess } from "./native-vapor-process-capture.mjs";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const runtimeVersion = fromVue("./package.json").version;
assert.equal(runtimeVersion, "3.6.0-rc.9");

function processFrame(label, packet, result, expectedFailure = false) {
  return {
    label,
    expectedFailure,
    input: JSON.stringify(packet),
    stdout: result.stdout ?? null,
    stderr: result.stderr ?? null,
    exit: result.status,
    signal: result.signal,
    error: result.error
      ? {
          name: result.error.name,
          message: result.error.message,
          stack: result.error.stack,
          code: result.error.code,
          errno: result.error.errno,
          syscall: result.error.syscall,
          spawnargs: result.error.spawnargs,
        }
      : null,
  };
}

async function configuredPacket(packet) {
  const { renderVaporSetupSfcReference, compilerVersion, pluginVersion } =
    await import("./native-vapor-setup-oracle.mjs");
  assert.ok(
    Array.isArray(packet.fixtures) && packet.fixtures.length > 0,
    "nonempty whole-module denominator",
  );
  assert.equal(new Set(packet.fixtures.map((fixture) => fixture.id)).size, packet.fixtures.length);
  const fixtures = [];
  for (const fixture of packet.fixtures) {
    assert.equal(typeof fixture.id, "string");
    assert.equal(typeof fixture.source, "string");
    assert.equal(typeof fixture.code, "string");
    assert.equal(fixture.map?.version, 3, "complete native map accompanies runtime input");
    assert.equal(fixture.reference.source, fixture.source, "original SFC primary custody");
    assert.equal(fixture.reference.id, fixture.id);
    assert.equal(fixture.reference.compilerVersion, compilerVersion);
    assert.equal(fixture.reference.pluginVersion, pluginVersion);
    assert.equal(fixture.reference.mode, "inline-production");
    for (const graph of [fixture.reference.client, fixture.reference.ssr]) {
      if (graph.map?.version !== 3) {
        assert.deepEqual(
          graph.map,
          { mappings: "" },
          "actual stock virtual-script main map remains exact",
        );
        assert.ok(graph.virtualModules.length > 0);
      }
      for (const virtual of graph.virtualModules) {
        assert.equal(virtual.loaded.map?.version, 3, "complete genuine virtual script map");
        assert.equal(virtual.transformed.map?.version, 3, "complete actual Vite transform map");
      }
    }
    const supplied = fixture.configurations ?? [
      { inheritAttrs: true, props: {} },
      ...(fixture.reference.rootElement
        ? [
            { inheritAttrs: true, props: { title: "inherited 雪🌸", "data-owner": "caller" } },
            { inheritAttrs: false, props: { title: "inherited 雪🌸", "data-owner": "caller" } },
          ]
        : []),
    ];
    assert.ok(supplied.length > 0, "nonempty caller configurations");
    const configurations = [];
    for (const value of supplied) {
      const configuration = {
        inheritAttrs: value.inheritAttrs ?? true,
        props: structuredClone(value.props ?? {}),
      };
      assert.equal(typeof configuration.inheritAttrs, "boolean");
      const server = await renderVaporSetupSfcReference(fixture.reference, configuration);
      if (value.serverHtml !== undefined)
        assert.equal(
          value.serverHtml,
          server.serverHtml,
          "caller HTML matches actual configured stock SSR",
        );
      configurations.push({ ...configuration, server });
    }
    fixtures.push({ ...fixture, configurations });
  }
  return { ...packet, version: runtimeVersion, pluginVersion, fixtures };
}

/** Capture raw children before any assertion, preserving real failures. */
export async function runVaporSetupSfcRuntime(packet) {
  const input = await configuredPacket(packet);
  const results = [];
  const frames = [];
  for (const flavor of ["development", "production"]) {
    const label = `native-vapor-setup-${flavor}`;
    const result = spawnSync(
      process.execPath,
      [new URL(import.meta.url).pathname, "--child", flavor],
      {
        input: JSON.stringify(input),
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
        timeout: 60_000,
      },
    );
    captureVaporProcess(label, input, result);
    frames.push(processFrame(label, input, result));
    assert.equal(result.error, undefined, `${label} process error`);
    assert.equal(result.signal, null, `${label} process signal`);
    assert.equal(result.status, 0, result.stderr || `${label} process exit`);
    const captured = JSON.parse(result.stdout);
    assert.equal(captured.runtime, flavor);
    assert.equal(captured.version, runtimeVersion);
    assert.equal(captured.fixtures.length, input.fixtures.length);
    results.push(captured);
    const absenceInput = { ...input, fixtures: [input.fixtures[0]] };
    for (const action of [
      "public-force-update",
      "public-primitive-setter",
      "private-instance-update",
    ]) {
      const absenceLabel = `${label}-missing-${action}`;
      const absence = spawnSync(
        process.execPath,
        [new URL(import.meta.url).pathname, "--child", flavor, action],
        {
          input: JSON.stringify(absenceInput),
          encoding: "utf8",
          maxBuffer: 16 * 1024 * 1024,
          timeout: 60_000,
        },
      );
      captureVaporProcess(absenceLabel, absenceInput, absence, true);
      frames.push(processFrame(absenceLabel, absenceInput, absence, true));
      assert.equal(absence.error, undefined);
      assert.equal(absence.signal, null);
      assert.equal(absence.status, 1, "actual missing native API fails in its original process");
      assert.match(absence.stderr, /TypeError:/, "actual native missing API TypeError retained");
      const authority = JSON.parse(absence.stdout);
      assert.equal(authority.action, action);
      assert.equal(authority.id, input.fixtures[0].id);
      assert.equal(authority.loadedSetupIdentity, true);
      assert.equal(authority.publicProxy, "undefined");
    }
  }
  assert.deepEqual(
    results[0].fixtures.map((fixture) => [fixture.id, fixture.traces]),
    results[1].fixtures.map((fixture) => [fixture.id, fixture.traces]),
    "actual primitive output/hydration/lifecycle matches both runtime builds",
  );
  return {
    version: runtimeVersion,
    pluginVersion: input.pluginVersion,
    runtimes: results,
    processFrames: frames,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const packet = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  const result =
    process.argv[2] === "--child"
      ? await (
          await import("./native-vapor-setup-runtime-child.mjs")
        ).runVaporSetupSfcRuntimeChild(packet, process.argv[3], process.argv[4] ?? null)
      : await runVaporSetupSfcRuntime(packet);
  process.stdout.write(JSON.stringify(result));
}
