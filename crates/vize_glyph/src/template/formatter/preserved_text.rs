//! Markup formatting when the compiler preserves authored text whitespace.
use super::{TemplateFormatter, parse_interpolation_range};
use crate::{error::FormatError, template::helpers};
use memchr::memchr2;
use vize_l0::String;

impl TemplateFormatter<'_> {
    pub(crate) fn format_preserving_text(&self, source: &[u8]) -> Result<String, FormatError> {
        let mut output = Vec::with_capacity(source.len());
        let mut pos = 0;
        // The retained source is already inside the SFC template block.
        let mut depth: usize = 1;
        while pos < source.len() {
            let Some(offset) = memchr2(b'<', b'{', source.get(pos..).unwrap_or_default()) else {
                output.extend_from_slice(source.get(pos..).unwrap_or_default());
                break;
            };
            let start = pos + offset;
            output.extend_from_slice(source.get(pos..start).unwrap_or_default());
            pos = start;

            // A '<' in an expression or literal cannot acquire tag ownership.
            if source.get(pos..pos + 2) == Some(b"{{".as_slice())
                && let Some((_, _, end)) = parse_interpolation_range(source, pos)
            {
                output.extend_from_slice(source.get(pos..end).unwrap_or_default());
                pos = end;
                continue;
            }
            if source.get(pos..pos + 4) == Some(b"<!--".as_slice()) {
                let end = helpers::find_bytes(source.get(pos..).unwrap_or_default(), b"-->")
                    .map_or(source.len(), |offset| pos + offset + 3);
                output.extend_from_slice(source.get(pos..end).unwrap_or_default());
                pos = end;
                continue;
            }
            if source.get(pos..pos + 2) == Some(b"</".as_slice())
                && let Some((tag_name, end)) = helpers::parse_closing_tag(source, pos)
                && source.get(end.saturating_sub(1)) == Some(&b'>')
            {
                output.extend_from_slice(b"</");
                output.extend_from_slice(tag_name.as_bytes());
                output.push(b'>');
                depth = depth.saturating_sub(1);
                pos = end;
                continue;
            }
            if source.get(pos) == Some(&b'<')
                && let Some((tag_name, mut attrs, self_closing, end)) =
                    self.parse_opening_tag(source, pos)
                && source.get(end.saturating_sub(1)) == Some(&b'>')
            {
                if self.options.sort_attributes {
                    super::sort_attributes(&mut attrs, self.options);
                }
                output.push(b'<');
                output.extend_from_slice(tag_name.as_bytes());
                let own_line = self.write_opening_attributes(&mut output, &tag_name, &attrs, depth);
                if self_closing {
                    output.extend_from_slice(if own_line { b"/>" } else { b" />" });
                } else {
                    if super::is_whitespace_significant_element(&tag_name, &attrs) {
                        // v-pre and native raw text retain nested markup too.
                        let content_start = output.len() + 1;
                        let close = self.copy_whitespace_significant_element(
                            source,
                            end,
                            &tag_name,
                            source.len(),
                            &mut output,
                        );
                        // The ordinary layout owns a newline after a complete
                        // close. Here only the source owns that boundary. An
                        // unclosed region is copied exactly and owns its tail.
                        if output.get(content_start..) != source.get(end..close)
                            && output.ends_with(self.newline)
                        {
                            output.truncate(output.len() - self.newline.len());
                        }
                        pos = close;
                        continue;
                    }
                    output.push(b'>');
                    if !helpers::is_void_element_str(&tag_name) {
                        depth += 1;
                    }
                }
                pos = end;
                continue;
            }
            output.extend_from_slice(source.get(pos..pos + 1).unwrap_or_default());
            pos += 1;
        }
        // Every source slice has ASCII boundaries; tag/attribute fragments are UTF-8.
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }
}

#[cfg(test)]
mod tests {
    use crate::{FormatOptions, VueVersion, template::format_template_content_preserving_text};

    #[test]
    fn incomplete_regions_retain_the_authored_tail() {
        for source in [
            "<p>  body </p\n",
            "<pre>  tail\n",
            "<pre><pre> nested </pre>\n",
            "<pre> body </pre\n",
            "<div v-pre><div> nested </div>\n",
        ] {
            let options = FormatOptions::default();
            let mut output = source.into();
            for _ in 0..3 {
                output = format_template_content_preserving_text(&output, &options, VueVersion::V3)
                    .unwrap();
                assert_eq!(output.as_str(), source);
            }
        }
    }
}
