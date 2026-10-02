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
];
export const hostModule = "crates/vize_carton/src/profile_allocator.rs";
export const hostModuleText =
  "//! Host allocator selection for allocation profiling.\n\nuse std::alloc::System;\n\nuse vize_l0::profiler::ProfilingAllocator;\n\n/// Select the host system allocator without another allocator wrapper.\n///\n/// The returned value is the exact L0 accounting wrapper around [`System`].\n/// Native callers selecting another allocator continue to use\n/// [`ProfilingAllocator::from_allocator`] directly.\n///\n/// ```\n/// const ALLOCATOR: vize_l0::profiler::ProfilingAllocator<std::alloc::System> =\n///     vize_carton::profile_allocator::system_allocator();\n/// let _ = ALLOCATOR;\n/// ```\npub const fn system_allocator() -> ProfilingAllocator<System> {\n    ProfilingAllocator::from_allocator(System)\n}\n";
const removedSystem =
  "impl ProfilingAllocator<System> {\n    /// Create a profiling allocator backed by [`System`].\n    pub const fn new() -> Self {\n        Self { inner: System }\n    }\n}\n\nimpl Default for ProfilingAllocator<System> {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\n";
const digest = (text: string) => createHash("sha256").update(text).digest("hex");
type Reader = (file: string) => string | undefined;

/** Return a complete validated plan; reject partial or changed input first. */
export function prepareAllocatorSelection(read: Reader): Map<string, string> {
  const found = contracts.map((contract) => ({ ...contract, text: read(contract.file) }));
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
    let text = read(contract.file)!;
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
