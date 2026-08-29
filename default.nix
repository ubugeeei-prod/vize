{
  craneLib,
  pkgs,
  root ? ./.,
}:
pkgs.callPackage ./package.nix {
  inherit craneLib root;
}
