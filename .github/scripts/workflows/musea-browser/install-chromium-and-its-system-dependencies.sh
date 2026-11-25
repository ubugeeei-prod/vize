#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/musea-browser/install-chromium-and-its-system-dependencies.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# Browser dependency resolution needs package indices, not desktop metadata.
sudo tee /etc/apt/apt.conf.d/81-vize-musea-package-indexes >/dev/null <<'APT'
Acquire::Languages "none";
Acquire::IndexTargets::deb::DEP-11::DefaultEnabled "false";
Acquire::IndexTargets::deb::CNF::DefaultEnabled "false";
APT
vize-ci-apt-retry vp exec --filter './npm/builder/vite-musea' -- playwright install --with-deps chromium
