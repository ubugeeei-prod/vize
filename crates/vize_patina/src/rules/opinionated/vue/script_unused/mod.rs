//! Template-aware script unused-vars and prefer-const.

mod classify;
mod decide;
mod exports;
mod no_unused_script_vars;
mod options;
mod prefer_const;
mod run;
mod template_use;

pub(super) use no_unused_script_vars::NoUnusedScriptVars;
pub(super) use prefer_const::PreferConst;

#[cfg(test)]
mod tests;
