#!/usr/bin/env -S vp node
/** Keep concrete host allocator selection outside the L0 wrapper. */
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const contracts = [
  {
    file: "davinci/vize_l0/src/profiler/allocation.rs",
    old: "d3e899fad86456821435fabbc70da210c55e7c44b9c5c8c06218acc221781517",
    next: "d97bd9786d45e715e74b84f46663c24b61639fc63772a6df33ac127da6d6339e",
  },
  {
    file: "crates/vize_carton/src/lib.rs",
    old: "1cf9c91e8cf51fe149299d26d22c5ebbc388677cf02c2bae6da580d29da1fb2e",
    next: "fc19310ca91b9adcdc738010c442353f68172e58cb560d6fc042250f57d8b32f",
  },
  {
    file: "crates/vize_atelier_sfc/tests/allocation_budget.rs",
    old: "541d84f2eb1304aef56e76b01354137f36387c2b6b6d05d2ef92d307d10b43c0",
    next: "958e159032c13e19222b2e54557d89d5e6927abf7ddd38d2a465ffe39269bb2c",
  },
  {
    file: "crates/vize_carton/tests/davinci_profile_export.rs",
    old: "9f18a586da1dee0097ad38c042caea7e52e31508b0747c42e1d6565cd12f7aae",
    next: "e35f20d09549d359661f8b054dcc6e5533f31adc07d5d45ffcd76289ccde9e8b",
  },
  {
    file: "tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs",
    old: "0039b68bc23fa3077a61407c0caf903a978e81027377c99563142120008829f1",
    next: "2d743d27f4328b01a811d7ba0f962fcd4541e528f09e8193d346d23ead9d8921",
  },
  {
    file: "tools/support/levels/profile-export-host-contract.ts",
    old: "9050edf0a4777c61029c1fdbbcb9d346470403659bac95754c8c6329c80f7c1a",
    next: "5b2c4fad9c9d3397bb47c023abe3ea9d3e26157d0fdb5b34ed9dbc105ff8748c",
  },
];
export const hostModule = "crates/vize_carton/src/profile_allocator.rs";
export const hostModuleText =
  "//! Host allocator selection for allocation profiling.\n\nuse std::alloc::System;\n\nuse vize_l0::profiler::ProfilingAllocator;\n\n/// Select the host system allocator without another allocator wrapper.\n///\n/// The returned value is the exact L0 accounting wrapper around [`System`].\n/// Native callers selecting another allocator continue to use\n/// [`ProfilingAllocator::from_allocator`] directly.\n///\n/// ```\n/// const ALLOCATOR: vize_l0::profiler::ProfilingAllocator<std::alloc::System> =\n///     vize_carton::profile_allocator::system_allocator();\n/// let _ = ALLOCATOR;\n/// ```\npub const fn system_allocator() -> ProfilingAllocator<System> {\n    ProfilingAllocator::from_allocator(System)\n}\n";
export const sfcImportGate = "tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs";
const sfcGateAnchor = '\nvoid test("Atelier SFC declares';
const sfcGateHelper =
  '\nfunction assertPreferredSource(source, relPath) {\n  const selected =\n    relPath === "crates/vize_atelier_sfc/tests/allocation_budget.rs"\n      ? source.replace(\n          /^#\\[global_allocator\\]\\nstatic GLOBAL: ProfilingAllocator<System> = vize_carton::profile_allocator::system_allocator\\(\\);$/mu,\n          "",\n        )\n      : source;\n  assert.doesNotMatch(selected, /\\bvize_carton\\b/u, relPath);\n}\n';
