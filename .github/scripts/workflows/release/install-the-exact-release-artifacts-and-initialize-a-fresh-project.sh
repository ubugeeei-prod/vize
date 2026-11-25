#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/install-the-exact-release-artifacts-and-initialize-a-fresh-project.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
evidence="$RUNNER_TEMP/release-editor-evidence"
install_root="$RUNNER_TEMP/release-editor-install"
fresh="$RUNNER_TEMP/release-editor-fresh-project"
mkdir -p "$evidence" "$install_root/cli" "$install_root/npm" "$fresh/src" target/source-editors

tar -xzf "$RUNNER_TEMP/release-cli/vize-x86_64-unknown-linux-gnu.tar.gz" \
  -C "$install_root/cli"
server="$install_root/cli/vize"
test -x "$server"
test "$("$server" --version)" = "vize $RELEASE_VERSION"

for editor in zed nvim vim helix emacs; do
  mv "editors/$editor" "target/source-editors/$editor"
  tar -xzf "$RUNNER_TEMP/release-editors/${editor}-vize-extension.tar.gz" \
    -C editors
  test -d "editors/$editor"
done
vsix="$RUNNER_TEMP/release-editors/editors/vscode/dist/vize.vsix"
test -s "$vsix"

test -f "$RUNNER_TEMP/release-npm/cli/package.json"
test "$(node "$VIZE_CI_FILES/read-package-version.cjs" "$RUNNER_TEMP/release-npm/cli/package.json")" = \
  "$RELEASE_VERSION"
npm pack "$RUNNER_TEMP/release-npm/cli" \
  --pack-destination "$install_root/npm" --ignore-scripts >/dev/null
cli_archive=$(find "$install_root/npm" -maxdepth 1 -type f -name '*.tgz' -print)
test "$(printf '%s\n' "$cli_archive" | wc -l)" -eq 1
mkdir -p "$install_root/npm/package"
tar -xzf "$cli_archive" -C "$install_root/npm/package" --strip-components=1
packaged_cli="$install_root/npm/package/bin/vize"
test -f "$packaged_cli"
test "$(node "$VIZE_CI_FILES/read-package-version.cjs" "$install_root/npm/package/package.json")" = \
  "$RELEASE_VERSION"

node "$VIZE_CI_FILES/install-the-exact-release-artifacts-and-initialize-a-fresh-project.ts" "$fresh"
node "$packaged_cli" init "$fresh" --yes --no-lint --vite --fmt --typecheck --editor --no-install \
  | tee "$evidence/init.txt"
dependency_root="$RUNNER_TEMP/release-editor-dependencies"
mkdir -p "$dependency_root"
npm install --prefix "$dependency_root" --ignore-scripts --no-audit --no-fund \
  --package-lock=false --save=false vue@3.5.34 typescript@6.0.3 \
  typescript-native@npm:typescript@7.1.0-beta
mv "$dependency_root/node_modules" "$fresh/node_modules"
native_resolver="$fresh/node_modules/typescript-native/lib/getExePath.js"
test -f "$native_resolver"
native_checker=$(node "$VIZE_CI_FILES/install-the-exact-release-artifacts-and-initialize-a-fresh-project-2.ts" \
  "$native_resolver")
test -x "$native_checker"
fresh_real=$(readlink -f "$fresh")
case "$(readlink -f "$native_checker")" in
  "$fresh_real/node_modules"/*) ;;
  *)
    echo "native checker escapes the fresh project: $native_checker" >&2
    exit 1
    ;;
esac
test -f "$fresh/.vscode/extensions.json"
grep -F 'ubugeeei.vize' "$fresh/.vscode/extensions.json"
set +e
"$server" lint "$fresh/src/Lint.vue" --format json --no-config \
  > "$evidence/diagnostics.json"
lint_status=$?
set -e
test "$lint_status" -eq 1
grep -F 'vue/require-v-for-key' "$evidence/diagnostics.json"
find "$fresh" -type l -print > "$evidence/project-links.txt"
find "$fresh" -path "$fresh/node_modules" -prune -o -type l -print \
  > "$evidence/project-links-outside-node-modules.txt"
test ! -s "$evidence/project-links-outside-node-modules.txt"
find "$fresh/node_modules" -type l -print > "$evidence/dependency-links.txt"
while IFS= read -r link; do
  target=$(readlink -f "$link")
  case "$target" in
    "$fresh/node_modules"/*) ;;
    *)
      echo "installed dependency link escapes the fresh project: $link -> $target" >&2
      exit 1
      ;;
  esac
done < "$evidence/dependency-links.txt"
if grep -R -F -n "$GITHUB_WORKSPACE" "$fresh" > "$evidence/workspace-links.txt"; then
  echo "fresh release project refers to the source checkout" >&2
  exit 1
fi
sha256sum "$server" "$vsix" "$RUNNER_TEMP/release-editors/"*.tar.gz \
  > "$evidence/artifacts.sha256"
find "$fresh" -type f -print | LC_ALL=C sort > "$evidence/project-files.txt"
printf 'server=%s\n' "$server" >> "$GITHUB_OUTPUT"
printf 'vsix=%s\n' "$vsix" >> "$GITHUB_OUTPUT"
printf 'fresh=%s\n' "$fresh" >> "$GITHUB_OUTPUT"
printf 'evidence=%s\n' "$evidence" >> "$GITHUB_OUTPUT"
