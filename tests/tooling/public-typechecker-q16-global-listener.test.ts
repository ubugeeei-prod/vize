import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import {
  afterProfile,
  requireProfile,
  sha256,
  type RegressionProfile,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  capture,
  inputImages,
  recorder,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import {
  readP73FullMetadata,
  selectP73FullMetadata,
  usesP73FullPackument,
  p73FullMetadataUrl,
  globalListenerArgs,
  globalListenerData,
  globalListenerFiles,
  globalListenerFix,
  globalListenerReferenceArgs,
  listenerNeedle,
  type ListenerState,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/global-listener-data.ts";
import {
  assessGlobalListener,
  globalListenerLimitations,
  writeGlobalListenerInputs,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/global-listener.ts";
import { requireOwnedDispatch } from "../../tools/benchmarks/scripts/public-typechecker-q16/run.ts";

const profile: RegressionProfile = {
  subset: "GlobalListener11373",
  fix: globalListenerFix,
  source: "1".repeat(40),
  candidate: "2".repeat(40),
  version: "0.653.0",
  run: "123",
  capsule: "3".repeat(40),
};
const data = globalListenerData();
function temporary(body: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "listener-pure-")));
  try {
    body(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
function output(status: number, stdout: string): ProcessOutput {
  return {
    status,
    stdout: Buffer.from(stdout),
    stderr: Buffer.alloc(0),
    signal: null,
    pid: 123,
    error: null,
    capacityLimited: false,
  };
}
function intended(state: ListenerState, receiver = "OpaqueA") {
  if (!state.diagnostics.length) return "";
  const d = state.diagnostics[0]!,
    needle = listenerNeedle(state, d);
  const header = `${d.file}(${needle.line},${needle.startColumn + 1}): error TS${d.code}: `;
  if (d.message !== null) return header + d.message + "\n";
  return (
    header +
    `Argument of type '${receiver}' is not assignable to parameter of type 'OpaqueB'.\n  Property 'to' is missing in type '${receiver}' but required in type 'OpaqueB'.\n`
  );
}
test("all twenty-two source images and fourteen independent contracts preserve the exact reported project", () => {
  assert.deepEqual(afterProfile, {
    subset: "GlobalListener11373",
    fix: "044768e5e35cedf030da532d78fe3c8f8a056310",
    version: "0.653.0",
    source: "1d355176f870d47ba75cbc1191b881556760f6fe",
    candidate: "bd211090b39049b5b2e3b71955cb8883c6a75c5b",
    run: "41535688394",
    capsule: "058fe417bffde1dae82fbba0300f650c9d83f0a7",
  });
  assert.equal(requireProfile(afterProfile), afterProfile);
  assert.equal(data.images.size, 22);
  assert.equal(data.states.length, 14);
  assert.deepEqual(
    data.states.map((s) => s.status),
    [0, 2, 2, 2, 0, 0, 2, 0, 2, 0, 0, 0, 2, 2],
  );
  assert.equal(
    data.states.reduce((n, s) => n + s.files.length, 0),
    89,
  );
  assert.deepEqual(data.states[0]!.files, data.states[10]!.files);
  assert.equal(
    data.states[9]!.files.find((f) => f.path === "src/GlobalComponentListener.vue")!.source,
    data.images.get("src/NoListener.vue.txt"),
  );
  const original = data.states[0]!,
    files = globalListenerFiles(original, profile);
  assert.deepEqual(files.slice(1), original.files.slice(1));
  assert.equal(
    files[0]!.source,
    original.files[0]!.source.replace('"vize": "0.640.0"', '"vize": "0.653.0"'),
  );
  assert.deepEqual(JSON.parse(files[0]!.source).devDependencies, {
    tsrs: "0.20.1-ts7.1.0-beta",
    typescript: "5.9.3",
    vize: "0.653.0",
    vue: "3.5.43",
    "vue-tsc": "3.2.6",
  });
  assert.equal(JSON.parse(files[1]!.source).compilerOptions.noUnusedLocals, true);
  assert.deepEqual(globalListenerArgs, [
    "--runExternalCode",
    "-p",
    "tsconfig.json",
    "--pretty",
    "false",
  ]);
  assert.deepEqual(globalListenerReferenceArgs, [
    "--noEmit",
    "-p",
    "tsconfig.json",
    "--pretty",
    "false",
  ]);
  assert.equal(globalListenerLimitations.sourceObservationsTransferred, false);
  assert.equal(requireProfile(profile), profile);
  assert.throws(() => requireProfile(null), assert.AssertionError);
  assert.throws(() => requireProfile({ ...profile, fix: "f".repeat(40) }), assert.AssertionError);
});
test("actual state writes preserve physical exact inputs including augmentation removal and exact restoration", () =>
  temporary((root) => {
    let files = globalListenerFiles(data.states[0]!, profile);
    for (const file of files) {
      const target = join(root, file.path);
      mkdirSync(dirname(target), { recursive: true });
      writeFileSync(target, file.source);
    }
    for (const state of data.states) {
      const next = globalListenerFiles(state, profile);
      writeGlobalListenerInputs(root, files, next, profile);
      files = next;
      assert.deepEqual(
        inputImages(root, files).map((f) => ({ path: f.path, source: f.source })),
        next,
      );
    }
    const canary = "P73_PRIVATE_INPUT_CANARY";
    const forged = files.map((f) =>
      f.path === "src/GlobalComponentListener.vue" ? { ...f, source: f.source + canary } : f,
    );
    let executed = 0,
      failure: Error | undefined;
    assert.throws(() => {
      try {
        writeGlobalListenerInputs(root, files, forged, profile);
        executed++;
      } catch (error) {
        failure = error as Error;
        throw error;
      }
    }, assert.AssertionError);
    assert.equal(executed, 0);
    assert(!String(failure?.message).includes(canary));
    assert.deepEqual(
      inputImages(root, files).map((f) => ({ path: f.path, source: f.source })),
      files,
    );
  }));
test("complete actor streams enforce seven clean and seven authored diagnostics without owning opaque receiver text", () =>
  temporary((root) => {
    const record = recorder(root);
    for (const [index, state] of data.states.entries()) {
      const stdout = intended(state),
        packet = capture(
          record,
          "state-" + index,
          ["inert"],
          root,
          () => {},
          () => output(state.status, stdout),
        );
      const result = assessGlobalListener(state, packet);
      assert.equal(result.status, state.status);
      assert(readFileSync(packet.stdoutImage.path).equals(Buffer.from(stdout)));
      assert.throws(
        () => assessGlobalListener(state, { ...packet, status: state.status === 0 ? 2 : 1 }),
        assert.AssertionError,
      );
      for (const change of [
        { stdout: stdout + "foreign\n" },
        { stderr: "foreign" },
        { wholeStreamsRetained: false },
        { signal: "SIGTERM" as const },
      ])
        assert.throws(
          () => assessGlobalListener(state, { ...packet, ...change }),
          assert.AssertionError,
        );
      if (state.diagnostics.length) {
        const d = state.diagnostics[0]!,
          needle = listenerNeedle(state, d);
        for (const bad of [
          stdout.replace(d.file, "src/Foreign.vue"),
          stdout.replace(`TS${d.code}`, "TS9999"),
          stdout.replace(
            `(${needle.line},${needle.startColumn + 1})`,
            `(${needle.line},${needle.endColumnExclusive + 2})`,
          ),
        ])
          assert.throws(
            () => assessGlobalListener(state, { ...packet, stdout: bad }),
            assert.AssertionError,
          );
        if (d.missingRequiredProperty !== null) {
          assert.equal(
            "wholePrimitiveMessageOwned" in result && result.wholePrimitiveMessageOwned,
            false,
          );
          const other = capture(
            record,
            "other-receiver",
            ["inert"],
            root,
            () => {},
            () => output(2, intended(state, "Different<{typed: number}>")),
          );
          assessGlobalListener(state, other);
          assert.throws(
            () =>
              assessGlobalListener(state, {
                ...packet,
                stdout: stdout.replace("Property 'to'", "Property 'href'"),
              }),
            assert.AssertionError,
          );
          assert.throws(
            () => assessGlobalListener(state, { ...packet, stdout: stdout.split("\n")[0]! + "\n" }),
            assert.AssertionError,
          );
          assert.throws(
            () => assessGlobalListener({ ...state, id: "foreign-case" }, packet),
            assert.AssertionError,
          );
        }
      }
    }
  }));
test("only the listener role changes declared ref runner and Node seams of the frozen controller workflow", () => {
  const workflow = readFileSync(
      new URL("../../.github/workflows/public-typechecker-consumer.yml", import.meta.url),
      "utf8",
    ),
    ref = "refs/heads/test/published-global-component-listener-11373-20261103",
    generic = "refs/heads/test/published-generic-arrow-tsx-20261103";
  const joint =
    "${{ github.event_name == 'workflow_dispatch' && inputs.consumer == 'q16-first-subset' && (github.ref == '" +
    generic +
    "' || github.ref == '" +
    ref +
    "') && 'ubuntu-24.04' || 'blacksmith-8vcpu-ubuntu-2404' }}";
  const prior =
    "${{ github.event_name == 'workflow_dispatch' && inputs.consumer == 'q16-first-subset' && github.ref == '" +
    generic +
    "' && 'ubuntu-24.04' || 'blacksmith-8vcpu-ubuntu-2404' }}";
  assert.equal(workflow.split(joint).length, 3);
  const node =
    "${{ (github.ref == 'refs/heads/main' || github.ref == '" +
    ref +
    "') && '26.12.1' || '26.13.0' }}";
  assert.equal(workflow.split(node).length, 2);
  const inverse = workflow
    .replaceAll(joint, prior)
    .replace(" || github.ref == '" + ref + "'", "")
    .replace(
      "        with:\n          node-version: ${{ (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/test/published-global-component-listener-11373-20261103') && '26.12.1' || '26.13.0' }}\n          package-manager-cache: false\n",
      "        with: { node-version: 26.13.0, package-manager-cache: false }\n",
    );
  assert.equal(sha256(inverse), "c4f57c0614c36966664251253fa89ac0b0b7bfc0e331b59b896479fc34da4ea0");
  requireOwnedDispatch({
    GITHUB_EVENT_NAME: "workflow_dispatch",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: ref,
  });
  assert.throws(
    () =>
      requireOwnedDispatch({
        GITHUB_EVENT_NAME: "workflow_dispatch",
        GITHUB_REPOSITORY: "ubugeeei-prod/vize",
        GITHUB_REF: "refs/heads/foreign",
      }),
    assert.AssertionError,
  );
});

// Whole genuine full-packument member, canonical JSON serialization; not an HTTP-body claim.
const actualFull653MemberBytes = Buffer.from(
  "eyJfaWQiOiJ2aXplQDAuNjUzLjAiLCJiaW4iOnsidnoiOiJiaW4vdml6ZSIsInZpemUiOiJiaW4vdml6ZSJ9LCJidWdzIjp7InVybCI6Imh0dHBzOi8vZ2l0aHViLmNvbS91YnVnZWVlaS1wcm9kL3ZpemUvaXNzdWVzIn0sImRpc3QiOnsic2hhc3VtIjoiZTc4ZjlmNjU0MzY4YTM4YzhlNmM2MmNmZDhkNWQ5OTA1MDNiNjlmYSIsInRhcmJhbGwiOiJodHRwczovL3JlZ2lzdHJ5Lm5wbWpzLm9yZy92aXplLy0vdml6ZS0wLjY1My4wLnRneiIsImZpbGVDb3VudCI6ODQsImludGVncml0eSI6InNoYTUxMi1QOVA1UFR6eGlzZzQ3WUsyT3Q1cFp0NmJjWVduaGxsWnM3RTNIbXVvUnpKUWM2Wis4U3p4cjN6RHZxZEFKaUZya1ZaQWFzSCtEMVR3WDhkOVhaTSs5Zz09Iiwic2lnbmF0dXJlcyI6W3sic2lnIjoiTUVRQ0lDS0VYSk11bGJ5YW82ZDdnNXlSYUZhVEN6WTkxcGt5TG5lNW1DQy9sMytmQWlCaWppY2JqMHlLOC9IZjNiUFAvVnhMajh1SEVZTXp5OVBVUkJaejAxVXJPQT09Iiwia2V5aWQiOiJTSEEyNTY6RGhROHdSNUFQQnZGSExGLytUYytBWXZQT2RUcGNJRHFPaHhzQkhSd0M3VSJ9LHsia2V5aWQiOiJTSEEyNTY6RGhROHdSNUFQQnZGSExGLytUYytBWXZQT2RUcGNJRHFPaHhzQkhSd0M3VSIsInNpZyI6Ik1FUUNJQkJybkZSSGNnMkhEbEtuVkJWT2FWWGNUL1kyUHFDVWdtWjkvZHFUY2ErMkFpQVVyUTFBQ0VBZDVWRnhxNTlkSUVPb3Q3ZVNxQURqUkdzY0dmWlhjckgxZFE9PSJ9XSwiYXR0ZXN0YXRpb25zIjp7InVybCI6Imh0dHBzOi8vcmVnaXN0cnkubnBtanMub3JnLy0vbnBtL3YxL2F0dGVzdGF0aW9ucy92aXplQDAuNjUzLjAiLCJwcm92ZW5hbmNlIjp7InByZWRpY2F0ZVR5cGUiOiJodHRwczovL3Nsc2EuZGV2L3Byb3ZlbmFuY2UvdjEifX0sInVucGFja2VkU2l6ZSI6ODI0MTc0fSwibWFpbiI6Ii4vZGlzdC9pbmRleC5tanMiLCJuYW1lIjoidml6ZSIsInR5cGUiOiJtb2R1bGUiLCJ0eXBlcyI6Ii4vZGlzdC9pbmRleC5kLm10cyIsImVuZ2luZXMiOnsibm9kZSI6Ij49MjIifSwiZXhwb3J0cyI6eyIuIjp7InR5cGVzIjoiLi9kaXN0L2luZGV4LmQubXRzIiwiaW1wb3J0IjoiLi9kaXN0L2luZGV4Lm1qcyIsImRlZmF1bHQiOiIuL2Rpc3QvaW5kZXgubWpzIn0sIi4vY29uZmlnIjp7InR5cGVzIjoiLi9kaXN0L2NvbmZpZy5kLm10cyIsImltcG9ydCI6Ii4vZGlzdC9jb25maWcubWpzIiwiZGVmYXVsdCI6Ii4vZGlzdC9jb25maWcubWpzIn0sIi4vdmlydHVhbC10cyI6eyJ0eXBlcyI6Ii4vdmlydHVhbC10cy5kLnRzIiwiaW1wb3J0Ijp7InR5cGVzIjoiLi92aXJ0dWFsLXRzLmQubXRzIn0sInJlcXVpcmUiOnsidHlwZXMiOiIuL3ZpcnR1YWwtdHMuZC5jdHMifX0sIi4vcGtsL3ZpemUucGtsIjoiLi9wa2wvdml6ZS5wa2wiLCIuL2NvbnRlbnQtbWFwcGVyLWhlbHBlcnMiOnsidHlwZXMiOiIuL2NvbnRlbnQtbWFwcGVyLWhlbHBlcnMuZC50cyIsImltcG9ydCI6eyJ0eXBlcyI6Ii4vY29udGVudC1tYXBwZXItaGVscGVycy5kLm10cyJ9LCJyZXF1aXJlIjp7InR5cGVzIjoiLi9jb250ZW50LW1hcHBlci1oZWxwZXJzLmQuY3RzIn19LCIuL3NjaGVtYXMvdml6ZS5jb25maWcuc2NoZW1hLmpzb24iOiIuL3NjaGVtYXMvdml6ZS5jb25maWcuc2NoZW1hLmpzb24ifSwibGljZW5zZSI6Ik1JVCIsInNjcmlwdHMiOnsiZm10IjoidnAgZm10IC0td3JpdGUgc3JjIHRlc3RzIHZpdGUuY29uZmlnLnRzIiwidGVzdCI6InZwIHBhY2sgJiYgdnAgdGVzdCBydW4gdGVzdHMvY29uZmlnLnRlc3QudHMgdGVzdHMvdml0ZS1wcm9qZWN0LWNvbmZpZy50ZXN0LnRzIHRlc3RzL3ZpdGUtbW9kdWxlLWNvbmZpZy50ZXN0LnRzIHRlc3RzL2RlZmluZS1jb25maWcudGVzdC50cyB0ZXN0cy9jb3JzYS1ydW50aW1lLnRlc3QudHMgdGVzdHMvc2V0dXAudGVzdC50cyB0ZXN0cy9zZXR1cC1veGxpbnQtY29uZmlnLnRlc3QudHMgdGVzdHMvaW5pdC50ZXN0LnRzIHRlc3RzL2luaXQtZnJhbWV3b3Jrcy50ZXN0LnRzIHRlc3RzL2luaXQtdHNjb25maWcudGVzdC50cyB0ZXN0cy9pbml0LXByb21wdC50ZXN0LnRzIHRlc3RzL2luaXQtY29uZmlnLWZyZWUudGVzdC50cyB0ZXN0cy9pbml0LXdvcmtzcGFjZS50ZXN0LnRzIiwiYnVpbGQiOiJ2cCBydW4gLS13b3Jrc3BhY2Utcm9vdCBnZW5lcmF0ZTpydWxlLXR5cGVzICYmIHZwIHBhY2siLCJjaGVjayI6InZwIGNoZWNrIHNyYyB0ZXN0cyB2aXRlLmNvbmZpZy50cyIsImNoZWNrOmZpeCI6InZwIGNoZWNrIC0tZml4IHNyYyB0ZXN0cyB2aXRlLmNvbmZpZy50cyIsImdlbmVyYXRlOnJ1bGUtdHlwZXMiOiJ2cCBydW4gLS13b3Jrc3BhY2Utcm9vdCBnZW5lcmF0ZTpydWxlLXR5cGVzIn0sInZlcnNpb24iOiIwLjY1My4wIiwiX25wbVVzZXIiOnsibmFtZSI6IkdpdEh1YiBBY3Rpb25zIiwiZW1haWwiOiJucG0tb2lkYy1uby1yZXBseUBnaXRodWIuY29tIiwidHJ1c3RlZFB1Ymxpc2hlciI6eyJpZCI6ImdpdGh1YiIsIm9pZGNDb25maWdJZCI6IjMzOGEyZDYwLTkyODUtNDRlOS1hZjM2LTM2ZGVkMDAzN2EzZiJ9fSwiaG9tZXBhZ2UiOiJodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplIiwia2V5d29yZHMiOlsiY2xpIiwiY29tcGlsZXIiLCJmb3JtYXR0ZXIiLCJsaW50ZXIiLCJydXN0Iiwidml6ZSIsInZ1ZSJdLCJtZXRhZGF0YSI6eyJ2aXplIjp7InN0YWJpbGl0eSI6ImFscGhhLXN1cHBvcnRlZCIsInN0YWJpbGl0eURvY3MiOiJodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplL2Jsb2IvbWFpbi9kb2NzL2NvbnRlbnQvc3RhYmlsaXR5Lm1kI3BhY2thZ2Utc3VwcG9ydC10aWVycyJ9fSwicmVwb3NpdG9yeSI6eyJ1cmwiOiJodHRwczovL2dpdGh1Yi5jb20vdWJ1Z2VlZWktcHJvZC92aXplIiwidHlwZSI6ImdpdCIsImRpcmVjdG9yeSI6Im5wbS9jbGkifSwidHlwZXNjcmlwdCI6eyJjb250ZW50TWFwcGVyIjp7ImV4ZWMiOlsibm9kZSIsIi4vYmluL3ZpemUiLCJjb250ZW50LW1hcHBlciJdLCJkeW5hbWljQ29uZmlnIjp0cnVlLCJjb21waWxlck9wdGlvbnMiOlsiY2hlY2tKcyIsIm5vVW51c2VkTG9jYWxzIiwicm9vdERpciIsIm1vZHVsZSIsIm1vZHVsZVJlc29sdXRpb24iLCJyZXNvbHZlUGFja2FnZUpzb25FeHBvcnRzIiwiY3VzdG9tQ29uZGl0aW9ucyIsImJhc2VVcmwiLCJwYXRocyIsIm1vZHVsZVN1ZmZpeGVzIiwibm9EdHNSZXNvbHV0aW9uIiwicHJlc2VydmVTeW1saW5rcyJdfX0sImRlc2NyaXB0aW9uIjoiVml6ZSAtIEhpZ2gtcGVyZm9ybWFuY2UgVnVlLmpzIHRvb2xjaGFpbiBpbiBSdXN0IiwiZGlyZWN0b3JpZXMiOnt9LCJtYWludGFpbmVycyI6W3sibmFtZSI6InVidWdlZWVpIiwiZW1haWwiOiJ1YnVnZTExMjJAZ21haWwuY29tIn1dLCJkZXBlbmRlbmNpZXMiOnsib3hjLXRyYW5zZm9ybSI6IjAuMTQ0LjAiLCJAdml6ZWpzL25hdGl2ZSI6IjAuNjUzLjAifSwicHVibGlzaENvbmZpZyI6eyJhY2Nlc3MiOiJwdWJsaWMifSwiX2hhc1Nocmlua3dyYXAiOmZhbHNlLCJkZXZEZXBlbmRlbmNpZXMiOnsidnVlIjoiMy41LjM1Iiwidml0ZSI6Im5wbTpAdm9pZHplcm8tZGV2L3ZpdGUtcGx1cy1jb3JlQDAuMS4yNCIsInRzZG93biI6IjAuMjIuMTQiLCJ2aXRlLXBsdXMiOiIwLjEuMjQiLCJ0eXBlc2NyaXB0IjoiNi4wLjMiLCJAdHNkb3duL2NzcyI6IjAuMjIuMTQiLCJAdHlwZXMvbm9kZSI6IjI1LjkuMiIsIkBwa2wtY29tbXVuaXR5L3BrbCI6IjAuMzAuMiJ9LCJwZWVyRGVwZW5kZW5jaWVzIjp7InZ1ZSI6Ij49Mi42LjAgfHwgPj0zLjYuMC0wIiwiQHBrbC1jb21tdW5pdHkvcGtsIjoiMC4zMC4yIn0sIm9wdGlvbmFsRGVwZW5kZW5jaWVzIjp7IkB0eXBlc2NyaXB0L3R5cGVzY3JpcHQtbGludXgteDY0IjoiNy4wLjIiLCJAdHlwZXNjcmlwdC90eXBlc2NyaXB0LXdpbjMyLXg2NCI6IjcuMC4yIiwiQHR5cGVzY3JpcHQvdHlwZXNjcmlwdC1kYXJ3aW4teDY0IjoiNy4wLjIiLCJAdHlwZXNjcmlwdC90eXBlc2NyaXB0LWxpbnV4LWFybTY0IjoiNy4wLjIiLCJAdHlwZXNjcmlwdC90eXBlc2NyaXB0LXdpbjMyLWFybTY0IjoiNy4wLjIiLCJAdHlwZXNjcmlwdC90eXBlc2NyaXB0LWRhcndpbi1hcm02NCI6IjcuMC4yIn0sInBlZXJEZXBlbmRlbmNpZXNNZXRhIjp7InZ1ZSI6eyJvcHRpb25hbCI6dHJ1ZX0sIkBwa2wtY29tbXVuaXR5L3BrbCI6eyJvcHRpb25hbCI6dHJ1ZX19LCJfbnBtT3BlcmF0aW9uYWxJbnRlcm5hbCI6eyJob3N0IjoiczM6Ly9ucG0tcmVnaXN0cnktcGFja2FnZXMtbnBtLXByb2R1Y3Rpb24iLCJ0bXAiOiJ0bXAvdml6ZV8wLjY1My4wXzE3OTM2NTk5MTEzNTNfMC4xNjE5NTM5NDg4MzE2NDQ1In19",
  "base64",
);
const actualFull653Member = JSON.parse(actualFull653MemberBytes.toString("utf8"));
const full653Fixture = { name: "vize", versions: { "0.653.0": actualFull653Member } };
const full653Lock = {
  version: "0.653.0",
  integrity: actualFull653Member.dist.integrity,
  resolved: actualFull653Member.dist.tarball,
};
const full653CapsuleEntry = {
  integrity: actualFull653Member.dist.integrity,
  tarball: actualFull653Member.dist.tarball,
  metadataOrigin: p73FullMetadataUrl,
};
test("only the genuine653 full member and exact capsule distribution bindings are admitted", () => {
  assert.equal(
    sha256(actualFull653MemberBytes),
    "31f00b3e5f5a3605b3098481f6ec07538b1aceb8e13a5752d4baaec6226196eb",
  );
  assert.deepEqual(
    selectP73FullMetadata(full653Fixture, full653Lock, full653CapsuleEntry),
    actualFull653Member,
  );
  assert(usesP73FullPackument(profile, "vize", "0.653.0", full653CapsuleEntry));
  assert(!usesP73FullPackument(profile, "foreign", "0.653.0", full653CapsuleEntry));
  assert(!usesP73FullPackument(profile, "vize", "0.652.0", full653CapsuleEntry));
  const mutants = [
    { ...full653Fixture, name: "foreign" },
    { ...full653Fixture, versions: {} },
    ...[
      { name: "foreign" },
      { version: "0.652.0" },
      {
        dist: {
          ...actualFull653Member.dist,
          integrity: "sha512-" + Buffer.alloc(64, 1).toString("base64"),
        },
      },
      {
        dist: {
          ...actualFull653Member.dist,
          tarball: "https://registry.npmjs.org/vize/-/vize-0.652.0.tgz",
        },
      },
      {
        dist: {
          ...actualFull653Member.dist,
          attestations: { url: "https://registry.npmjs.org/-/npm/v1/attestations/vize@0.652.0" },
        },
      },
    ].map((change) => ({
      name: "vize",
      versions: { "0.653.0": { ...actualFull653Member, ...change } },
    })),
  ];
  for (const mutant of mutants)
    assert.throws(
      () => selectP73FullMetadata(mutant, full653Lock, full653CapsuleEntry),
      assert.AssertionError,
    );
  assert.throws(
    () =>
      selectP73FullMetadata(
        full653Fixture,
        { ...full653Lock, version: "0.652.0" },
        full653CapsuleEntry,
      ),
    assert.AssertionError,
  );
  assert.throws(
    () =>
      selectP73FullMetadata(full653Fixture, full653Lock, {
        ...full653CapsuleEntry,
        metadataOrigin: "https://registry.npmjs.org/vize/0.653.0",
      }),
    assert.AssertionError,
  );
});
test("full member reader retains actual response-shaped bytes and caches without canonical fallback", async () => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "listener-full-meta-")));
  try {
    const first = join(root, "first"),
      second = join(root, "second");
    mkdirSync(first);
    mkdirSync(second);
    const body = Buffer.from(JSON.stringify(full653Fixture)),
      cache = new Map<string, Buffer>();
    let requests = 0;
    const fetcher: typeof fetch = async (url, options) => {
      requests++;
      assert.equal(url, p73FullMetadataUrl);
      assert.equal(new Headers(options?.headers).get("Accept"), "application/json");
      assert.equal(options?.redirect, "error");
      const response = new Response(body, {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
      Object.defineProperty(response, "url", { value: p73FullMetadataUrl });
      return response;
    };
    assert.deepEqual(
      await readP73FullMetadata(
        profile,
        full653Lock,
        full653CapsuleEntry,
        recorder(first),
        cache,
        fetcher,
      ),
      actualFull653Member,
    );
    assert.deepEqual(
      await readP73FullMetadata(
        profile,
        full653Lock,
        full653CapsuleEntry,
        recorder(second),
        cache,
        fetcher,
      ),
      actualFull653Member,
    );
    assert.equal(requests, 1);
    for (const out of [first, second]) {
      assert(readFileSync(join(out, "vize-full-packument.raw.json")).equals(body));
      const selection = JSON.parse(
        readFileSync(join(out, "vize-full-packument-selection.json"), "utf8"),
      );
      assert.equal(selection.canonicalVersionHttpStatusClaimed, false);
      assert.equal(selection.representation, "EXACT_FULL_PACKUMENT_MEMBER");
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
