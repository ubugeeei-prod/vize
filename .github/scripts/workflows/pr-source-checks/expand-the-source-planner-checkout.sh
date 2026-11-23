#!/usr/bin/env bash
git sparse-checkout set --no-cone --stdin < .github/source-plan-sparse-checkout
