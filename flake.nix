{
  description = "Vize development environment and CLI flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import ./tools/nix/systems.nix;

      imports = [
        ./tools/nix/blacksmith.nix
        ./tools/nix/dev-shell.nix
        ./tools/nix/moonbit.nix
        ./tools/nix/packages.nix
        ./tools/nix/pkgs.nix
        ./tools/nix/vp.nix
      ];

      perSystem =
        { pkgs, ... }:
        {
          formatter = pkgs.nixfmt;
        };
    };
}