const sfcGateLaws =
  '\nvoid test("Atelier SFC admits only its exact host allocator selection and rejects Carton storage", () => {\n  const relPath = "crates/vize_atelier_sfc/tests/allocation_budget.rs";\n  const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");\n  assertPreferredSource(source, relPath);\n  assert.throws(() =>\n    assertPreferredSource(source, "crates/vize_atelier_sfc/src/compile_template.rs"),\n  );\n  for (const changed of [\n    source.replace("#[global_allocator]\\n", ""),\n    source.replace("ProfilingAllocator<System>", "ProfilingAllocator<WrongAllocator>"),\n    source.replace("system_allocator()", "system_allocator(WrongAllocator)"),\n    source.replace("profile_allocator::system_allocator", "profile_export::system_allocator"),\n    source + "\\nuse vize_carton::String;\\n",\n    source + "\\nuse vize_carton::profiler::ProfilingAllocator;\\n",\n    source +\n      "\\nstatic OTHER: ProfilingAllocator<System> = vize_carton::profile_allocator::system_allocator();\\n",\n    source +\n      "\\nstatic GLOBAL: ProfilingAllocator<System> = vize_carton::profile_allocator::system_allocator();\\n",\n  ]) {\n    assert.throws(() => assertPreferredSource(changed, relPath), { code: "ERR_ASSERTION" });\n  }\n});\n';
const sfcGateOldCall = "assert.doesNotMatch(source, /\\bvize_carton\\b/u, relPath);";
const sfcGateNewCall = "assertPreferredSource(source, relPath);";
export const profileExportContract = "tools/support/levels/profile-export-host-contract.ts";
const sfcOldGateDigestDeclaration =
  '      "0039b68bc23fa3077a61407c0caf903a978e81027377c99563142120008829f1",\n';
const sfcHostGateDigestDeclaration =
  '      "2d743d27f4328b01a811d7ba0f962fcd4541e528f09e8193d346d23ead9d8921",\n';
const removedSystem =
  "impl ProfilingAllocator<System> {\n    /// Create a profiling allocator backed by [`System`].\n    pub const fn new() -> Self {\n        Self { inner: System }\n    }\n}\n\nimpl Default for ProfilingAllocator<System> {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\n";
const digest = (text: string) => createHash("sha256").update(text).digest("hex");
type Reader = (file: string) => string | undefined;

/** Retain original replay through exact later exporter and IO/path host states.
 * Current continuations are checks only; replay never overwrites those sources.
 */
export function allocatorReplayText(file: string, text: string | undefined): string | undefined {
  if (
    file === "crates/vize_carton/src/lib.rs" &&
    text !== undefined &&
    [
      "ca130e97a8c3e035dfb68202184392532bf119012a3fffc5d58d44caeaa02f94",
      "9c135560ad6debbba6078194084a36e6dc086bb9129c48d8d61d04ba12d14ae2",
      "fc77ccf3dc066dedce6fb03b83b02d503fe3414328139a0ae001edb393b6f49e",
    ].includes(digest(text))
  ) {
    const historical = text
      .replace("\npub mod timing_observer;\n", "")
      .replace("\npub mod path;\n", "")
      .replace("\npub mod source_io;\n", "");
    if (digest(historical) !== contracts.find((contract) => contract.file === file)!.next)
      throw new Error("changed, missing, colliding or partial allocator selection");
    return historical;
  }
  if (
    file !== profileExportContract ||
    text === undefined ||
    ![
      "18a7b520cdb57b36e6c29c609be9d23632ae4875830d2d4e86d6528baeb5b82e",
      "4e1dcd7780d3d7d80041143e806c61ee731a90ef952bce9e8fbbf76949a5b347",
    ].includes(digest(text))
  )
    return text;
  let historical = text;
  for (const addition of [
    '      "3d1747c3dcfcf4ec8bc80ec722a6bd58d488cfdef2c7de84374a526417ebc9f9",\n',
    '  assembly: [\n    "crates/vize_carton/src/profile_export/assemble.rs",\n    "9f9c430af4303fe7fc5dabf12d1a2f7ec9c2794bab3541e64b3d5fe7d8c780ab",\n  ],\n',
    '  snapshotCompanion: "109d5bce0860c6d2007e00593d8eba756ee5d1b4bd7cb4e251cf89a4635d425d",\n  snapshotCaller: [\n    "crates/vize_curator/src/inspector/stages/profile.rs",\n    "7e1ad6a552d23ff11d409b32c135047857bbff88b35bfe058345af34aa5cd533",\n  ],\n',
  ])
    historical = historical.replace(addition, "");
  if (digest(historical) !== contracts.find((contract) => contract.file === file)!.next)
    throw new Error("changed, missing, colliding or partial allocator selection");
  return historical;
}

