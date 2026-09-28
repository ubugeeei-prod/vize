# Rust guest SDK crate axis (2026-09-28)

Tracked in [#6826](https://github.com/ubugeeei-prod/vize/issues/6826) and
[#6841](https://github.com/ubugeeei-prod/vize/issues/6841). The maintainer
[approved option A](https://github.com/ubugeeei-prod/vize/issues/6826#issuecomment-5872285480):
keep a standalone Rust guest SDK as an external product, named
`vize_guest`. The internal level-only naming rule remains in force.

- `vize_guest` owns the versioned `vize:contracts` WIT, Rust guest bindings,
  handshake constants, page writers, and `no_std` guest runtime. Its shipped
  dependency graph contains no Vize implementation crate. P6-2 still packs the
  crate and builds an external hello-world component from the tarball.
- `vize_extension_contract` is internal implementation, so its transport-neutral
  records belong in L0, input surface in L1, accepted input page in L1→L2,
  expression facts in L2, and output projection in L4. Move these in small
  move-only slices as their destination APIs become available.
- `vize_extension_host` is internal host implementation. Its WIT and process
  transports belong behind the existing `vize` product shell. The host must
  continue to validate the same versioned WIT and run the packed guest test.
- `vize_dialect_moonbit` is internal language implementation: retain MoonBit
  modules at the appropriate levels, with L2 owning expression semantics and
  native L1 input decoding replacing any legacy Croquis edge.

The first change moves the guest SDK directory without editing files, followed
by a scripted package-reference adaptation. The archived 0.1.2 source and its
recorded hashes stay byte-for-byte fixed, and the `vize:contracts` package,
worlds, versions, fields, and handshake remain the public ABI. The JS/TS
`@vizejs/extension-sdk` package is a separate published product and is outside
this Rust crate rename.

The old internal extension and dialect packages are not accepted long-term
crate boundaries. Their retirement remains open under #6841 and #6833; the SDK
rename alone does not close those issues or mark Davinci complete.
