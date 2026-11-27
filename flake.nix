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
    # Compiles the repository's `.ush` programs. Their committed `compiled/*.sh`
    # run without it; tools/support/ci/ush-programs.ts checks this pin.
    ush = {
      url = "github:ubugeeei-prod/ush/v0.19.0";
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
        ./tools/nix/kache.nix
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
