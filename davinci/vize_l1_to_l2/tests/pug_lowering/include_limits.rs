//! Total-work guards also bound acyclic graphs and cached include expansions.
use vize_l0::String;
use vize_l1_to_l2::lower::pug::{PugIncludes, PugRendering, derive_template_source_including};

struct Branching {
    reads: usize,
}

impl PugIncludes for Branching {
    fn read(&mut self, _from: &str, path: &str) -> Result<(String, String), String> {
        self.reads += 1;
        assert!(self.reads <= 4096, "acyclic graph exceeded the load budget");
        let depth = path
            .trim_end_matches(".pug")
            .parse::<usize>()
            .expect("depth");
        let content = if depth < 20 {
            vize_l0::cstr!("include {next}.pug\ninclude {next}.pug\n", next = depth + 1)
        } else {
            String::from("p leaf\n")
        };
        Ok((vize_l0::cstr!("/root/{path}"), content))
    }
}

#[test]
fn acyclic_branching_graph_stops_before_a_4097th_read() {
    let mut reader = Branching { reads: 0 };
    let result = derive_template_source_including(
        "include 0.pug\n",
        PugRendering::Pug,
        "/root/main.pug",
        &mut reader,
    );
    assert_eq!(reader.reads, 4096);
    assert!(result.has_errors());
    assert!(result.html.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].message.as_str(),
        "pug includes exceed the compilation budget (4096 loads/expansions or 16 MiB source/output)"
    );
}

struct Raw {
    reads: usize,
    bytes: usize,
}

impl PugIncludes for Raw {
    fn read(&mut self, _from: &str, path: &str) -> Result<(String, String), String> {
        self.reads += 1;
        Ok((
            vize_l0::cstr!("/root/{path}"),
            String::from("x".repeat(self.bytes).as_str()),
        ))
    }
}

#[test]
fn cached_includes_have_an_independent_expansion_budget() {
    let source = format!(
        "mixin leaf\n  include leaf.txt\n{}",
        "+leaf()\n".repeat(4097)
    );
    let mut reader = Raw { reads: 0, bytes: 1 };
    let result =
        derive_template_source_including(&source, PugRendering::Pug, "/root/main.pug", &mut reader);
    assert_eq!(reader.reads, 1);
    assert!(result.has_errors());
    assert_eq!(result.html.len(), 4096);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].message.as_str(),
        "pug includes exceed the compilation budget (4096 loads/expansions or 16 MiB source/output)"
    );
}

#[test]
fn cached_raw_files_cannot_multiply_output_past_sixteen_mib() {
    let source = format!("mixin leaf\n  include leaf.txt\n{}", "+leaf()\n".repeat(9));
    let mut reader = Raw {
        reads: 0,
        bytes: 2 * 1024 * 1024,
    };
    let result =
        derive_template_source_including(&source, PugRendering::Pug, "/root/main.pug", &mut reader);
    assert_eq!(reader.reads, 1);
    assert!(result.has_errors());
    assert_eq!(result.html.len(), 16 * 1024 * 1024);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].message.as_str(),
        "pug includes exceed the compilation budget (4096 loads/expansions or 16 MiB source/output)"
    );
}

#[test]
fn absolute_paths_never_reach_the_host_reader() {
    for path in [
        "/root.pug",
        r"\root.pug",
        r"C:\root.pug",
        "C:root.pug",
        r"\\server\share\root.pug",
        r"\\?\C:\root.pug",
    ] {
        let source = format!("include {path}\n");
        let mut reader = Raw { reads: 0, bytes: 1 };
        let result = derive_template_source_including(
            &source,
            PugRendering::Pug,
            "/root/main.pug",
            &mut reader,
        );
        assert_eq!(reader.reads, 0, "{path}");
        assert!(result.has_errors(), "{path}");
        assert_eq!(result.diagnostics.len(), 1, "{path}");
        assert_eq!(
            result.diagnostics[0].message.as_str(),
            "the \"basedir\" option is required to use includes and extends with \"absolute\" paths",
            "{path}"
        );
    }
}

#[test]
fn relative_parent_paths_still_reach_the_host_reader() {
    let mut reader = Raw { reads: 0, bytes: 1 };
    let result = derive_template_source_including(
        "include ../sibling.txt\n",
        PugRendering::Pug,
        "/root/main.pug",
        &mut reader,
    );
    assert_eq!(reader.reads, 1);
    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(result.html.as_str(), "x");
}
