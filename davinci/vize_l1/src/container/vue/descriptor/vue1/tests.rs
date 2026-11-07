use super::*;
use crate::container::{ContainerFormat, Vue};
use crate::dialect::vue1::surface::{self, TextRefusal};
use crate::dialect::vue1::text::TextBoundaryKind;
use crate::embed::syntax::EmbedHole;
use crate::markup::SurfaceParseOptions;
use crate::markup::{SurfaceChild, check_fidelity};
use alloc::format;
use oxc_span::GetSpan;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, Span, String};

mod counts;
mod custody;
mod refusals;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V1,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn resolved_descriptor_keeps_the_established_debug_surface() {
    let allocator = Allocator::default();
    let observation =
        Vue.observe_vue1_descriptor(&allocator, "<template><div/></template>", options());
    let debug = format!("{observation:#?}");
    assert!(!debug.contains("FileDialect"));
    assert!(!debug.contains("syntax: Standard"));
}
