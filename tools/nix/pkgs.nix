{ inputs, ... }:
{
  perSystem =
    { system, ... }:
    let
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
        # Only the vendor toolchain this flake pins may be unfree; nothing
        # else is allowed to pull unfree code in behind them.
        config.allowUnfreePredicate =
          pkg:
          builtins.elem (pkg.pname or null) [
            "blacksmith"
          ];
      };
    in
    {
      # Modules share one overlaid package set. Package builds retain the MSRV;
      # development follows the exact checked-in contributor toolchain.
      _module.args = {
        inherit pkgs;
        # Pinned here rather than read from `rust-toolchain.toml`: the flake
        # builds against the `rust-version` the workspace declares (1.99.1),
        # which that file may run ahead of when contributors adopt a newer toolchain.
        rustToolchain = pkgs.rust-bin.stable."1.99.1".default.override {
          extensions = [
            "clippy"
            "rust-src"
            "rustfmt"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };
        rustDevToolchain = pkgs.rust-bin.fromRustupToolchainFile ../../rust-toolchain.toml;
      };
    };
}
