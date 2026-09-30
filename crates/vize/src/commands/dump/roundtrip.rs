//! Typed roundtrip bindings. Dumps never become compiler pipeline inputs.

use vize_l0::dump::{Dump, Error as DumpError, Mode as DumpMode};
use vize_l0::{Allocator, String};
use vize_l2::dump::Page as L2Page;
use vize_l3::dump::Page as L3Page;

use super::Level;

pub(super) fn reprint(level: Level, input: &str) -> Result<String, DumpError> {
    match level {
        Level::L1 => {
            let allocator = Allocator::default();
            let (tree, _) = vize_l1::parse(&allocator, input);
            let mut printed = String::default();
            vize_l1::render::render(&tree, &mut |piece| printed.push_str(piece));
            Ok(printed)
        }
        Level::L2 => L2Page::parse(input).map(|page| page.print_to_string(DumpMode::Full)),
        Level::L3 => L3Page::parse(input).map(|page| page.print_to_string(DumpMode::Full)),
    }
}

pub(super) fn first_divergent_line(input: &str, printed: &str) -> usize {
    let mut input_lines = input.split('\n');
    let mut printed_lines = printed.split('\n');
    let mut line = 1;
    loop {
        match (input_lines.next(), printed_lines.next()) {
            (Some(input), Some(printed)) if input == printed => line += 1,
            _ => return line,
        }
    }
}
