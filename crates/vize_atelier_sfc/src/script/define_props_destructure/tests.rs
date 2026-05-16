//! Tests for props destructure handling.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::helpers::gen_props_access_exp;
    use super::super::transform::transform_destructured_props;
    use super::super::{PropsDestructureBinding, PropsDestructuredBindings};
    use vize_carton::ToCompactString;

    fn make_bindings(names: &[&str]) -> PropsDestructuredBindings {
        let mut bindings = PropsDestructuredBindings::default();
        for name in names {
            bindings.bindings.insert(
                name.to_compact_string(),
                PropsDestructureBinding {
                    local: name.to_compact_string(),
                    default: None,
                },
            );
        }
        bindings
    }

    #[test]
    fn test_gen_props_access_exp() {
        assert_eq!(gen_props_access_exp("msg"), "__props.msg");
        assert_eq!(gen_props_access_exp("my-prop"), "__props[\"my-prop\"]");
    }

    #[test]
    fn test_transform_simple() {
        let bindings = make_bindings(&["msg"]);
        let source = "console.log(msg)";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_with_shadowing() {
        let bindings = make_bindings(&["msg"]);

        // msg is shadowed by the arrow function parameter
        let source = "const fn = (msg) => console.log(msg)";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_in_computed() {
        let bindings = make_bindings(&["count"]);

        // count inside computed arrow function should be rewritten
        let source = "const double = computed(() => count * 2)";
        let result = transform_destructured_props(source, &bindings);
        assert_eq!(result, "const double = computed(() => __props.count * 2)");
    }

    #[test]
    fn test_transform_multiple_refs() {
        let bindings = make_bindings(&["foo", "bar"]);

        let source = "const result = foo + bar";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    // ==================== New test cases ====================

    #[test]
    fn test_transform_in_function_declaration() {
        let bindings = make_bindings(&["count"]);

        // count inside function declaration should be rewritten
        let source = r#"function double() {
    return count * 2
}"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_in_nested_arrow_function() {
        let bindings = make_bindings(&["msg"]);

        // msg inside nested arrow function should be rewritten
        let source = r#"const outer = () => {
    const inner = () => msg
    return inner()
}"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_in_method() {
        let bindings = make_bindings(&["count"]);

        // count inside regular function expression should be rewritten
        let source = r#"const obj = {
    getCount: function() { return count }
}"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_in_watch_callback() {
        let bindings = make_bindings(&["count"]);

        let source = r#"watch(() => count, (newVal) => {
    console.log(newVal)
})"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_shadowed_in_for_of() {
        let bindings = make_bindings(&["item"]);

        // item is shadowed in for...of
        let source = r#"for (const item of items) {
    console.log(item)
}"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_shadowed_in_for_in() {
        let bindings = make_bindings(&["key"]);

        // key is shadowed in for...in
        let source = r#"for (const key in obj) {
    console.log(key)
}"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_shadowed_in_catch() {
        let bindings = make_bindings(&["error"]);

        // error is shadowed in catch
        let source = r#"try {
    doSomething()
} catch (error) {
    console.log(error)
}"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_in_block_scope() {
        let bindings = make_bindings(&["count"]);

        // count inside a block but not shadowed should be rewritten
        let source = r#"{
    const doubled = count * 2
    console.log(doubled)
}"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_shadowed_in_block() {
        let bindings = make_bindings(&["count"]);

        // count is shadowed in block
        let source = r#"{
    const count = 10
    console.log(count)
}"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_template_literal() {
        let bindings = make_bindings(&["name"]);

        let source = "const greeting = `Hello, ${name}!`";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_ternary() {
        let bindings = make_bindings(&["show"]);

        let source = "const display = show ? 'visible' : 'hidden'";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_logical_expression() {
        let bindings = make_bindings(&["enabled", "active"]);

        let source = "const isOn = enabled && active";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_object_shorthand() {
        let bindings = make_bindings(&["foo"]);

        let source = "const obj = { foo }";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_array_access() {
        let bindings = make_bindings(&["items"]);

        let source = "const first = items[0]";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_member_expression() {
        let bindings = make_bindings(&["user"]);

        let source = "const name = user.name";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_chained_member() {
        let bindings = make_bindings(&["data"]);

        let source = "const value = data.nested.value";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_call_argument() {
        let bindings = make_bindings(&["count"]);

        let source = "doSomething(count, 'test')";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_multiple_statements() {
        let bindings = make_bindings(&["msg", "count"]);

        let source = r#"console.log(msg)
const double = count * 2
return { msg, count }"#;
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_no_transform_property_key() {
        let bindings = make_bindings(&["msg"]);

        // msg as property key should NOT be rewritten
        let source = "const obj = { msg: 'hello' }";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_no_transform_property_access() {
        let bindings = make_bindings(&["name"]);

        // .name should NOT be rewritten (it's property access, not reference)
        let source = "const userName = user.name";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_transform_aliased_prop() {
        let mut bindings = PropsDestructuredBindings::default();
        // prop key is "message", local name is "msg"
        bindings.bindings.insert(
            "message".to_compact_string(),
            PropsDestructureBinding {
                local: "msg".to_compact_string(),
                default: None,
            },
        );

        let source = "console.log(msg)";
        let result = transform_destructured_props(source, &bindings);
        insta::assert_snapshot!(result.as_str());
    }

    // ==================== Snapshot tests ====================

    #[allow(clippy::disallowed_macros)]
    mod snapshots {
        use super::{
            make_bindings, transform_destructured_props, PropsDestructureBinding,
            PropsDestructuredBindings,
        };
        use vize_carton::ToCompactString;

        #[test]
        fn test_basic_usage() {
            let bindings = make_bindings(&["foo"]);
            let source = "console.log(foo)";
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_nested_scope() {
            let bindings = make_bindings(&["foo", "bar"]);
            let source = r#"function test(foo) {
    console.log(foo)
    console.log(bar)
}"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_multiple_variable_declarations() {
            let bindings = make_bindings(&["foo"]);
            let source = r#"const bar = 'fish', hello = 'world'
console.log(foo)"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_function_param_same_name() {
            let bindings = make_bindings(&["value"]);
            let source = r#"function test(value) {
    try {
    } catch {
    }
}
console.log(value)"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_aliasing() {
            let mut bindings = PropsDestructuredBindings::default();
            bindings.bindings.insert(
                "foo".to_compact_string(),
                PropsDestructureBinding {
                    local: "x".to_compact_string(),
                    default: None,
                },
            );
            bindings.bindings.insert(
                "foo".to_compact_string(),
                PropsDestructureBinding {
                    local: "y".to_compact_string(),
                    default: None,
                },
            );
            let source = r#"let a = x
let b = y"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_computed_property() {
            let bindings = make_bindings(&["count"]);
            let source = r#"const double = computed(() => count * 2)
const triple = computed(function() { return count * 3 })"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_watch_callback() {
            let bindings = make_bindings(&["count"]);
            let source = r#"watch(() => count, (newVal, oldVal) => {
    console.log('changed from', oldVal, 'to', newVal)
})"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_template_literal() {
            let bindings = make_bindings(&["name", "age"]);
            let source = r#"const greeting = `Hello ${name}, you are ${age} years old`"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_object_shorthand_props() {
            let bindings = make_bindings(&["foo", "bar"]);
            let source = r#"const obj = { foo, bar, baz: 123 }"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_complex_nested_function() {
            let bindings = make_bindings(&["data", "config"]);
            let source = r#"function processData() {
    const result = data.map(item => {
        const inner = config.map(c => c.value)
        return { item, inner }
    })
    return result
}"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_for_loops() {
            let bindings = make_bindings(&["items", "index"]);
            let source = r#"for (let i = 0; i < items.length; i++) {
    console.log(items[i])
}
for (const item of items) {
    console.log(item)
}
for (const index in items) {
    console.log(index)
}"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_try_catch_finally() {
            let bindings = make_bindings(&["error", "data"]);
            let source = r#"try {
    console.log(data)
} catch (error) {
    console.log(error)
} finally {
    console.log(error)
}"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_conditional_expression() {
            let bindings = make_bindings(&["show", "msg"]);
            let source = r#"const display = show ? msg : 'hidden'
const result = show && msg || 'default'"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_member_expression_chain() {
            let bindings = make_bindings(&["user"]);
            let source = r#"const name = user.profile.name
const email = user?.contact?.email
const id = user['id']"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_array_methods() {
            let bindings = make_bindings(&["items"]);
            let source = r#"const filtered = items.filter(x => x > 0)
const mapped = items.map(x => x * 2)
const reduced = items.reduce((acc, x) => acc + x, 0)"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_destructuring_in_params() {
            let bindings = make_bindings(&["data"]);
            let source = r#"const fn = ({ x, y }) => x + y
console.log(data)"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_switch_statement() {
            let bindings = make_bindings(&["value", "msg"]);
            let source = r#"switch (value) {
    case 1:
        console.log(msg)
        break
    case 2:
        console.log('two')
        break
    default:
        console.log(value)
}"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_while_and_do_while() {
            let bindings = make_bindings(&["count"]);
            let source = r#"while (count > 0) {
    console.log(count)
}
do {
    console.log(count)
} while (count > 0)"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_rest_spread_member_access() {
            // const { actions, ...props } = defineProps<{...}>()
            // props.status should become __props.status
            let mut bindings = PropsDestructuredBindings::default();
            bindings.bindings.insert(
                "actions".to_compact_string(),
                PropsDestructureBinding {
                    local: "actions".to_compact_string(),
                    default: None,
                },
            );
            bindings.rest_id = Some("props".to_compact_string());

            let source = r#"const status = computed(() => {
    if (props.status.reblog) return props.status.reblog
    return props.status
})
const rebloggedBy = computed(() => props.status.reblog ? props.status.account : null)
console.log(actions)"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }

        #[test]
        fn test_rest_spread_bare_identifier() {
            // const { a, ...rest } = defineProps<{...}>()
            // bare `rest` should become `__props`
            let mut bindings = PropsDestructuredBindings::default();
            bindings.bindings.insert(
                "a".to_compact_string(),
                PropsDestructureBinding {
                    local: "a".to_compact_string(),
                    default: None,
                },
            );
            bindings.rest_id = Some("rest".to_compact_string());

            let source = r#"console.log(rest)
const b = rest"#;
            let result = transform_destructured_props(source, &bindings);
            insta::assert_snapshot!(result);
        }
    }
}
