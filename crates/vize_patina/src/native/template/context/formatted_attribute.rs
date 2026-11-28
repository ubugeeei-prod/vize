//! Formatted help retains the same checked-attribute reporting authority.

use super::{NativeTemplateAttribute, NativeTemplateLintContext, NativeTemplateLintRefusal};
use crate::Severity;
use vize_carton::i18n::t_fmt;

impl<'a> NativeTemplateLintContext<'_, 'a> {
    /// Report an original checked attribute with independently formatted message
    /// and help variables. The existing unformatted-help reporters keep their
    /// contract; this capability grants no caller-provided range or fix.
    ///
    /// ```compile_fail
    /// use vize_l0::Span;
    /// use vize_patina::native::template::NativeTemplateLintContext;
    /// fn raw(context: &mut NativeTemplateLintContext<'_, '_>) {
    ///     context.warn_attribute_with_formatted_help(Span::new(0, 7), "message", &[], "help", &[]);
    /// }
    /// ```
    pub fn warn_attribute_with_formatted_help(
        &mut self,
        attribute: &NativeTemplateAttribute<'_, 'a>,
        message_key: &str,
        message_variables: &[(&str, &str)],
        help_key: &str,
        help_variables: &[(&str, &str)],
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(attribute.original().component(), self.root.component()) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        self.report_translated_with_help(
            Severity::Warning,
            attribute.range(),
            t_fmt(self.linter.locale, message_key, message_variables),
            t_fmt(self.linter.locale, help_key, help_variables).as_str(),
        );
        Ok(())
    }
}
