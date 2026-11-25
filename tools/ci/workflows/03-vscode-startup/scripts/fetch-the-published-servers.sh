#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-vscode-startup/scripts/fetch-the-published-servers.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p ext-builds/server && cd ext-builds/server
for archive in vize-x86_64-unknown-linux-gnu.tar.gz vize-x86_64-pc-windows-msvc.zip; do
  for attempt in 1 2 3 4 5; do
    gh release download "v$SERVER_VERSION" --repo "$GITHUB_REPOSITORY" --clobber \
      --pattern "$archive" --pattern "$archive.sha256" && break
    [[ $attempt == 5 ]] && exit 1
    sleep 15
  done
  echo "$(cut -d' ' -f1 "$archive.sha256")  $archive" | sha256sum --check
done
tar -xzf vize-x86_64-unknown-linux-gnu.tar.gz && unzip -q vize-x86_64-pc-windows-msvc.zip
rm -f ./*.tar.gz ./*.zip ./*.sha256
./vize --version && ls -l
