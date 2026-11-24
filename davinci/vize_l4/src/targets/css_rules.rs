//! Write ordered CSS rules and their original source ranges.
#[cfg(feature = "scoped-expanded-css")]
use crate::module::ScopeId;
use crate::write::EmitDocument;
use vize_l0::{Span, String};
#[cfg(feature = "scoped-expanded-css")]
use vize_l2::style::ScopeAtSelectorEnd;
use vize_l2::style::{ExpandedRules, Property, RuleRun};

pub fn emit<'a>(program: &impl ExpandedRules<'a>) -> String {
    let rules = program.runs();
    if rules.is_empty() {
        return String::default();
    }
    let mut css = EmitDocument::new(false);
    write::<false>(
        &mut css,
        program,
        rules,
        #[cfg(feature = "scoped-expanded-css")]
        None,
    );
    css.into_string()
}

/// Append one expanded stylesheet directly to its final CSS document.
/// Each selector and property name is mapped. The lowering supplies value links.
pub fn append<'a>(css: &mut EmitDocument, program: &impl ExpandedRules<'a>) {
    append_rules(
        css,
        program,
        #[cfg(feature = "scoped-expanded-css")]
        None,
    );
}

/// Scope the rightmost compound of every selector in the admitted rule program.
///
/// ```compile_fail,E0277
/// use vize_l2::style::ExpandedRules;
/// use vize_l4::{module::ScopeId, targets::css_rules, write::EmitDocument};
/// fn unclassified<'a>(css: &mut EmitDocument, rules: &impl ExpandedRules<'a>, id: ScopeId<'_>) {
///     css_rules::append_scoped(css, rules, id, false);
/// }
/// ```
#[cfg(feature = "scoped-expanded-css")]
pub fn append_scoped<'a>(
    css: &mut EmitDocument,
    program: &impl ScopeAtSelectorEnd<'a>,
    id: ScopeId<'_>,
    trim: bool,
) {
    append_rules(css, program, Some((id, trim)));
}

fn append_rules<'a>(
    css: &mut EmitDocument,
    program: &impl ExpandedRules<'a>,
    #[cfg(feature = "scoped-expanded-css")] scope: Option<(ScopeId<'_>, bool)>,
) {
    let rules = program.runs();
    if rules.is_empty() {
        return;
    }
    reserve_style(
        css,
        rules,
        #[cfg(feature = "scoped-expanded-css")]
        scope,
    );
    if css.is_recording() {
        write::<true>(
            css,
            program,
            rules,
            #[cfg(feature = "scoped-expanded-css")]
            scope,
        );
    } else {
        write::<false>(
            css,
            program,
            rules,
            #[cfg(feature = "scoped-expanded-css")]
            scope,
        );
    }
}

fn reserve_style(
    css: &mut EmitDocument,
    rules: &[RuleRun<'_>],
    #[cfg(feature = "scoped-expanded-css")] scope: Option<(ScopeId<'_>, bool)>,
) {
    // Reserve a complete style at its join point. A stream of small writes
    // otherwise grows the final buffer past the capacity of the fragment join.
    let mut bytes = 0;
    let mut group = None;
    for rule in rules {
        if let Some(previous) = group {
            let gap = previous != rule.group;
            #[cfg(feature = "scoped-expanded-css")]
            let gap = gap && !scope.is_some_and(|(_, trim)| trim);
            bytes += 1 + usize::from(gap);
        }
        #[cfg(feature = "scoped-expanded-css")]
        if let Some((id, _)) = scope {
            bytes += id.as_str().len() + 2;
        }
        bytes += rule.selector.len() + " {\n}".len();
        for property in &rule.declarations {
            bytes +=
                "  ".len() + property.name.len() + ": ".len() + property.value.len() + ";\n".len();
        }
        group = Some(rule.group);
    }
    css.reserve_text(bytes);
}

fn write<'a, const MAPPED: bool>(
    css: &mut EmitDocument,
    program: &impl ExpandedRules<'a>,
    rules: &[RuleRun<'a>],
    #[cfg(feature = "scoped-expanded-css")] scope: Option<(ScopeId<'_>, bool)>,
) {
    let mut value_links = MAPPED.then(|| program.value_links());
    let mut previous_group = None;
    for rule in rules {
        if let Some(previous) = previous_group {
            css.push_char('\n');
            let gap = previous != rule.group;
            #[cfg(feature = "scoped-expanded-css")]
            let gap = gap && !scope.is_some_and(|(_, trim)| trim);
            if gap {
                css.push_char('\n');
            }
        }
        if MAPPED {
            css.push_linked(&rule.selector, rule.selector_span);
        } else {
            css.push_str(&rule.selector);
        }
        #[cfg(feature = "scoped-expanded-css")]
        if let Some((id, _)) = scope {
            // The scoped entry requires the final compound to accept this suffix.
            css.push_char('[');
            css.push_str(id.as_str());
            css.push_char(']');
        }
        if let Some((first, rest)) = rule.declarations.split_first() {
            css.push_str(" {\n  ");
            write_property::<MAPPED>(css, first, &mut value_links);
            for property in rest {
                css.push_str(";\n  ");
                write_property::<MAPPED>(css, property, &mut value_links);
            }
            css.push_str(";\n}");
        } else {
            css.push_str(" {\n}");
        }
        previous_group = Some(rule.group);
    }
}

#[inline]
fn write_property<'a, const MAPPED: bool>(
    css: &mut EmitDocument,
    property: &Property<'a>,
    value_links: &mut Option<impl FnMut(&Property<'a>) -> Option<Span>>,
) {
    if MAPPED {
        css.push_linked(property.name, property.name_span);
    } else {
        css.push_str(property.name);
    }
    css.push_str(": ");
    if let Some(span) = value_links.as_mut().and_then(|locate| locate(property)) {
        css.push_linked(&property.value, span);
    } else {
        css.push_str(&property.value);
    }
}
