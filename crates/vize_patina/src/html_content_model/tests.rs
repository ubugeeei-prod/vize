//! Per-class checker fixtures: every class proven in its minimal context,
//! refuted in the nearest conforming one, and unknown where the template
//! cannot decide it.

#[cfg(test)]
mod checker_tests {
    use crate::html_content_model::{Context, NodeKind, authored_skeleton, check, facts};
    use vize_s0::Allocator;

    /// `node class <- evidence` for every proven violation, in node order.
    fn findings(source: &str, context: Context) -> Vec<String> {
        let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let skeleton = authored_skeleton(&allocator, source);
        let label = |index: u32| -> String {
            let node = skeleton.node(index);
            let span = match &node.kind {
                NodeKind::Element(element) => element.name_span,
                _ => node.span,
            };
            format!(
                "{}@{}",
                &source[span.start as usize..span.end as usize],
                span.start
            )
        };
        check(&skeleton, 0, &context)
            .findings()
            .map(|(node, class, evidence)| {
                let evidence = evidence.map_or_else(|| "-".to_string(), |(_, e)| label(e));
                format!("{} {} <- {}", label(node), class.id(), evidence)
            })
            .collect()
    }

    fn body(source: &str) -> Vec<String> {
        findings(source, Context::Body)
    }

    fn truncated(source: &str) -> Vec<String> {
        findings(source, Context::Truncated)
    }

    #[test]
    fn fact_table_loads_with_every_row() {
        assert_eq!(facts().universe().count(), 146);
    }

    #[test]
    fn paragraph_auto_closed() {
        assert_eq!(
            body("<p><div></div></p>"),
            ["div@4 paragraph-auto-closed <- p@1"]
        );
        assert_eq!(
            body("<p><span><table></table></span></p>"),
            ["table@10 paragraph-auto-closed <- p@1"]
        );
        // The button is a scope boundary: the `<p>` stays open, and the
        // `<div>` violates the button's phrasing content model instead.
        assert_eq!(
            body("<p><button><div></div></button></p>"),
            ["div@12 phrasing-content-expected <- button@4"]
        );
        // A root `<p>` is HTML in every faithful context, so its children decide.
        assert_eq!(
            truncated("<p><ul></ul></p>"),
            ["ul@4 paragraph-auto-closed <- p@1"]
        );
        // The mount point decides whether `<div>` closes an outer `<p>`; in
        // every context where it does not, `<span>` forbids it: proven.
        assert_eq!(
            truncated("<span><div></div></span>"),
            ["div@7 phrasing-content-expected <- span@1"]
        );
    }

    #[test]
    fn heading_list_form_button_select() {
        assert_eq!(
            body("<h1><h2></h2></h1>"),
            ["h2@5 heading-auto-closed <- h1@1"]
        );
        assert_eq!(
            body("<li><span><li></li></span></li>"),
            ["li@11 list-item-auto-closed <- li@1"]
        );
        assert_eq!(body("<li><ul><li></li></ul></li>"), Vec::<String>::new());
        assert_eq!(
            body("<dl><dd><div><dt></dt></div></dd></dl>"),
            ["dt@14 list-item-auto-closed <- dd@5"]
        );
        assert_eq!(
            body("<form><div><form></form></div></form>"),
            ["form@12 nested-form-dropped <- form@1"]
        );
        assert_eq!(
            body("<button><span><button></button></span></button>"),
            ["button@15 button-auto-closed <- button@1"]
        );
        assert_eq!(
            body("<select><select></select></select>"),
            ["select@9 select-auto-closed <- select@1"]
        );
        assert_eq!(
            body("<select><input></select>"),
            ["input@9 select-auto-closed <- select@1"]
        );
    }

    #[test]
    fn formatting_adoption_and_markers() {
        assert_eq!(
            body("<a><div><a></a></div></a>"),
            ["a@9 formatting-adopted <- a@1"]
        );
        assert_eq!(
            body("<a><object><a></a></object></a>"),
            ["a@12 interactive-content-nested <- a@1"]
        );
        assert_eq!(
            body("<nobr><nobr></nobr></nobr>"),
            ["nobr@7 formatting-adopted <- nobr@1"]
        );
    }

    #[test]
    fn implied_end_tags() {
        assert_eq!(
            body("<select><option><option></option></option></select>"),
            ["option@17 implied-end-tag-closed <- option@9"]
        );
        assert_eq!(
            body("<ruby><rt><rp></rp></rt></ruby>"),
            ["rp@11 implied-end-tag-closed <- rt@7"]
        );
        // `rp`/`rt` leave an open `rtc` alone (the parser rule excepts it);
        // `rtc` itself is obsolete and outside `ruby`'s content model.
        assert_eq!(
            body("<ruby><rtc><rt></rt></rtc></ruby>"),
            ["rtc@7 phrasing-content-expected <- ruby@1"]
        );
    }

