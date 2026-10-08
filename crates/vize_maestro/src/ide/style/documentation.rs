//! Render only a selected immutable entry, preserving authored reference links.

use super::{
    data::{CssBaseline, CssEntry},
    resolve::Selected,
    vue,
};
use crate::ide::markup::{Markdown, link};

pub(super) fn markdown(selected: Selected) -> String {
    match selected {
        Selected::Vue(feature) => vue::markdown(feature),
        Selected::Entry(kind, entry) => {
            let mut doc = Markdown::new().title(entry.name).meta(match kind {
                "property" => "CSS property",
                "pseudo" => "CSS selector",
                _ => "CSS at-rule",
            });
            if let Some(description) = entry.description {
                doc = doc.paragraph(description);
            }
            if let Some(at_rule) = entry.at_rule {
                doc = doc.meta(&["Related at-rule: ", at_rule].concat());
            }
            if let Some(syntax) = entry.syntax {
                doc = doc.section("Syntax", &crate::ide::markup::code_block("css", syntax));
            }
            if entry.entry_type == Some("mediaFeature") {
                doc = doc.meta("CSS media feature");
            }
            if !entry.descriptors.is_empty() {
                doc = doc.section(
                    "Features",
                    &entry
                        .descriptors
                        .iter()
                        .map(|descriptor| descriptor.name)
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            }
            if kind == "property"
                && let Some(value) = entry.values.first()
            {
                doc = doc.example("css", &[entry.name, ": ", value.name, ";"].concat());
            }
            references(
                availability(doc, entry.status, entry.baseline, entry.browsers),
                entry,
            )
            .build()
        }
        Selected::Value(property, value) | Selected::Color(property, value) => {
            let mut doc = value_header(property, value.name, value.description);
            if matches!(selected, Selected::Color(_, _)) {
                doc = doc.docs(
                    "MDN CSS named colors",
                    "https://developer.mozilla.org/en-US/docs/Web/CSS/named-color",
                );
            }
            references(
                availability(doc, None, value.baseline, value.browsers),
                property,
            )
            .build()
        }
        Selected::Wide(property, value) | Selected::Function(property, value) => {
            let example = match selected {
                Selected::Function(_, value) if value.name == "var" => {
                    [property.name, ": var(--theme-value, inherit);"].concat()
                }
                Selected::Function(_, _) => {
                    [property.name, ": ", calc_example(property), ";"].concat()
                }
                _ => [property.name, ": ", value.name, ";"].concat(),
            };
            references(
                Markdown::new()
                    .title(selected.label())
                    .meta(&["CSS value for ", property.name].concat())
                    .paragraph(value.description)
                    .example("css", &example)
                    .docs("CSS specification", value.reference),
                property,
            )
            .build()
        }
    }
}

fn calc_example(property: &CssEntry) -> &'static str {
    if property.restrictions.contains(&"length") {
        "calc(1rem + 2px)"
    } else if property.restrictions.contains(&"percentage") {
        "calc(50% + 10%)"
    } else if property.restrictions.contains(&"angle") {
        "calc(90deg + 10deg)"
    } else if property.restrictions.contains(&"time") {
        "calc(1s + 250ms)"
    } else if property.restrictions.contains(&"frequency") {
        "calc(100Hz + 20Hz)"
    } else {
        "calc(1 + 1)"
    }
}

fn value_header(property: &CssEntry, name: &str, description: Option<&str>) -> Markdown {
    let mut doc = Markdown::new()
        .title(name)
        .meta(&["CSS value for ", property.name].concat());
    if let Some(description) = description {
        doc = doc.paragraph(description);
    } else if let Some(description) = property.description {
        doc = doc.section("Property", description);
    }
    doc = doc.example("css", &[property.name, ": ", name, ";"].concat());
    if let Some(syntax) = property.syntax {
        doc = doc.section(
            "Property syntax",
            &crate::ide::markup::code_block("css", syntax),
        );
    }
    doc
}

fn references(mut doc: Markdown, entry: &CssEntry) -> Markdown {
    if !entry.references.is_empty() {
        doc = doc.section(
            "Docs",
            &entry
                .references
                .iter()
                .map(|reference| link(reference.name, reference.url))
                .collect::<Vec<_>>()
                .join(" · "),
        );
    }
    doc
}

fn availability(
    mut doc: Markdown,
    status: Option<&str>,
    baseline: Option<CssBaseline>,
    browsers: &[&str],
) -> Markdown {
    if let Some(status) = status.filter(|status| *status != "standard") {
        doc = doc.section("Status", status);
    }
    if let Some(baseline) = baseline {
        let status = match baseline.status {
            "high" => "Widely available",
            "low" => "Newly available",
            _ => "Limited availability",
        };
        let mut text = status.to_owned();
        if let Some(date) = baseline.low_date {
            text.push_str("; interoperable since ");
            text.push_str(date);
        }
        if let Some(date) = baseline.high_date {
            text.push_str("; widely available since ");
            text.push_str(date);
        }
        doc = doc.section("Baseline (catalog snapshot)", &text);
    } else if !browsers.is_empty() {
        doc = doc.section("Browser versions (catalog snapshot)", &browsers.join(", "));
    }
    doc
}
