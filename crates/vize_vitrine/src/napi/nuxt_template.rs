//! Template facts for the checks Nuxt runs as a Vue compiler node transform.
//!
//! Nuxt 4.7 checks templates against its app (page routes, server components)
//! in a node transform, which Vize's compiler never runs. This reads each SFC
//! the way Nuxt's transform reads Vue's parsed template and returns what it
//! would find, before the app-dependent part: whether a link target matches a
//! route and whether a tag is a server component stay Nuxt's own decision, so
//! `@vizejs/nuxt` hands these findings back to Nuxt's transform.
#![cfg_attr(
    feature = "napi",
    expect(
        clippy::disallowed_types,
        reason = "N-API values cross the boundary as std `String`s"
    )
)]

mod findings;
#[cfg(test)]
mod tests;
mod values;

#[cfg(feature = "napi")]
use napi_derive::napi;

pub(crate) use findings::{Finding, nuxt_template_findings};

/// A template construct Nuxt's template node transform reports.
#[cfg(feature = "napi")]
#[napi(object)]
pub struct NuxtTemplateFindingNapi {
    /// `link` (a static `<NuxtLink>` target), `event` (`v-on`/`v-model` on a
    /// candidate server component), `prop` (an unserializable prop on one) or
    /// `island` (an unserializable `<NuxtIsland>` `props`/`context` entry).
    pub kind: String,
    /// The element's tag as authored.
    pub tag: String,
    /// One-based line of the attribute in the SFC.
    pub line: u32,
    /// One-based UTF-16 column of the attribute.
    pub column: u32,
    /// UTF-16 offset of the attribute in the SFC.
    pub offset: u32,
    /// `link`: the attribute holding the target (`to` or `href`).
    pub attribute: Option<String>,
    /// `link`: the target as written.
    pub to: Option<String>,
    /// `event`: the directive as authored, e.g. `@click.stop`.
    pub directive: Option<String>,
    /// `prop`/`island`: the prop or entry name.
    pub prop: Option<String>,
    /// `prop`/`island`: what the value is (`a function`, `a class`, ...).
    pub value: Option<String>,
    /// `island`: the static `name` attribute, when it has a value.
    pub island_name: Option<String>,
    /// `island`: the bound prop (`props` or `context`).
    pub island_prop: Option<String>,
}

#[cfg(feature = "napi")]
impl From<Finding> for NuxtTemplateFindingNapi {
    fn from(finding: Finding) -> Self {
        let owned = |value: Option<vize_l0::String>| value.map(|value| value.as_str().into());
        Self {
            kind: finding.kind.as_str().into(),
            tag: finding.tag.as_str().into(),
            line: finding.line,
            column: finding.column,
            offset: finding.offset,
            attribute: owned(finding.attribute),
            to: owned(finding.to),
            directive: owned(finding.directive),
            prop: owned(finding.prop),
            value: finding.value.map(Into::into),
            island_name: owned(finding.island_name),
            island_prop: owned(finding.island_prop),
        }
    }
}

/// Find the constructs Nuxt's template diagnostics check in an SFC.
///
/// `serverTags` names the tags of the app's server components; `<NuxtLink>`
/// and `<NuxtIsland>` are always checked. An SFC that does not parse, or whose
/// template is external or not HTML, yields nothing.
#[cfg(feature = "napi")]
#[napi(js_name = "nuxtTemplateDiagnostics")]
pub fn nuxt_template_diagnostics(
    source: String,
    server_tags: Option<Vec<String>>,
) -> Vec<NuxtTemplateFindingNapi> {
    let server_tags = server_tags.unwrap_or_default();
    let server_tags: Vec<&str> = server_tags.iter().map(String::as_str).collect();
    nuxt_template_findings(&source, &server_tags)
        .into_iter()
        .map(Into::into)
        .collect()
}
