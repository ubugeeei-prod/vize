{
  craneLib,
  lib,
  libiconv,
  pkg-config,
  root ? ./.,
  stdenv,
}:
let
  inherit (lib.importTOML (root + /Cargo.toml)) workspace;
  inherit (workspace.package) version license;

  # Installed binary -> the crate that produces it. The napi and wasm crates
  # (`vize_fresco`, `vize_vitrine`) are deliberately absent: one links against
  # the Node headers napi-build expects and the other only makes sense for the
  # wasm32 target, so neither builds as a host binary and neither is what
  # `nix run` should hand you.
  #
  # One mapping rather than two parallel lists, because the build selects
  # crates and `meta` describes binaries, and the pairing between them is what
  # would rot if they were kept apart.
  binaries = {
    "vize" = "vize";
  };

  mainProgram = "vize";

  # The binary crate states its own description; it is not restated here.
  describe = crate: (lib.importTOML (root + "/crates/${crate}/Cargo.toml")).package.description;

  # `filterCargoSources` keeps only what Cargo itself reads, but parts of this
  # workspace are embedded at compile time with `include_str!`: the carton
  # message catalogues, the canon TypeScript preamble, the marquette JSON
  # schemas. The CLI's config schema lives outside `crates/` entirely, beside
  # the npm package that also ships it. Filtered out, they fail the build
  # during macro expansion rather than at link time, which is why the error
  # names a file and not a symbol.
  #
  # `.snap` is deliberately not in the list. Insta snapshots are read only by
  # tests, which `doCheck = false` skips, and there are several thousand.
  crateAssetSuffixes = [
    ".json"
    ".txt"
  ];

  externalAssets = [
    "/npm/cli/schemas/vize.config.schema.json"
  ];

  src = lib.cleanSourceWith {
    name = "vize-source";
    src = lib.cleanSource root;
    filter =
      path: type:
      craneLib.filterCargoSources path type
      || (lib.hasInfix "/crates/" path && lib.any (suffix: lib.hasSuffix suffix path) crateAssetSuffixes)
      || lib.any (asset: lib.hasSuffix asset path) externalAssets;
  };

  commonArgs = {
    pname = "vize";
    inherit version src;
    strictDeps = true;

    # The workspace's tests are driven by the Vite+ task graph (`vp run test`),
    # which knows about the fixtures and the corpora this derivation does not
    # carry.
    doCheck = false;

    cargoExtraArgs = lib.concatMapStringsSep " " (crate: "-p ${crate}") (lib.attrValues binaries);

    nativeBuildInputs = [ pkg-config ];
    buildInputs = lib.optionals stdenv.hostPlatform.isDarwin [ libiconv ];
  };

  # Built once from a manifest-only source tree and reused by every later
  # build, so editing a crate does not rebuild oxc and its dependents.
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    passthru = {
      inherit binaries cargoArtifacts commonArgs;
    };

    meta = {
      description = describe binaries.${mainProgram};
      homepage = "https://vizejs.dev";
      inherit mainProgram;
      license = lib.getLicenseFromSpdxId license;
      platforms = import (root + /tools/nix/systems.nix);
    };
  }
)
