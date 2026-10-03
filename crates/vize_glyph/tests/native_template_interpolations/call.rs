//! Whole selected Call documents keep actual syntax, source and framing custody.

use super::{format, preservation::assert_preserved};
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::Allocator;

#[path = "call/custody.rs"]
mod custody;
#[path = "call/refusals.rs"]
mod refusals;

const WIDTHS: [usize; 5] = [0, 1, 7, 80, 200];
const SCRIPTS: [&str; 2] = ["", "<script setup lang=ts>let a=1</script>"];

#[test]
fn ordinary_calls_pin_complete_selected_js_ts_outputs_without_new_call_breaks() {
    for (content, body) in [
        ("f()", "f ( )"),
        ("f(a,b)", "f ( a, b )"),
        ("f ( a , b )", "f ( a, b )"),
        ("f(a,)", "f ( a, )"),
        ("f(a,b,)", "f ( a, b, )"),
        ("f(g(a),h(b))", "f ( g ( a ), h ( b ) )"),
        ("obj.key(a,-1)", "obj . key ( a, - 1 )"),
        ("obj[key](true,null)", "obj [ key ] ( true, null )"),
        ("f().key", "f ( ) . key"),
        ("obj[f(a)]", "obj [ f ( a ) ]"),
        ("日本('x\\x20y')", "日本 ( 'x\\x20y' )"),
        ("f&#40;a&#44;b&#41;", "f &#40; a&#44; b &#41;"),
        ("f&#40;a&#44;&#41;", "f &#40; a&#44; &#41;"),
        ("f&#32;&#40;&#9;a&#41;", "f&#32;&#40;&#9;a &#41;"),
        ("f(/*empty*/)", "f (/*empty*/)"),
        ("f(a /*before*/, /*after*/)", "f ( a /*before*/, /*after*/)"),
        ("f(a,/*tail*/)", "f ( a,/*tail*/)"),
        ("/*#__PURE__*/f(a)", "/*#__PURE__*/f ( a )"),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!(
                "<!--前--><template><!--keep--><p>{{{{{content}}}}}</p></template>{script}"
            );
            for width in WIDTHS {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let newline = if line_ending == LineEnding::Lf {
                        "\n"
                    } else {
                        "\r\n"
                    };
                    let options = PrintOptions {
                        width,
                        line_ending,
                        ..PrintOptions::default()
                    };
                    let expected = if width < 80 {
                        vize_l0::cstr!("<!--keep--><p>{{{{{newline}    {body}{newline}  }}}}</p>")
                    } else {
                        vize_l0::cstr!("<!--keep--><p>{{{{ {body} }}}}</p>")
                    };
                    let output = format(&source, options);
                    assert_eq!(output, expected, "{source} / {options:?}");
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(format(&replay, options), output, "{replay}");
                }
            }
        }
        assert_preserved(content);
    }
    for width in WIDTHS {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            let newline = if line_ending == LineEnding::Lf {
                "\n"
            } else {
                "\r\n"
            };
            let expected = if width < 80 {
                vize_l0::cstr!(
                    "<p>{{{{{newline}    f ( a +{newline}      b, c ){newline}  }}}}</p>"
                )
            } else {
                vize_l0::String::from("<p>{{ f ( a + b, c ) }}</p>")
            };
            assert_eq!(
                format(
                    "<template><p>{{f(a+b,c)}}</p></template>",
                    PrintOptions {
                        width,
                        line_ending,
                        ..PrintOptions::default()
                    }
                ),
                expected
            );
        }
    }
    assert_preserved("f(a+b,c)");
}

#[test]
fn call_arguments_keep_authored_comments_entities_and_physical_lf_at_every_width() {
    for (content, body) in [
        ("f(&#39;//x&#39;\n)", "f ( &#39;//x&#39;\n)"),
        ("f(&quot;//x&quot;\r\n)", "f ( &quot;//x&quot;\r\n)"),
        ("f(&#39;//x&#39;\n,a)", "f ( &#39;//x&#39;\n, a )"),
        ("f(a,&quot;//x&quot;\r\n,)", "f ( a, &quot;//x&quot;\r\n, )"),
        ("f(a //tail\n,b)", "f ( a //tail\n, b )"),
        ("f(a /*原\r\n*/,b)", "f ( a /*原\r\n*/, b )"),
        ("f(a,&#47;&#47;tail\n b)", "f ( a,&#47;&#47;tail\n b )"),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template><p>{{{{{content}}}}}</p></template>{script}");
            for width in WIDTHS {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let newline = if line_ending == LineEnding::Lf {
                        "\n"
                    } else {
                        "\r\n"
                    };
                    let options = PrintOptions {
                        width,
                        line_ending,
                        ..PrintOptions::default()
                    };
                    let output = format(&source, options);
                    assert_eq!(
                        output,
                        vize_l0::cstr!("<p>{{{{{newline}    {body}{newline}  }}}}</p>"),
                        "{source} / {options:?}"
                    );
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    assert_eq!(format(&replay, options), output, "{replay}");
                }
            }
        }
        assert_preserved(content);
    }
    for terminator in ["", "\r"] {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{f(&#39;//x&#39;{terminator})}}}}</template>");
        let descriptor = super::Vue.observe_descriptor(
            &arena,
            &source,
            super::DescriptorOptions {
                version: super::VueVersion::V3,
                dialect: super::VueDialect::Vue,
                template: super::SurfaceParseOptions::default(),
            },
        );
        assert!(descriptor.admitted().is_err(), "{source}");
        assert!(
            descriptor.issues().iter().any(|issue| issue.code
                == vize_l1::container::vue::DescriptorIssueCode::UnsupportedBoundary)
        );
        assert!(core::ptr::eq(descriptor.source(), source.as_str()));
    }
}
