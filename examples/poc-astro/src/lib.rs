//! Proof of concept: an Astro component compiled through the level crates.
//!
//! [`container`] splits the frontmatter from the markup with an L1 container
//! format, the L1 markup parser reads the markup in place, [`dialect`] lowers
//! Astro's expressions and its embedded-markup idioms to neutral L2 ops, the
//! shared L2 to L3 lowering separates static from dynamic, and [`target`]
//! writes a server-rendering module for Astro's runtime. The frontmatter runs
//! once per render; its imports move to the module top.
//! Not a supported product surface (2026-11-03 scope decision).

mod container;
mod dialect;
mod target;

use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use poc_support::{Error, View, dedent, indent};
use vize_l0::{Allocator, Span, String, cstr};
use vize_l1::container::ContainerFormat;
use vize_l4::write::{NoLinks, Writer};

const RUNTIME: &str = "import { createAstro as $$createAstro, createComponent as $$createComponent, render as $$render, addAttribute as $$addAttribute } from \"astro/runtime/server/index.js\";\n";

/// Compile one `.astro` component to a JavaScript module.
pub fn compile(source: &str) -> Result<String, Error> {
    let allocator = Allocator::default();
    let file = container::Astro.split(&allocator, source);
    if let Some(error) = file.errors.first() {
        let at = Span::new(error.offset, error.offset);
        return Err(Error::new(cstr!("{:?}", error.code), at));
    }
    let (imports, script, markup_start) = match file.blocks.first() {
        Some(block) => {
            let content = block.content;
            let text = source
                .get(content.start as usize..content.end as usize)
                .unwrap_or_default();
            let (imports, script) = frontmatter(text, content.start)?;
            (imports, script, block.authored_span().end)
        }
        None => (String::default(), String::default(), 0),
    };
    let markup = source.get(markup_start as usize..).unwrap_or_default();
    let (tree, errors) = vize_l1::markup::parse(&allocator, markup);
    if let Some(error) = errors.first() {
        let at = Span::new(markup_start + error.offset, markup_start + error.offset);
        return Err(Error::new(cstr!("markup: {:?}", error.code), at));
    }
    let region = dialect::Lowering::new(&allocator, source).region(tree.children.iter())?;
    let lowered = poc_support::lower(&allocator, &region)?;
    let render = target::render(View::new(&lowered.program))?;

    let mut out = Writer::<NoLinks>::with_capacity(source.len() * 2);
    out.push(RUNTIME);
    out.push(&imports);
    out.push("\nconst $$Astro = $$createAstro();\n");
    out.push("const $$Component = $$createComponent(async ($$result, $$props, $$slots) => {\n");
    out.push("  const Astro = $$result.createAstro($$Astro, $$props, $$slots);\n");
    out.push("  Astro.self = $$Component;\n");
    if !script.is_empty() {
        out.push(&cstr!("  {}\n", indent(&script, "  ")));
    }
    out.push(&cstr!("  return {render};\n"));
    out.push("});\n\nexport default $$Component;\n");
    Ok(out.finish().text)
}

/// The frontmatter's import declarations and the rest of its statements.
fn frontmatter(text: &str, base: u32) -> Result<(String, String), Error> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::mjs()).parse();
    if let Some(error) = parsed.diagnostics.iter().next() {
        return Err(Error::new(cstr!("{error}"), Span::new(base, base)));
    }
    let mut imports = String::default();
    let mut rest = String::default();
    let mut cursor = 0;
    for statement in &parsed.program.body {
        if let Statement::ImportDeclaration(import) = statement {
            let span = import.span();
            rest.push_str(text.get(cursor..span.start as usize).unwrap_or_default());
            imports.push_str(
                text.get(span.start as usize..span.end as usize)
                    .unwrap_or_default(),
            );
            imports.push('\n');
            cursor = span.end as usize;
        }
    }
    rest.push_str(text.get(cursor..).unwrap_or_default());
    Ok((imports, dedent(&rest)))
}

#[cfg(test)]
mod tests {
    use super::compile;

    const CARD: &str = include_str!("../input/Card.astro");
    const SNAPSHOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/__snapshots__/Card.js");

    #[test]
    fn card_compiles_to_an_astro_server_module() {
        let output = compile(CARD);
        assert!(output.is_ok(), "{output:?}");
        let output = output.unwrap_or_default();
        assert_eq!(poc_support::check_module(&output), Ok(()));
        assert_eq!(poc_support::snapshot(SNAPSHOT, &output), Ok(()));
    }

    #[test]
    fn other_embedded_markup_is_refused_with_its_span() {
        let refused = compile("---\n---\n<ul>{items.filter((item) => <li>{item}</li>)}</ul>");
        assert_eq!(refused.map_err(|error| error.span.start), Err(12));
    }

    #[test]
    fn astro_slot_elements_are_refused() {
        let refused = compile("<slot />");
        assert!(refused.is_err());
    }

    #[test]
    fn top_level_await_stays_inside_an_async_component() {
        let output =
            compile("---\nconst value = await Promise.resolve('ready');\n---\n<p>{value}</p>")
                .unwrap_or_default();
        assert!(output.contains("$$createComponent(async ("));
        assert_eq!(poc_support::check_module(&output), Ok(()));
    }

    #[test]
    fn static_attribute_entities_are_preserved() {
        let output =
            compile("<p title=\"a &amp; b\" data-note='say \"hi\"'>ok</p>").unwrap_or_default();
        assert!(output.contains("title=\"a &amp; b\""));
        assert!(output.contains("data-note=\"say &quot;hi&quot;\""));
    }
}
