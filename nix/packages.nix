{ inputs, ... }:
let
  root = ./..;
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
      # crane builds with the toolchain `nix/pkgs.nix` pins, not the one
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
    };

  # Downstream flakes take the CLI through the overlay rather than reaching
  # into `packages` for a system they have to name themselves.
  flake.overlays.default = final: prev: {
    vize = inputs.self.packages.${prev.system}.default;
  };
}
