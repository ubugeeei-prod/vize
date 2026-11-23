//! Retained host snapshots and cancellable source queries.
//!
//! These adapters use actual open-document revisions. They do not select a
//! production request route or turn a host snapshot into native File admission.

#[cfg(feature = "retained-program")]
pub mod navigation;
mod project;
mod query;
mod snapshot;

#[cfg(feature = "experimental-source-navigation")]
pub(crate) use project::ModuleLinkPublicationError;
pub use project::{ProjectQuery, ProjectQueryResult, SourceQueryProject};
pub use query::{SourceQuery, SourceQueryResult};
pub use snapshot::{SnapshotKey, SnapshotRefusal, SourceSnapshot, SourceSnapshotCache};

#[cfg(feature = "experimental-resident-blocks")]
pub mod blocks;

#[cfg(feature = "experimental-source-project")]
mod edit;
#[cfg(feature = "experimental-source-project")]
pub use edit::SourceEditRefusal;

#[cfg(all(test, feature = "experimental-source-project"))]
mod edit_tests;
#[cfg(all(test, feature = "experimental-source-project"))]
mod program_tests;
#[cfg(test)]
mod tests;

#[cfg(feature = "experimental-versioned-fixes")]
mod positioned;

#[cfg(feature = "experimental-script-queries")]
mod script_queries;
#[cfg(feature = "experimental-script-queries")]
pub use script_queries::{ScriptModuleSpecifier, ScriptQueryProducts, ScriptQueryRefusal};

#[cfg(feature = "experimental-script-tools")]
pub use script_queries::tools::{ScriptBlockLint, ScriptToolAnalysis, ScriptToolProducts};

#[cfg(feature = "experimental-scoped-component-session")]
mod scoped_component;
#[cfg(feature = "experimental-scoped-component-session")]
pub use scoped_component::ScopedComponentQueryError;

#[cfg(feature = "experimental-scss-definitions")]
mod scss;
#[cfg(feature = "experimental-scss-definitions")]
pub use scss::ScssDefinitionError;

#[cfg(feature = "experimental-verified-style-format")]
mod style_format;
#[cfg(feature = "experimental-verified-style-format")]
pub use style_format::{PreparedScss, StyleFormatError, VerifiedStyleFormat};
#[cfg(feature = "experimental-literal-products")]
pub mod literal_products;

#[cfg(feature = "experimental-recovered-literal-prefix")]
pub mod recovered_script;

#[cfg(all(
    test,
    feature = "experimental-recovered-literal-prefix",
    feature = "experimental-verified-style-format",
    feature = "experimental-style-lint-products"
))]
mod toolchain_session;
#[cfg(feature = "experimental-resident-template-passes")]
mod template;
#[cfg(feature = "experimental-resident-template-passes")]
pub use template::TemplateRequestError;
