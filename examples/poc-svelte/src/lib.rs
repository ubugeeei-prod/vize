//! Proof of concept: a Svelte 5 component compiled through the level crates.
//!
//! [`container`] splits the file with an L1 container format, the L1 markup
//! parser reads the template in place, [`dialect`] lowers Svelte's tags and
//! directives to neutral L2 ops, the shared L2 to L3 lowering decides what is
//! reactive, and [`target`] writes a `svelte/internal/client` module. [`runes`]
//! turns the instance script's runes into runtime signal accesses.
//! Not a supported product surface (2026-11-03 scope decision).

mod container;
mod dialect;
mod runes;
mod target;

use poc_support::{Error, View, dedent, indent};
use vize_l0::{Allocator, Span, String, append, cstr};
use vize_l1::container::ContainerFormat;
use vize_l4::write::{NoLinks, Writer};

/// Compile one `.svelte` component to a JavaScript module.
pub fn compile(source: &str) -> Result<String, Error> {
    let allocator = Allocator::default();
    let file = container::Svelte.split(&allocator, source);
    if let Some(error) = file.errors.first() {
        let at = Span::new(error.offset, error.offset);
        return Err(Error::new(cstr!("{:?}", error.code), at));
    }
    let (runes, script) = match file.blocks.iter().find(|block| block.name == "script") {
        Some(block) => {
            let span = block.content;
            let text = source
                .get(span.start as usize..span.end as usize)
                .unwrap_or_default();
            runes::script(text, span.start)?
        }
        None => (runes::Runes::default(), String::default()),
    };
    let (tree, errors) = vize_l1::markup::parse(&allocator, source);
    if let Some(error) = errors.first() {
        let at = Span::new(error.offset, error.offset);
        return Err(Error::new(cstr!("markup: {:?}", error.code), at));
    }
    let region = dialect::Lowering::new(&allocator, source).root(&tree)?;
    let lowered = poc_support::lower(&allocator, &region)?;
    let mut module = target::Module::new(&runes);
    let mount = module.mount(View::new(&lowered.program))?;

    let mut out = Writer::<NoLinks>::with_capacity(source.len() * 2);
    out.push("import * as $ from \"svelte/internal/client\";\n\n");
    for (index, html) in module.templates.iter().enumerate() {
        let name = match index {
            0 => String::from("root"),
            n => cstr!("root_{n}"),
        };
        let literal = html.replace('`', "\\`").replace("${", "\\${");
        out.push(&cstr!("var {name} = $.from_html(`{literal}`);\n"));
    }
    out.push("\nexport default function Component($$anchor, $$props) {\n");
    let script = dedent(&script);
    if !script.is_empty() {
        out.push(&cstr!("  {}\n\n", indent(&script, "  ")));
    }
    for line in &mount {
        out.push(&cstr!("  {}\n", indent(line, "  ")));
    }
    out.push("}\n");
    if let Some(delegate) = module.delegate() {
        let mut tail = String::from("\n");
        append!(tail, "{delegate}\n");
        out.push(&tail);
    }
    Ok(out.finish().text)
}

#[cfg(test)]
mod tests {
    use super::compile;

    const COUNTER: &str = include_str!("../input/Counter.svelte");
    const SNAPSHOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/__snapshots__/Counter.js");

    #[test]
    fn counter_compiles_to_a_svelte_client_module() {
        let output = compile(COUNTER);
        assert!(output.is_ok(), "{output:?}");
        let output = output.unwrap_or_default();
        assert_eq!(poc_support::check_module(&output), Ok(()));
        assert_eq!(poc_support::snapshot(SNAPSHOT, &output), Ok(()));
    }

    #[test]
    fn uncovered_syntax_is_refused_with_its_span() {
        let refused = compile("<p>{#await promise}</p>");
        assert_eq!(refused.map_err(|error| error.span.start), Err(3));
    }
}
