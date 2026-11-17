/**
 * The one glibc floor every shipped Linux GNU artifact (the `@vizejs/native`
 * and `@vizejs/fresco-native` bindings and the `vize` CLI archives, x64 and
 * arm64) is linked against and gated on.
 *
 * glibc 2.17 is the manylinux2014 baseline and the floor of Rust's own
 * `*-unknown-linux-gnu` standard library, so no supported dependency needs
 * more. Official Node.js 22+ builds already need glibc 2.28, so the binding
 * never narrows where Node itself runs: Amazon Linux 2023 (Vercel builds and
 * functions, AWS Lambda), Debian 10+, Ubuntu 20.04+, and RHEL/Alma 8+ load it.
 *
 * Release builds link through `cargo zigbuild --target <triple>.2.17`, whose
 * glibc stubs only expose symbol versions up to the floor, and
 * `verify-glibc-symbols.ts` reads every built ELF image and fails the build
 * when any required symbol version is newer. `npm/native/native-binding.js`
 * repeats the value in its load-failure diagnostics.
 */
import { isEntrypoint, runEntrypoint } from "../command-runtime.ts";

export const linuxGnuGlibcFloor = "2.17";

export function isLinuxGnuTarget(target: string): boolean {
  return /^(?:x86_64|aarch64)-unknown-linux-gnu$/u.test(target);
}

/**
 * The cargo-zigbuild spelling that links a Linux GNU target against the floor;
 * every other target is returned unchanged.
 */
export function glibcFloorTarget(target: string): string {
  return isLinuxGnuTarget(target) ? `${target}.${linuxGnuGlibcFloor}` : target;
}

export function main(args: readonly string[]): number {
  const [command, target] = args;
  if (command === "floor" && args.length === 1) {
    console.log(linuxGnuGlibcFloor);
    return 0;
  }
  if (command === "target" && target !== undefined && args.length === 2) {
    console.log(glibcFloorTarget(target));
    return 0;
  }
  throw new Error("Usage: node tools/support/github/glibc-floor.ts <floor | target <rust-target>>");
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(process.argv.slice(2)));
