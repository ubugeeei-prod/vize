//! Relative script classification over the dependency walk's one parsed list.

use vize_carton::String;

pub(super) fn relative_ts_specifiers(specifiers: &[String]) -> impl Iterator<Item = &String> {
    specifiers.iter().filter(|specifier| {
        (specifier.starts_with("./") || specifier.starts_with("../"))
            && !specifier.ends_with(".vue")
            && !specifier.ends_with(".vue.ts")
            && !specifier.ends_with(".vue.tsx")
    })
}

// Preserve the byte-exact original collector as a differential test oracle.
#[cfg(test)]
mod original;

#[cfg(test)]
mod tests {
    use oxc_span::SourceType;

    use super::{original::collect_relative_ts_specifiers, relative_ts_specifiers};

    #[test]
    fn collects_type_only_and_require_dependency_specifiers() {
        let source = r#"import type { User } from "./user";
export type { Model } from "../model";
export type Lazy = import("./lazy").Lazy;
import Common = require("./common");
const runtime = require("./runtime");
type VirtualTs = typeof import("./App.vue.ts");
type VirtualTsx = typeof import("./App.vue.tsx");
declare module "./augment" {}
type App = typeof import("./App.vue");
"#;

        assert_eq!(
            collect_relative_ts_specifiers(source, SourceType::ts()),
            vec![
                "./user",
                "../model",
                "./lazy",
                "./common",
                "./runtime",
                "./augment"
            ]
        );
        let collected =
            crate::batch::ImportRewriter::new().collect_all_specifiers(source, SourceType::ts());
        assert_eq!(
            relative_ts_specifiers(&collected)
                .cloned()
                .collect::<Vec<_>>(),
            collect_relative_ts_specifiers(source, SourceType::ts())
        );
    }

    #[test]
    fn one_module_list_preserves_script_specifiers_for_js_ts_and_jsx() {
        let sources = [
            (
                SourceType::ts(),
                r#"import Default from './same';
export { Default } from './same';
export * from '../export-all';
const lazy = import('./dynamic');
const common = require('./common');
type Typed = import('./typed', { with: { 'resolution-mode': 'require' } }).Typed;
import External = require('./external');
declare module './augment' {}
import './App.vue'; import './App.vue.ts'; import './App.vue.tsx';
import '#alias'; import 'package'; import '/absolute';
// import './comment';
const text = "import './string'";
const computed = import('./computed' + suffix);
"#,
            ),
            (
                SourceType::mjs(),
                r#"import './first'; export * from './second';
const lazy = import('./third'); const common = require('./fourth');
"#,
            ),
            (
                SourceType::tsx(),
                r#"import './first';
const element = <div title="require('./text')">{import('./second')}</div>;
"#,
            ),
            (
                SourceType::jsx(),
                r#"import './first';
const element = <div>{require('./second')}</div>;
"#,
            ),
        ];
        for (source_type, source) in sources {
            let modules =
                crate::batch::ImportRewriter::new().collect_all_specifiers(source, source_type);
            let scripts = relative_ts_specifiers(&modules)
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(scripts, collect_relative_ts_specifiers(source, source_type));
        }
    }
}
