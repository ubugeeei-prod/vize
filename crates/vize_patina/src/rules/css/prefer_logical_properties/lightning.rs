//! Backward-compatible lightningcss registration for the exported rule.
//!
//! Built-in presets use the L1 structure implementation in the parent module.
//! This implementation preserves external `CssLinter::add_rule` callers.

use lightningcss::declaration::DeclarationBlock;
use lightningcss::properties::PropertyId;
use lightningcss::rules::CssRule as LightningRule;

use super::{META, PreferLogicalProperties};
use crate::rules::css::CssLintResult;
use crate::rules::css::declaration_positions::DeclarationPositions;

impl PreferLogicalProperties {
    pub(super) fn check_lightning_rule(
        &self,
        rule: &LightningRule,
        source: &str,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        match rule {
            LightningRule::Style(style_rule) => {
                let mut positions = DeclarationPositions::new(source, style_rule);
                self.check_lightning_declarations(
                    &style_rule.declarations,
                    &mut positions,
                    offset,
                    result,
                );
                for rule in &style_rule.rules.0 {
                    self.check_lightning_rule(rule, source, offset, result);
                }
            }
            LightningRule::Media(media) => {
                for rule in &media.rules.0 {
                    self.check_lightning_rule(rule, source, offset, result);
                }
            }
            LightningRule::Supports(supports) => {
                for rule in &supports.rules.0 {
                    self.check_lightning_rule(rule, source, offset, result);
                }
            }
            LightningRule::LayerBlock(layer) => {
                for rule in &layer.rules.0 {
                    self.check_lightning_rule(rule, source, offset, result);
                }
            }
            _ => {}
        }
    }

    fn check_lightning_declarations(
        &self,
        declarations: &DeclarationBlock,
        positions: &mut DeclarationPositions<'_>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        for declaration in declarations
            .declarations
            .iter()
            .chain(&declarations.important_declarations)
        {
            let property = match declaration.property_id() {
                PropertyId::MarginLeft => "margin-left",
                PropertyId::MarginRight => "margin-right",
                PropertyId::PaddingLeft => "padding-left",
                PropertyId::PaddingRight => "padding-right",
                PropertyId::BorderLeftWidth => "border-left-width",
                PropertyId::BorderRightWidth => "border-right-width",
                PropertyId::Left => "left",
                PropertyId::Right => "right",
                _ => continue,
            };
            let Some((start, end)) = positions.take(property, offset) else {
                continue;
            };
            result.warn(
                META.name,
                "Consider using logical properties for better RTL support",
                "Use logical properties like margin-inline-start instead of margin-left",
                start,
                end,
            );
        }
    }
}
