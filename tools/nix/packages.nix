{ inputs, ... }:
let
  renderScript = import ./render-script.nix;
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
      # crane builds with the MSRV `tools/nix/pkgs.nix` pins, not the compiler
      # nixpkgs happens to carry or the newer contributor dev shell toolchain.
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
      checks.embedded-source-assets = pkgs.runCommand "vize-embedded-source-assets" { } (
        renderScript ./scripts/check-embedded-assets.sh {
          SOURCE = "${vize.src}";
          ORIGINAL_OFFICIAL_COMPILER = "${root + /crates/vize/src/commands/inspector/official_compiler.ts}";
          ORIGINAL_PATTERN_MATCHING_D = "${
            root + /crates/vize_canon/src/virtual_ts/helpers/pattern_matching.d.ts
          }";
          ORIGINAL_WHATWG = "${root + /crates/vize_patina/src/html_content_model/whatwg.tsv}";
          ORIGINAL_MODULE_RUNTIME = "${root + /crates/vize_carton/src/config/loader/module-runtime.mjs}";
          ORIGINAL_VITE_RUNTIME_VALIDATION = "${
            root + /crates/vize_carton/src/config/loader/vite-runtime-validation.js
          }";
        }
      );
    };

  # Downstream flakes take the CLI through the overlay rather than reaching
  # into `packages` for a system they have to name themselves.
  flake.overlays.default = final: prev: {
    vize = inputs.self.packages.${prev.system}.default;
  };
}
