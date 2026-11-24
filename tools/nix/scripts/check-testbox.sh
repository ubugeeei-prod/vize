# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/nix/scripts/check-testbox.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
export VIZE_TESTBOX_SHELL=inherited
export VIZE_BLACKSMITH_BIN=/host/blacksmith
@CLEAR_TESTBOX@
test -z "${VIZE_TESTBOX_SHELL:-}"
test -z "${VIZE_BLACKSMITH_BIN:-}"
@ACTIVATE_TESTBOX@
test "$VIZE_TESTBOX_SHELL" = 1
test "$VIZE_BLACKSMITH_BIN" = "@BLACKSMITH@/bin/blacksmith"
test -x "$VIZE_BLACKSMITH_BIN"
touch "$out"
