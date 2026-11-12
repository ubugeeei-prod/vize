//! Template-aware script unused-vars and prefer-const.

mod art_use;
mod classify;
mod decide;
mod exports;
mod no_unused_script_vars;
mod options;
mod prefer_const;
mod run;
mod sfc_use;
mod template_use;
mod usage;

pub(crate) use no_unused_script_vars::NoUnusedScriptVars;
pub(crate) use prefer_const::PreferConst;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_art;
#[cfg(test)]
mod tests_issue;
#[cfg(test)]
mod tests_tse;