/** Return a complete validated plan; reject partial or changed input first. */
export function prepareAllocatorSelection(read: Reader): Map<string, string> {
  const found = contracts.map((contract) => ({
    ...contract,
    text: allocatorReplayText(contract.file, read(contract.file)),
  }));
  const module = read(hostModule);
  const old =
    module === undefined &&
    found.every(
      (contract) => contract.text !== undefined && digest(contract.text) === contract.old,
    );
  const next =
    module === hostModuleText &&
    found.every(
      (contract) => contract.text !== undefined && digest(contract.text) === contract.next,
    );
  if (!old && !next) throw new Error("changed, missing, colliding or partial allocator selection");
  if (next) return new Map();
  const planned = new Map<string, string>();
  for (const contract of found) {
    let text = contract.text!;
    if (contract.file === "davinci/vize_l0/src/profiler/allocation.rs")
      text = text
        .replace(
          "use std::alloc::{GlobalAlloc, Layout, System};",
          "use std::alloc::{GlobalAlloc, Layout};",
        )
        .replace("pub struct ProfilingAllocator<A = System>", "pub struct ProfilingAllocator<A>")
        .replace(removedSystem, "");
    else if (contract.file === "crates/vize_carton/src/lib.rs")
      text = text.replace(
        "pub mod profile_export;",
        "pub mod profile_allocator;\npub mod profile_export;",
      );
    else if (contract.file === sfcImportGate)
      text =
        text
          .replace(sfcGateAnchor, sfcGateHelper + sfcGateAnchor)
          .replace(sfcGateOldCall, sfcGateNewCall) + sfcGateLaws;
    else if (contract.file === profileExportContract)
      text = text.replace(
        sfcOldGateDigestDeclaration,
        sfcOldGateDigestDeclaration + sfcHostGateDigestDeclaration,
      );
    else
      text = text.replace(
        "ProfilingAllocator::new()",
        "vize_carton::profile_allocator::system_allocator()",
      );
    if (digest(text) !== contract.next)
      throw new Error("allocator replay does not match reviewed bytes");
    planned.set(contract.file, text);
  }
  planned.set(hostModule, hostModuleText);
  return planned;
}

/** Recover exact old inputs for replay laws without another source fixture. */
export function originalAllocatorSelection(read: Reader): Map<string, string> {
  if (prepareAllocatorSelection(read).size)
    throw new Error("original recovery requires final state");
  const original = new Map<string, string>();
  for (const contract of contracts) {
    let text = allocatorReplayText(contract.file, read(contract.file))!;
    if (contract.file === "davinci/vize_l0/src/profiler/allocation.rs")
      text = text
        .replace(
          "use std::alloc::{GlobalAlloc, Layout};",
          "use std::alloc::{GlobalAlloc, Layout, System};",
        )
        .replace("pub struct ProfilingAllocator<A>", "pub struct ProfilingAllocator<A = System>")
        .replace(
          "impl<A> ProfilingAllocator<A> {",
          removedSystem + "impl<A> ProfilingAllocator<A> {",
        );
    else if (contract.file === "crates/vize_carton/src/lib.rs")
      text = text.replace("pub mod profile_allocator;\n", "");
    else if (contract.file === sfcImportGate)
      text = text
        .replace(sfcGateHelper, "")
        .replace(sfcGateNewCall, sfcGateOldCall)
        .replace(sfcGateLaws, "");
    else if (contract.file === profileExportContract)
      text = text.replace(sfcHostGateDigestDeclaration, "");
    else
      text = text.replace(
        "vize_carton::profile_allocator::system_allocator()",
        "ProfilingAllocator::new()",
      );
    if (digest(text) !== contract.old) throw new Error("original allocator bytes changed");
    original.set(contract.file, text);
  }
  return original;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
  const mode = process.argv[2];
  if (mode !== "integrate" && mode !== "check")
    throw new Error("usage: select-profile-allocator-host.ts integrate|check");
  const planned = prepareAllocatorSelection((file) => {
    const full = path.join(root, file);
    return fs.existsSync(full) ? fs.readFileSync(full, "utf8") : undefined;
  });
  if (mode === "check" && planned.size)
    throw new Error("host allocator selection remains incomplete");
  if (mode === "integrate")
    for (const [file, text] of planned) {
      fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
      fs.writeFileSync(path.join(root, file), text);
    }
}
