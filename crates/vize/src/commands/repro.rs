//! `vize repro` - replay a crash repro folio (P2-13, TS-23).
//!
//! Reads a `repro.folio` written by a failed build, replays it through
//! [`davinci_ice::replay`], and compares the replayed failure against the
//! recorded one by **exact equality** - stage, pass and reason all
//! byte-equal, the assurance doctrine's "no partial matching" applied to the
//! ICE policy itself. The comparison lives in the tool, not only in its
//! tests: a repro that replays to a *different* failure is a finding, and
//! this command says so instead of printing something a human has to diff.
//!
//! Exit codes: 0 = reproduced (the replay failed identically), 1 = did not
//! reproduce (the replay completed, or failed differently), 2 = the file is
//! unreadable, malformed, or not replayable.

use std::path::PathBuf;

use clap::Args;

use super::davinci_ice::{self, IceFailure};
use vize_curator::repro::Page as ReproPage;
use vize_l0::dump::Dump;

#[derive(Args, Default)]
pub struct ReproArgs {
    /// Path to a repro.folio written by a failed build
    pub file: PathBuf,
}

pub fn run(args: ReproArgs) {
    let path = args.file.as_path();
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("repro: cannot read {}: {error}", path.display());
            std::process::exit(2);
        }
    };
    let folio = match ReproPage::parse(&text) {
        Ok(folio) => folio,
        Err(error) => {
            eprintln!("repro: {}: {error}", path.display());
            std::process::exit(2);
        }
    };
    let recorded = IceFailure {
        stage: folio.failed_stage.clone(),
        pass: folio.failed_pass.clone(),
        reason: folio.reason.clone(),
    };
    match davinci_ice::replay(&folio) {
        Err(message) => {
            eprintln!("repro: {}: {message}", path.display());
            std::process::exit(2);
        }
        Ok(None) => {
            eprintln!(
                "repro: did not reproduce: the pipeline completed (recorded {})",
                recorded.text()
            );
            std::process::exit(1);
        }
        Ok(Some(replayed)) => {
            if replayed == recorded {
                println!("repro: reproduced: {}", replayed.text());
            } else {
                eprintln!(
                    "repro: diverged: replayed {} (recorded {})",
                    replayed.text(),
                    recorded.text()
                );
                std::process::exit(1);
            }
        }
    }
}
