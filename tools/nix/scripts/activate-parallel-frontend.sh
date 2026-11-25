# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/nix/scripts/activate-parallel-frontend.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
export VIZE_JOBS_FRONTEND="${VIZE_JOBS_FRONTEND:-8}"
export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-Zunstable-options --jobs-frontend=$VIZE_JOBS_FRONTEND"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$VIZE_WORKSPACE_ROOT/target/nightly-frontend-j$VIZE_JOBS_FRONTEND}"
echo "Parallel frontend: $(rustc --version), --jobs-frontend=$VIZE_JOBS_FRONTEND, target $CARGO_TARGET_DIR"
echo "Not for CI, release builds or measurements; run the final clippy and tests in the default shell."
