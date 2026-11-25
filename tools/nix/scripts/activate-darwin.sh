# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/nix/scripts/activate-darwin.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
export VIZE_DARWIN_LIBICONV_LIB="@LIBICONV@/lib"
export LIBRARY_PATH="$VIZE_DARWIN_LIBICONV_LIB${LIBRARY_PATH:+:$LIBRARY_PATH}"
export RUSTFLAGS="-L native=$VIZE_DARWIN_LIBICONV_LIB${RUSTFLAGS:+ $RUSTFLAGS}"
