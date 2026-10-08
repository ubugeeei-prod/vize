use super::*;
use crate::SurfaceParseOptions;
use crate::container::{ContainerFormat, Vue};
use crate::dialect::vue1::surface::{self, TextRefusal};
use crate::dialect::vue1::text::TextBoundaryKind;
use crate::embed::syntax::EmbedHole;
use crate::{SurfaceChild, check_fidelity};
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
