// A CommonJS-only ambient declaration. `export =` is rejected outright in an ES
// module (`TS1203`), so this file is the one that still has to be mirrored with
// CommonJS spelling (#2679). Neither tool may report anything for it.
declare module "legacy-shout" {
  function legacyShout(label: string): string;
  export = legacyShout;
}
