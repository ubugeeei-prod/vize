//! `$This` inherits `mixins:` / `extends:` members (#3609).
//!
//! Before this, the typed-instance bridge listed only the component's own
//! option names, so a `methods`/`computed` body calling a mixin-provided
//! member reported `TS2339 Property 'x' does not exist on type '$This'`
//! on code `vue-tsc` accepts.

use super::super::super::generate_virtual_ts_with_offsets_legacy_vue2;
use super::super::analyze_options_api_script;
use vize_carton::{String, cstr};

/// The generated bridge from `// Options API typed instance bridge` through
/// the `$This` declaration, which is the whole surface these tests pin.
fn bridge_header(code: &str) -> String {
    let mut lines = Vec::new();
    let mut started = false;
    for line in code.lines() {
        if line == "  // Options API typed instance bridge" {
            started = true;
        }
        if !started {
            continue;
        }
        lines.push(line);
        if line.starts_with("  }") && line.ends_with(';') {
            break;
        }
    }
    String::from(lines.join("\n"))
}

fn legacy_bridge_header(script: &str) -> String {
    let summary = analyze_options_api_script(script);
    let output = generate_virtual_ts_with_offsets_legacy_vue2(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );
    bridge_header(&output.code)
}

const HELPERS: &str = concat!(
    "  type $InheritedInstance<T> = 0 extends (1 & T) ? any : T extends abstract new (...args: any[]) => infer __I ? __I : T extends { methods: infer __M } ? 0 extends (1 & __M) ? $NoInheritedMembers : [__M] extends [never] ? $NoInheritedMembers : __M extends object ? [keyof __M] extends [never] ? $NoInheritedMembers : string extends keyof __M ? $NoInheritedMembers : number extends keyof __M ? $NoInheritedMembers : symbol extends keyof __M ? $NoInheritedMembers : __M : $NoInheritedMembers : $NoInheritedMembers;\n",
    "  interface $NoInheritedMembers {}\n",
    "  type $InheritedMembers<T, __I = $InheritedInstance<T>> = [$IsAny<__I>] extends [true] ? $NoInheritedMembers : __I;",
);

#[test]
fn imported_mixin_and_extends_references_join_the_this_shape() {
    let script = r#"import { defineComponent } from 'vue'
import greeter from './greeter'
import counter from './counter'

export default defineComponent({
    mixins: [greeter],
    extends: counter,
    methods: {
        shout() {
            return this.greet()
        },
    },
})
"#;

    assert_eq!(
        legacy_bridge_header(script),
        cstr!(
            "  // Options API typed instance bridge\n\
             {HELPERS}\n\
             \x20 type $Inherited0 = $InheritedMembers<typeof greeter>;\n\
             \x20 type $Inherited1 = $InheritedMembers<typeof counter>;\n\
             \x20 type $This = {{\n\
             \x20   shout: any;\n\
             \x20 }} & $Inherited0 & $Inherited1;"
        )
    );
}

/// A mixin declared in the same script block is named the same way — the
/// bridge is emitted below the authored copy, so the local binding is in
/// scope for the `typeof` query.
#[test]
fn locally_declared_mixin_is_named_by_its_binding() {
    let script = r#"import { defineComponent } from 'vue'

const inlineGreeter = defineComponent({
    methods: {
        ping() {
            return 1
        },
    },
})

export default defineComponent({
    mixins: [inlineGreeter],
    methods: {
        double() {
            return this.ping() * 2
        },
    },
})
"#;

    assert_eq!(
        legacy_bridge_header(script),
        cstr!(
            "  // Options API typed instance bridge\n\
             {HELPERS}\n\
             \x20 type $Inherited0 = $InheritedMembers<typeof inlineGreeter>;\n\
             \x20 type $This = {{\n\
             \x20   double: any;\n\
             \x20 }} & $Inherited0;"
        )
    );
}

/// A namespaced reference is still a legal `typeof` operand.
#[test]
fn namespaced_mixin_reference_keeps_its_member_chain() {
    let script = r#"import { defineComponent } from 'vue'
import * as shared from './shared'

export default defineComponent({
    mixins: [shared.greeter],
    methods: {
        shout() {
            return this.greet()
        },
    },
})
"#;

    assert_eq!(
        legacy_bridge_header(script),
        cstr!(
            "  // Options API typed instance bridge\n\
             {HELPERS}\n\
             \x20 type $Inherited0 = $InheritedMembers<typeof shared.greeter>;\n\
             \x20 type $This = {{\n\
             \x20   shout: any;\n\
             \x20 }} & $Inherited0;"
        )
    );
}

#[test]
fn inline_object_mixin_members_keep_the_existing_direct_shape() {
    let script = r#"import { defineComponent } from 'vue'

export default defineComponent({
    mixins: [{ methods: { greet() {} } }],
    methods: { shout() { return this.greet() } },
})
"#;

    assert_eq!(
        legacy_bridge_header(script),
        "  // Options API typed instance bridge\n  \
         type $This = {\n    \
         greet: any;\n    \
         shout: any;\n  };"
    );
}

#[test]
fn any_typed_mixin_uses_the_guarded_empty_contribution() {
    let script = r#"import { defineComponent } from 'vue'
import untyped from './untyped'

export default defineComponent({
    mixins: [untyped],
    methods: { own() {} },
})
"#;

    assert_eq!(
        legacy_bridge_header(script),
        cstr!(
            "  // Options API typed instance bridge\n\
             {HELPERS}\n\
             \x20 type $Inherited0 = $InheritedMembers<typeof untyped>;\n\
             \x20 type $This = {{\n\
             \x20   own: any;\n\
             \x20 }} & $Inherited0;"
        )
    );
}
