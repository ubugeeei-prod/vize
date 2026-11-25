# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/nix/scripts/xcrun.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [ "$#" -eq 3 ] && [ "$1" = "--sdk" ] && [ "$2" = "macosx" ] && [ "$3" = "--show-sdk-path" ]; then
  if [ -n "${SDKROOT:-}" ]; then
    printf '%s\n' "$SDKROOT"
    exit 0
  fi
fi

if [ "$#" -eq 2 ] && [ "$1" = "-f" ]; then
  case "$2" in
    clang | clang++)
      command -v "$2"
      exit $?
      ;;
  esac
fi

if [ -x /usr/bin/xcrun ]; then
  exec /usr/bin/xcrun "$@"
fi

printf 'unsupported xcrun invocation: %s\n' "$*" >&2
exit 127
