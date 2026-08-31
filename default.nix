{
  craneLib,
  pkgs,
  root ? ./.,
}:
pkgs.callPackage ./tools/nix/package.nix {
  inherit craneLib root;
}
