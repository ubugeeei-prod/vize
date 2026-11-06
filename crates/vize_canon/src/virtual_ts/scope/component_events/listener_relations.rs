//! Named listener relations share one declaration identity in hoisted projects.
use vize_carton::{String, cstr};

const RELATIONS: &str = include_str!("../../helpers/declarations/listener-args.d.ts.txt");

pub(super) struct ListenerRelations {
    pub(super) args: String,
    missing: String,
    embedded: bool,
}

impl ListenerRelations {
    pub(super) fn new(prefix: &str, hoisted: bool) -> Self {
        Self {
            args: if hoisted {
                String::from(crate::virtual_ts::helpers::shared_types::LISTENER_ARGS.select(true))
            } else {
                cstr!("{prefix}_listener_args")
            },
            missing: if hoisted {
                String::default()
            } else {
                cstr!("{prefix}_listener_missing")
            },
            embedded: !hoisted,
        }
    }

    pub(super) fn emit(&self, ts: &mut String, indent: &str) {
        if self.embedded {
            let declarations = RELATIONS
                .replace("$ListenerArgs", self.args.as_str())
                .replace("$ListenerMissing", self.missing.as_str());
            for line in declarations.lines() {
                ts.push_str(indent);
                ts.push_str(line);
                ts.push('\n');
            }
        }
    }
}
