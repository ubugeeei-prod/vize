# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/nix/scripts/install-kache.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
runHook preInstall
# kache and the engine it launches must come from the same release.
install -Dm755 kache $out/bin/kache
install -Dm755 kache-engine $out/bin/kache-engine
runHook postInstall
