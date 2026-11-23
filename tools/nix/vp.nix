{
  perSystem =
    { pkgs, ... }:
    let
      nodejs = pkgs.nodejs_24;
      pnpm = pkgs.pnpm;
    in
    {
      packages.vp = pkgs.writeShellApplication {
        name = "vp";
        runtimeInputs = [
          nodejs
          pnpm
        ];
        text = builtins.readFile ./scripts/vp.sh;
      };
    };
}
