{ inputs, ... }:
let
  root = ../..;
in
{
  perSystem =
    {
      lib,
      pkgs,
      rustToolchain,
      ...
    }:
    let
      # crane builds with the toolchain `tools/nix/pkgs.nix` pins, not the one
      # nixpkgs happens to carry, so the package and the dev shell compile with
      # the same rustc.
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

      vize = import (root + /default.nix) { inherit craneLib pkgs root; };
    in
    {
      packages = {
        inherit vize;
        default = vize;
      };

      apps =
        lib.genAttrs (lib.attrNames vize.passthru.binaries) (name: {
          type = "app";
          program = lib.getExe' vize name;
        })
        // {
          default = {
            type = "app";
            program = lib.getExe vize;
          };
        };

      checks.package = vize;
      checks.embedded-source-assets = pkgs.runCommand "vize-embedded-source-assets" { } ''
        cmp \
          ${vize.src}/crates/vize_canon/src/virtual_ts/helpers/pattern_matching.d.ts \
          ${root + /crates/vize_canon/src/virtual_ts/helpers/pattern_matching.d.ts}
        cmp \
          ${vize.src}/crates/vize_patina/src/html_content_model/whatwg.tsv \
          ${root + /crates/vize_patina/src/html_content_model/whatwg.tsv}
        test ! -e ${vize.src}/playground/package.json
        test ! -e ${vize.src}/crates/vize_canon/tests/snapshots/patterned_typechecking__patterned_root_virtual_ts.snap
        touch "$out"
      '';
    };

  # Downstream flakes take the CLI through the overlay rather than reaching
  # into `packages` for a system they have to name themselves.
  flake.overlays.default = final: prev: {
    vize = inputs.self.packages.${prev.system}.default;
  };
}
