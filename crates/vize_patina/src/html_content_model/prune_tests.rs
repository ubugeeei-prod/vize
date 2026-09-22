//! FP-3: a child subtree behind `v-if="prop"` is pruned exactly when the
//! usage leaves `prop` unpassed and the oracle proves its absent value falsy.

#[cfg(test)]
mod pruning {
    use crate::html_content_model::{
        Skeleton, authored_skeleton, composable_skeleton, compose_with,
    };
    use vize_s0::Allocator;

    /// The class ids composition proves for `parent` rendering `Kv`, with
    /// `falsy` the props the oracle proves falsy when unpassed.
    fn classes(parent: &str, child: &str, falsy: &[&str]) -> Vec<&'static str> {
        let skeletons: Vec<Skeleton> = [parent, child]
            .iter()
            .map(|source| composable_skeleton(&Allocator::with_capacity(4096), source))
            .collect();
        let resolve = |_: u32, tag: &str| (tag == "Kv").then_some(1);
        let absent_falsy = |file: u32, prop: &str| file == 1 && falsy.contains(&prop);
        compose_with(&skeletons, &resolve, &absent_falsy)
            .into_iter()
            .map(|finding| finding.class.id())
            .collect()
    }

    const KV: &str = "<span>v<button v-if=\"copy\">c</button></span>";

    #[test]
    fn an_unpassed_falsy_guard_prunes_the_branch() {
        assert_eq!(
            classes("<button><Kv /></button>", KV, &["copy"]),
            Vec::<&str>::new()
        );
        // The rest of the child is still checked in the parent's chain.
        assert_eq!(
            classes(
                "<button><Kv /></button>",
                "<div><button v-if=\"copy\">c</button></div>",
                &["copy"]
            ),
            ["phrasing-content-expected"]
        );
    }

    #[test]
    fn a_guard_that_may_hold_is_checked() {
        let nested = ["button-auto-closed"];
        // Passed statically, bound, camelized from kebab-case, or spread.
        for parent in [
            "<button><Kv copy=\"x\" /></button>",
            "<button><Kv :copy=\"x\" /></button>",
            "<button><Kv v-bind:copy=\"x\" /></button>",
            "<button><Kv v-bind=\"attrs\" /></button>",
            "<button><Kv :[name]=\"x\" /></button>",
        ] {
            assert_eq!(classes(parent, KV, &["copy"]), nested, "{parent}");
        }
        assert_eq!(
            classes(
                "<button><Kv copy-text=\"x\" /></button>",
                "<span><button v-if=\"copyText\">c</button></span>",
                &["copyText"]
            ),
            nested
        );
        // The oracle does not prove the absent value falsy.
        assert_eq!(classes("<button><Kv /></button>", KV, &[]), nested);
        // A scope variable of the same name shadows the prop; a non-bare guard
        // is not read.
        for child in [
            "<span v-for=\"copy in items\"><button v-if=\"copy\">c</button></span>",
            "<span><button v-for=\"copy in items\" v-if=\"copy\">c</button></span>",
            "<span><button v-if=\"copy.length\">c</button></span>",
        ] {
            assert_eq!(classes("<button><Kv /></button>", child, &["copy"]), nested);
        }
    }

    #[test]
    fn only_composable_skeletons_record_prop_facts() {
        let source = "<Kv copy=\"x\" v-on:pick=\"f\" v-model=\"m\"><b v-if=\"on\" /></Kv>";
        let allocator = Allocator::with_capacity(4096);
        assert_eq!(
            authored_skeleton(&allocator, source).props,
            Default::default()
        );
        let props = composable_skeleton(&allocator, source).props;
        let passed: Vec<&str> = props.passed.iter().map(|(_, name)| name.as_str()).collect();
        assert_eq!(passed, ["copy", "onPick", "modelValue"]);
        assert_eq!(props.guards.len(), 1);
        assert_eq!(props.guards[0].1.as_str(), "on");
        assert!(props.opaque.is_empty());
    }
}
