//! Authored whole strings and independent Unicode/pending-suffix thresholds.

use super::*;

#[test]
fn empty_plain_comments_and_original_outer_bytes_remain_exact_at_every_width() {
    for source in [
        "<template></template>",
        "\u{feff}<!--前-->\r\n<template lang='html'>plain &amp;\n<!--inside--></template><!--尾-->\r\n",
        "<template><div>plain</div><br><input></template>",
    ] {
        for width in [0, 1, 7, 80, 200] {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                fixed(source, source, options(width, 2, ending));
            }
        }
    }
}

#[test]
fn original_headers_keep_boolean_quoted_unquoted_values_order_entities_and_final_bytes() {
    let source = "<!--前--><template><div  id = '雪' disabled title=raw data-x=\"&amp;{{a}}\">{{a+b}}</div></template><!--尾-->\r\n";
    let expected = "<!--前--><template><div id='雪' disabled title=raw data-x=\"&amp;{{a}}\">{{a + b}}</div></template><!--尾-->\r\n";
    fixed(source, expected, options(200, 2, LineEnding::Lf));
    for (ending, newline) in [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")] {
        for indent in [0, 1, 2, 4] {
            let source = "<template><section><input  disabled id = 'x' /></section></template>";
            let attr = " ".repeat(2 * indent);
            let close = " ".repeat(indent);
            let expected = format!(
                "<template><section><input{newline}{attr}disabled{newline}{attr}id='x'{newline}{close}/></section></template>"
            );
            fixed(source, &expected, options(0, indent, ending));
        }
    }
}

#[test]
fn full_prefix_column_suffix_lookahead_and_unicode_scalar_width_choose_binary_layout() {
    for (source, width, expected) in [
        (
            "<template><div>{{a+b}}</div></template>",
            41,
            "<template><div>{{a + b}}</div></template>",
        ),
        (
            "<template><div>{{a+b}}</div></template>",
            40,
            "<template><div>{{a +\n    b}}</div></template>",
        ),
        (
            "<template><div>{{a+b}}</div></template><!--尾-->",
            49,
            "<template><div>{{a + b}}</div></template><!--尾-->",
        ),
        (
            "<template><div>{{a+b}}</div></template><!--尾-->",
            48,
            "<template><div>{{a +\n    b}}</div></template><!--尾-->",
        ),
        (
            "<!--🙂--><template><div>{{a+b}}</div></template>",
            49,
            "<!--🙂--><template><div>{{a + b}}</div></template>",
        ),
        (
            "<!--🙂--><template><div>{{a+b}}</div></template>",
            48,
            "<!--🙂--><template><div>{{a +\n    b}}</div></template>",
        ),
        (
            "<template>\n<div>{{a+b}}</div>\n</template>",
            20,
            "<template>\n<div>{{a + b}}</div>\n</template>",
        ),
        (
            "<template>\n<div>{{a+b}}</div>\n</template>",
            19,
            "<template>\n<div>{{a +\n    b}}</div>\n</template>",
        ),
    ] {
        fixed(source, expected, options(width, 2, LineEnding::Lf));
    }
}

#[test]
fn same_original_multiple_filters_trim_gaps_and_nested_division_keep_fixed_points() {
    for (ending, newline) in [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")] {
        let source = "\u{feff}<!--前-->\r\n<template><div>甲{{ (a/2)/(b/3) | add(2+3,) | upper () }}乙<span>{{ &#26085;&#26412; | upper( ) }}</span></div></template>\r\n";
        let expected = format!(
            "\u{feff}<!--前-->\r\n<template><div>甲{{{{ (a /{newline}    2) /{newline}    (b /{newline}      3) | add(2 +{newline}    3,) | upper () }}}}乙<span>{{{{ &#26085;&#26412; | upper( ) }}}}</span></div></template>\r\n"
        );
        fixed(source, &expected, options(0, 2, ending));
    }
    fixed(
        "<template><p>{{ a+b\r\n | upper }}{{a|upper()}}{{a|upper( )}}{{a|upper ()}}</p></template>",
        "<template><p>{{ a + b\r\n | upper }}{{a|upper()}}{{a|upper( )}}{{a|upper ()}}</p></template>",
        options(200, 2, LineEnding::Lf),
    );
}