    #[test]
    fn table_anatomy() {
        assert_eq!(
            body("<table><tr></tr></table>"),
            ["tr@8 table-wrapper-inserted <- table@1"]
        );
        assert_eq!(
            body("<table><tbody><td></td></tbody></table>"),
            ["td@15 table-wrapper-inserted <- tbody@8"]
        );
        assert_eq!(
            body("<table><col></table>"),
            ["col@8 table-wrapper-inserted <- table@1"]
        );
        assert_eq!(
            body("<table><div></div></table>"),
            ["div@8 foster-parented <- table@1"]
        );
        assert_eq!(
            body("<table><tbody><tr>x</tr></tbody></table>"),
            ["x@18 foster-parented <- tr@15"]
        );
        assert_eq!(
            body("<table><tbody><tr> </tr></tbody></table>"),
            Vec::<String>::new()
        );
        assert_eq!(
            body("<table><tbody><thead></thead></tbody></table>"),
            ["thead@15 table-auto-closed <- tbody@8"]
        );
        assert_eq!(
            body("<div><tr></tr></div>"),
            ["tr@6 table-part-misplaced <- div@1"]
        );
        assert_eq!(
            body("<table><form><tbody></tbody></form></table>"),
            [
                "form@8 child-not-permitted <- table@1",
                "tbody@14 form-in-table-emptied <- form@8"
            ]
        );
        assert_eq!(
            body("<table><form></form></table>"),
            ["form@8 child-not-permitted <- table@1"]
        );
        assert_eq!(
            body("<table><input type=\"hidden\"></table>"),
            ["input@8 child-not-permitted <- table@1"]
        );
        assert_eq!(
            body("<table><input></table>"),
            ["input@8 foster-parented <- table@1"]
        );
        assert_eq!(
            body("<table><input :type=\"t\"></table>"),
            ["input@8 child-not-permitted <- table@1"]
        );
        assert_eq!(truncated("<tr><td></td></tr>"), Vec::<String>::new());
        assert_eq!(
            truncated("<tr><div></div></tr>"),
            ["div@5 foster-parented <- tr@1"]
        );
    }

    #[test]
    fn foreign_content() {
        assert_eq!(
            body("<svg><div></div></svg>"),
            ["div@6 foreign-content-breakout <- svg@1"]
        );
        assert_eq!(
            body("<svg><foreignObject><div></div></foreignObject></svg>"),
            Vec::<String>::new()
        );
        assert_eq!(body("<svg><a><a></a></a></svg>"), Vec::<String>::new());
        assert_eq!(body("<svg><font></font></svg>"), Vec::<String>::new());
        assert_eq!(
            body("<svg><font color=\"red\"></font></svg>"),
            ["font@6 foreign-content-breakout <- svg@1"]
        );
        assert_eq!(
            body("<math><mi><div></div></mi></math>"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn document_image_raw_text() {
        assert_eq!(
            body("<div><body></body></div>"),
            ["body@6 document-element-dropped <- -"]
        );
        assert_eq!(body("<div><image></div>"), ["image@6 image-renamed <- -"]);
        assert_eq!(
            body("<iframe><p></p></iframe>"),
            ["p@9 raw-text-content <- iframe@1"]
        );
        // A root `<a>` compiles to HTML: the mount assumption decides it.
        assert_eq!(
            truncated("<a><a></a></a>"),
            ["a@4 formatting-adopted <- a@1"]
        );
    }

    #[test]
    fn content_models() {
        assert_eq!(
            body("<span><div></div></span>"),
            ["div@7 phrasing-content-expected <- span@1"]
        );
        assert_eq!(
            body("<span><a><div></div></a></span>"),
            ["div@10 phrasing-content-expected <- span@1"]
        );
        assert_eq!(
            body("<ul><div></div></ul>"),
            ["div@5 child-not-permitted <- ul@1"]
        );
        assert_eq!(
            body("<ul>text</ul>"),
            ["text@4 child-not-permitted <- ul@1"]
        );
        assert_eq!(
            body("<ul><template v-for=\"i in l\"><li></li></template></ul>"),
            Vec::<String>::new()
        );
        assert_eq!(
            body("<a href=\"#\"><button></button></a>"),
            ["button@13 interactive-content-nested <- a@1"]
        );
        assert_eq!(
            body("<button><input type=\"hidden\"></button>"),
            Vec::<String>::new()
        );
        assert_eq!(
            body("<button><span tabindex=\"0\"></span></button>"),
            ["span@9 interactive-content-nested <- button@1"]
        );
        assert_eq!(
            body("<label><select></select></label>"),
            Vec::<String>::new()
        );
        assert_eq!(body("<video><source><track></video>"), Vec::<String>::new());
        assert_eq!(body("<p><map><area></map></p>"), Vec::<String>::new());
        assert_eq!(body("<ul><MyItem /></ul>"), Vec::<String>::new());
    }

    #[test]
    fn vue_constructs_erase_or_bound() {
        assert_eq!(
            body("<p><template v-if=\"x\"><div></div></template></p>"),
            ["div@23 paragraph-auto-closed <- p@1"]
        );
        assert_eq!(
            body("<p><Transition><div></div></Transition></p>"),
            ["div@16 paragraph-auto-closed <- p@1"]
        );
        assert_eq!(
            body("<p><MyCard><div></div></MyCard></p>"),
            Vec::<String>::new()
        );
        // A hyphenated tag stays an element until composition resolves it, so
        // an authored `<div>` inside it still closes the paragraph.
        assert_eq!(
            body("<p><my-card><div></div></my-card></p>"),
            ["div@13 paragraph-auto-closed <- p@1"]
        );
        assert_eq!(body("<p><my-card /></p>"), Vec::<String>::new());
        assert_eq!(
            body("<p><Teleport to=\"body\"><div></div></Teleport></p>"),
            Vec::<String>::new()
        );
        assert_eq!(
            body("<p><slot><div></div></slot></p>"),
            ["div@10 paragraph-auto-closed <- p@1"]
        );
        assert_eq!(
            body("<p><component :is=\"c\"><div></div></component></p>"),
            Vec::<String>::new()
        );
        assert_eq!(
            body("<p><div v-html=\"h\"></div></p>"),
            ["div@4 paragraph-auto-closed <- p@1"]
        );
        assert_eq!(
            body("<MyCard><p><div></div></p></MyCard>"),
            ["div@12 paragraph-auto-closed <- p@9"]
        );
    }
}
