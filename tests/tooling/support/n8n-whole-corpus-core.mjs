// Derived comparison views never replace the complete provider/process packets.
import assert from "node:assert/strict";
import path from "node:path";
import { projection, scopedConfig } from "./n8n-cli-config-inputs.mjs";

export function oracleRules(config) {
  const rules = {};
  for (const [rule, severity] of Object.entries(config.linter.rules)) {
    const name = projection.oracleRules[rule];
    assert.ok(name, `unmapped explicit identity: ${rule}`);
    const option = config.linter.ruleOptions[rule];
    if (rule === "vue/component-name-in-template-casing") {
      const { casing, ...scope } = option;
      rules[name] = [severity, casing, { registeredComponentsOnly: true, ...scope }];
    } else {
      rules[name] = option === undefined ? severity : [severity, option];
    }
  }
  assert.equal(Object.keys(rules).length, 51);
  return rules;
}

export function oracleLayers(packageRoot, mode) {
  assert.ok(["baseline", "scoped"].includes(mode));
  const config = scopedConfig(packageRoot, mode === "scoped");
  return {
    config,
    layers: [
      { files: ["**/*.vue"], rules: oracleRules(config) },
      ...(config.entries ?? []).map(({ files, linter }) => ({
        files,
        rules: Object.fromEntries(
          Object.entries(linter.rules).map(([rule, severity]) => [
            projection.oracleRules[rule],
            severity,
          ]),
        ),
      })),
    ],
  };
}

export function recordCorpusPackets(rawPackets, workspace) {
  // JSON is the provider's public result serialization. Keep that whole form
  // separately; only the named sidecar rewrites a known workspace filePath.
  const serializedRawPackets = JSON.parse(JSON.stringify(rawPackets));
  const packets = structuredClone(serializedRawPackets);
  for (const packet of packets) {
    const relative = path.relative(workspace, packet.filePath);
    if (relative && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative)) {
      packet.filePath = relative.replaceAll("\\", "/");
    }
  }
  return { serializedRawPackets, packets };
}

export function assertCompletePackets(packets, expectedFiles, provider) {
  const field = provider === "eslint" ? "filePath" : "file";
  assert.deepEqual(
    packets.map((packet) => packet[field]).sort(),
    [...expectedFiles].sort(),
    `${provider}: every original, clean and scriptless file exactly once`,
  );
  for (const packet of packets) {
    assert.ok(Array.isArray(packet.messages), `${provider}: messages array`);
    assert.equal(
      packet.errorCount,
      packet.messages.filter(({ severity }) => severity === 2).length,
      `${provider}: complete error count`,
    );
    assert.equal(
      packet.warningCount,
      packet.messages.filter(({ severity }) => severity === 1).length,
      `${provider}: complete warning count`,
    );
    if (provider === "eslint") {
      assert.ok(Array.isArray(packet.suppressedMessages), "complete suppressed messages");
      assert.equal(
        packet.fatalErrorCount,
        0,
        "provider parse/config failures are not findings parity",
      );
      assert.ok(packet.messages.every(({ ruleId }) => typeof ruleId === "string"));
    }
  }
}

export function physicalOffset(source, line, column, provider) {
  if (line === undefined || line === null || column === undefined || column === null) return null;
  assert.ok(Number.isInteger(line) && line > 0);
  assert.ok(Number.isInteger(column) && column > 0);
  // Native SourceLineIndex counts Unicode scalars; Vue ESLint counts UTF-16.
  // Both Vue providers use LF line starts (including CRLF), not lone CR/LS/PS.
  // These units are independently checked for template and script diagnostics.
  const starts = [0];
  for (const match of source.matchAll(/\n/gu)) starts.push(match.index + 1);
  assert.ok(line <= starts.length, "physical source line exists");
  const start = starts[line - 1];
  const segment = source.slice(start, starts[line] ?? source.length);
  const characters = provider === "native" ? [...segment] : segment.split("");
  assert.ok(column <= characters.length + 1, "physical source column exists");
  const prefix = characters.slice(0, column - 1).join("");
  // A provider coordinate may not split a supplementary character.
  assert.ok(!/[\uD800-\uDBFF]$/u.test(prefix), "coordinate is on a UTF-8 boundary");
  return Buffer.byteLength(source.slice(0, start) + prefix);
}

export function semanticFindings(packets, provider, packageRoot, sourceForFile) {
  const reverse = Object.fromEntries(
    Object.entries(projection.oracleRules).map(([native, oracle]) => [oracle, native]),
  );
  return packets.flatMap((packet) => {
    const file = provider === "eslint" ? packet.filePath : `${packageRoot}/${packet.file}`;
    const source = sourceForFile(file);
    return packet.messages.map((message) => {
      try {
        return {
          file,
          // Unknown/administrative identities remain visible rather than filtered.
          rule:
            provider === "eslint" ? (reverse[message.ruleId] ?? message.ruleId) : message.ruleId,
          severity: message.severity,
          startByte: physicalOffset(source, message.line, message.column, provider),
          endByte: physicalOffset(source, message.endLine, message.endColumn, provider),
        };
      } catch (error) {
        Object.assign(error, { provider, file, originalMessage: message });
        throw error;
      }
    });
  });
}

export function assertQualifiedSummary(summary) {
  assert.equal(summary.schema, "vize.n8n.whole-corpus-observer");
  assert.equal(summary.version, 1);
  assert.equal(summary.captureComplete, true, "complete two-provider observation is required");
  assert.deepEqual(summary.observedInventory, {
    originalFiles: 1369,
    scriptlessFiles: 19,
    packageRoots: 9,
  });
  assert.deepEqual(summary.observations, {
    native: 36,
    eslint: 36,
    nativeFilePackets: 5476,
    eslintFilePackets: 5476,
  });
  assert.deepEqual(summary.modes, ["baseline", "scoped"]);
  assert.equal(summary.repeats, 2);
  assert.deepEqual(summary.failures, []);
  assert.equal(summary.comparisons.length, 18);
  assert.deepEqual(
    summary.comparisons.map(({ packageRoot, mode }) => `${packageRoot}/${mode}`).sort(),
    Object.keys(projection.packageVueFileCounts)
      .flatMap((packageRoot) => ["baseline", "scoped"].map((mode) => `${packageRoot}/${mode}`))
      .sort(),
    "each original package/mode comparison exactly once",
  );
  assert.equal(
    summary.canonicalDifferenceCount,
    0,
    "whole-corpus findings differ; retain all research packets",
  );
  assert.ok(
    summary.comparisons.every(
      ({ nativeOnly, oracleOnly }) => nativeOnly.length === 0 && oracleOnly.length === 0,
    ),
  );
}

export function compareSemanticFindings(native, oracle) {
  // A multiset preserves duplicate findings. Neither paths nor ranges are
  // coarsened, and a missing end coordinate cannot match a present one.
  const pending = new Map();
  for (const finding of oracle) {
    const key = JSON.stringify(finding);
    pending.set(key, [...(pending.get(key) ?? []), finding]);
  }
  const nativeOnly = [];
  let matched = 0;
  for (const finding of native) {
    const bucket = pending.get(JSON.stringify(finding));
    if (bucket?.length) {
      bucket.pop();
      matched += 1;
    } else nativeOnly.push(finding);
  }
  return { matched, nativeOnly, oracleOnly: [...pending.values()].flat() };
}
