//! Named listener relations share one declaration identity in hoisted projects.
use vize_carton::{String, cstr};

const RELATIONS: &str = include_str!("../../helpers/declarations/listener-args.d.ts.txt");

pub(super) struct ListenerRelations {
    pub(super) args: String,
    missing: String,
    hoisted: bool,
}

impl ListenerRelations {
    pub(super) fn new(prefix: &str, hoisted: bool) -> Self {
        Self {
            // Keep a local alias identity even when its implementation lives in
            // the shared SDK. Reusing the imported generic identity directly
            // makes TypeScript structurally compare every component contract
            // that reaches one aggregate, causing TS2859 at project scale.
            args: cstr!("{prefix}_listener_args"),
            missing: if hoisted {
                String::default()
            } else {
                cstr!("{prefix}_listener_missing")
            },
            hoisted,
        }
    }

    pub(super) fn emit(&self, ts: &mut crate::virtual_ts::TextBuf, indent: &str) {
        if self.hoisted {
            let shared = crate::virtual_ts::helpers::shared_types::LISTENER_ARGS.select(true);
            ts.push_str(indent);
            ts.push_str("type ");
            ts.push_str(self.args.as_str());
            ts.push_str("<__P, __K extends string | number | symbol, __Raw extends string | number | symbol = never> = ");
            ts.push_str(shared);
            ts.push_str("<__P, __K, __Raw>;\n");
            return;
        }
        let declarations = RELATIONS
            .replace("$ListenerArgs", self.args.as_str())
            .replace("$ListenerMissing", self.missing.as_str());
        for line in declarations.lines() {
            ts.push_str(indent);
            crate::virtual_ts::generated_names::push_lib_text(ts, line);
            ts.push('\n');
        }
    }
}
