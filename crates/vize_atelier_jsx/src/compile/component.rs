use crate::JsxOutputMode;
use crate::scoped::ScopedStyle;
use crate::ssr::SsrComponent;
use crate::vapor::VaporComponent;
use crate::vdom::VdomComponent;

/// A component compiled through the mode-aware JSX pipeline.
pub enum JsxComponent {
    /// Compiled to VDOM output.
    Vdom(VdomComponent),
    /// Compiled to Vapor output.
    Vapor(VaporComponent),
    /// Compiled to SSR output.
    Ssr(SsrComponent),
}

impl JsxComponent {
    /// The component name recovered from the enclosing function, if any.
    pub fn component_name(&self) -> Option<&str> {
        match self {
            Self::Vdom(component) => component.component_name.as_deref(),
            Self::Vapor(component) => component.component_name.as_deref(),
            Self::Ssr(component) => component.component_name.as_deref(),
        }
    }

    /// The resolved client output mode for this component.
    pub fn mode(&self) -> JsxOutputMode {
        match self {
            Self::Vdom(_) => JsxOutputMode::Vdom,
            Self::Vapor(_) => JsxOutputMode::Vapor,
            Self::Ssr(component) => component.mode,
        }
    }

    /// Generated render code.
    pub fn code(&self) -> &str {
        match self {
            Self::Vdom(component) => component.code.as_str(),
            Self::Vapor(component) => component.code.as_str(),
            Self::Ssr(component) => component.code.as_str(),
        }
    }

    /// Runtime-helper preamble to place before [`Self::code`].
    ///
    /// Vapor and SSR currently inline their imports into the generated code.
    pub fn preamble(&self) -> &str {
        match self {
            Self::Vdom(component) => component.preamble.as_str(),
            Self::Vapor(_) | Self::Ssr(_) => "",
        }
    }

    /// Optional source map JSON.
    pub fn map(&self) -> Option<&str> {
        match self {
            Self::Vdom(component) => component.map.as_deref(),
            Self::Vapor(_) | Self::Ssr(_) => None,
        }
    }

    /// Optional extracted scoped style metadata for bundlers.
    pub fn scoped_style(&self) -> Option<&ScopedStyle> {
        match self {
            Self::Vdom(component) => component.scoped_style.as_ref(),
            Self::Vapor(component) => component.scoped_style.as_ref(),
            Self::Ssr(component) => component.scoped_style.as_ref(),
        }
    }
}
