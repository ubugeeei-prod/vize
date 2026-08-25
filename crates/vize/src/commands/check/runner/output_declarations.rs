use std::{collections::BTreeSet, time::Instant};
use vize_s0::{String, cstr};

use super::{
    super::resolve_declaration_emit_options, CheckArgs, DeclarationSummary, ProgramExecution,
};

pub(super) fn emit_declarations(
    args: &CheckArgs,
    executions: &[ProgramExecution],
    total_errors: usize,
) -> Result<Option<DeclarationSummary>, String> {
    if !args.declaration {
        return Ok(None);
    }
    if total_errors > 0 {
        if !args.quiet {
            eprintln!("Skipping declaration emit because type errors were reported.");
        }
        return Ok(None);
    }

    let start = Instant::now();
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    for execution in executions {
        let options = resolve_declaration_emit_options(
            args.declaration_dir.as_deref(),
            execution.tsconfig_path.as_deref(),
            &execution.program_root,
        );
        directories.insert(options.out_dir.clone());
        let result = execution
            .checker
            .emit_declarations(&options)
            .map_err(|error| cstr!("{}", error))?;
        files.extend(result.files.into_iter().map(|file| file.path));
    }
    Ok(Some(DeclarationSummary {
        files,
        directories,
        elapsed: start.elapsed(),
    }))
}
