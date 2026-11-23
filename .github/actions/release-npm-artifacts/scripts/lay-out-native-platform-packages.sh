#!/usr/bin/env bash
vp exec napi create-npm-dirs
vp exec napi artifacts --output-dir artifacts
vp exec napi pre-publish -t npm --no-gh-release --skip-optional-publish
