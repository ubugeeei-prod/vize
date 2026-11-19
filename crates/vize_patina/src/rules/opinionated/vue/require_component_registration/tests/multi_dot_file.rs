use super::create_linter;

/// Vue's self name drops only the last extension: `router-link.demo.vue` is
/// `RouterLink.demo`, so `<router-link>` there needs a registration.
#[test]
fn a_multi_dot_file_name_is_not_a_shorter_self_name() {
    let linter = create_linter();
    let sfc = "<template>\n  <router-link to=\"/\" custom />\n</template>\n";
    let reported = |filename: &str| {
        linter
            .lint_sfc(sfc, filename)
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.start, diagnostic.end))
            .collect::<Vec<_>>()
    };
    let start = sfc.find("router-link").unwrap_or_default() as u32;

    assert_eq!(
        reported("demos/router-link.demo.vue"),
        [(start, start + "router-link".len() as u32)]
    );
    assert_eq!(reported("demos/router-link.vue"), Vec::<(u32, u32)>::new());
}
