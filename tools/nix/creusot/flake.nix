# The pinned Creusot toolchain for the Davinci deductive proofs.
#
# It is a flake of its own so the product flake, its lock and the stable
# Rust 1.100 build never see Creusot's nightly compiler. See
# docs/davinci/decisions/2026-11-17-creusot.md.
#
#   cd tests/formal/creusot
#   nix develop ../../../tools/nix/creusot --command cargo creusot prove --replay
#
# `free` is Creusot's solver bundle with the open-source Alt-Ergo 2.4.3
# (alongside Z3, CVC5 and CVC4); the default bundle's Alt-Ergo 2.6 carries a
# non-commercial licence.
{
  description = "Pinned Creusot toolchain for Davinci proofs";

  inputs.creusot.url = "github:creusot-rs/creusot/v0.14.0";

  outputs =
    { creusot, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAll =
        f:
        builtins.listToAttrs (
          map (system: {
            name = system;
            value = f system;
          }) systems
        );
    in
    {
      packages = forAll (system: {
        default = creusot.packages.${system}.free;
      });

      devShells = forAll (
        system:
        let
          pkgs = creusot.inputs.nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = [ creusot.packages.${system}.free ];
          };
        }
      );
    };
}
