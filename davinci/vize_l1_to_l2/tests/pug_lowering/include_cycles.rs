//! The read counter turns the original exponential cycle into a bounded RED.
use vize_l0::String;
use vize_l1_to_l2::lower::pug::{PugIncludes, PugRendering, derive_template_source_including};

struct Reader {
    reads: usize,
    files: Vec<(&'static str, &'static str)>,
}

impl PugIncludes for Reader {
    fn read(&mut self, _from: &str, path: &str) -> Result<(String, String), String> {
        self.reads += 1;
        assert!(
            self.reads <= 8,
            "cycle was not refused before recursive expansion"
        );
        let content = self
            .files
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, content)| *content)
            .ok_or_else(|| String::from("missing fixture"))?;
        Ok((vize_l0::cstr!("/root/{path}"), String::from(content)))
    }
}

#[test]
fn direct_branching_cycle_stops_after_two_reads() {
    let source = "include a.pug\ninclude a.pug\n";
    let mut reader = Reader {
        reads: 0,
        files: vec![("a.pug", source)],
    };
    let result =
        derive_template_source_including(source, PugRendering::Pug, "/root/a.pug", &mut reader);
    assert!(result.has_errors());
    assert_eq!(reader.reads, 2);
    assert!(result.html.is_empty());
    assert_eq!(result.diagnostics.len(), 2);
}

#[test]
fn indirect_branching_cycle_stops_after_four_reads() {
    let source = "include b.pug\ninclude b.pug\n";
    let mut reader = Reader {
        reads: 0,
        files: vec![("a.pug", source), ("b.pug", "include a.pug\n")],
    };
    let result =
        derive_template_source_including(source, PugRendering::Pug, "/root/a.pug", &mut reader);
    assert!(result.has_errors());
    assert_eq!(reader.reads, 4);
    assert!(result.html.is_empty());
    assert_eq!(result.diagnostics.len(), 2);
}

#[test]
fn repeated_siblings_are_not_cycles() {
    let source = "include a.pug\ninclude a.pug\n";
    let mut reader = Reader {
        reads: 0,
        files: vec![("a.pug", "include b.pug\n"), ("b.pug", "p child\n")],
    };
    let result =
        derive_template_source_including(source, PugRendering::Pug, "/root/main.pug", &mut reader);
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(reader.reads, 4);
    assert_eq!(result.html.as_str(), "<p>child</p><p>child</p>");
    assert_eq!(
        result.dependencies,
        [String::from("/root/a.pug"), String::from("/root/b.pug")]
    );
}
