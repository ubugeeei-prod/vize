#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/setup-wild-http-recovery/scripts/install-wild.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
case "$RUNNER_ARCH" in
  X64)   target_arch="x86_64" ;;
  ARM64) target_arch="aarch64" ;;
  *) echo "Unsupported architecture: $RUNNER_ARCH" >&2; exit 1 ;;
esac

mkdir "${RUNNER_TEMP}/wild-install"

url="https://github.com/wild-linker/wild/releases/download/0.9.0/wild-linker-0.9.0-${target_arch}-unknown-linux-gnu.tar.gz"
archive="${RUNNER_TEMP}/wild.tar.gz"
. "$GITHUB_ACTION_PATH/download-linux-x64.sh" "${archive}" "${url}" "5"

tar -xzf "${archive}" -C "${RUNNER_TEMP}/wild-install" --strip-components=1
rm -f "${archive}"

mkdir -p "$HOME/.cargo"
for libc in gnu musl; do
  TARGET="target.${target_arch}-unknown-linux-${libc}"
  if ! grep "${TARGET}" "$HOME/.cargo/config.toml" 2>/dev/null; then
        echo -e "# Added by wild GitHub Action\n[${TARGET}]\nlinker = \"clang\"\nrustflags = [\"-Clink-arg=--ld-path=${RUNNER_TEMP}/wild-install/wild\"]\n" >>"$HOME/.cargo/config.toml"
  fi
done
