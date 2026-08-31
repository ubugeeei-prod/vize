{ inputs, ... }:
{
  perSystem =
    { system, ... }:
    let
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
        # Only the two vendor toolchains this flake pins may be unfree; nothing
        # else is allowed to pull unfree code in behind them.
        config.allowUnfreePredicate =
          pkg:
          builtins.elem (pkg.pname or null) [
            "blacksmith"
            "moonbit"
          ];
      };
    in
    {
      # Every module that needs a package set needs the same overlaid one, and
      # every module that needs a compiler needs the same pinned toolchain.
      # Both are resolved once here and handed out as module arguments.
      _module.args = {
        inherit pkgs;
        # Pinned here rather than read from `rust-toolchain.toml`: the flake
        # builds against the `rust-version` the workspace declares (1.95.0),
        # while that file tracks the newer toolchain contributors run locally.
        rustToolchain = pkgs.rust-bin.stable."1.95.0".default.override {
          extensions = [
            "clippy"
            "rust-src"
            "rustfmt"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      };
    };
}
