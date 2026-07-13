use vize_carton::{String, cstr};
use vize_rendu::{RenderEmitSettings, RenderOutputMode};

/// Owned JavaScript sections and the static HTML templates referenced by them.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaporEmitResult {
    pub code: String,
    pub preamble: String,
    pub body: String,
    pub templates: std::vec::Vec<String>,
}

pub(super) fn preamble(helpers: &[&str], settings: &RenderEmitSettings) -> String {
    if helpers.is_empty() {
        return String::default();
    }
    let imports = helpers
        .iter()
        .map(|helper| cstr!("{helper} as _{helper}"))
        .collect::<std::vec::Vec<_>>()
        .join(", ");
    match settings.mode {
        RenderOutputMode::Module => cstr!(
            "import {{ {} }} from '{}';\n",
            imports.as_str(),
            settings.runtime_module_name
        ),
        RenderOutputMode::Function => cstr!(
            "const {{ {} }} = {}\n",
            imports.replace(" as ", ": "),
            settings.runtime_global_name
        ),
    }
}
