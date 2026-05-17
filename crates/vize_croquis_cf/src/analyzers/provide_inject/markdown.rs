use super::types::{ProvideInjectTree, ProvideNode};
use crate::registry::ModuleRegistry;
use vize_carton::{CompactString, String, cstr};

impl ProvideInjectTree {
    /// Render the tree as a markdown string for visualization.
    pub fn to_markdown(&self, registry: &ModuleRegistry) -> String {
        let mut output = String::with_capacity(4096);
        output.push_str("## Provide/Inject Tree\n\n");

        if self.roots.is_empty() {
            output.push_str("_No provide/inject relationships found._\n");
            return output;
        }

        for root in &self.roots {
            Self::render_node(&mut output, root, registry, 0);
        }

        output
    }

    fn render_node(
        output: &mut String,
        node: &ProvideNode,
        registry: &ModuleRegistry,
        depth: usize,
    ) {
        use std::fmt::Write;

        let indent = "  ".repeat(depth);
        let name = node
            .component_name
            .as_deref()
            .or_else(|| {
                registry
                    .get(node.file_id)
                    .and_then(|e| e.path.file_stem()?.to_str())
            })
            .unwrap_or("<unknown>");

        // Component name
        writeln!(output, "{}📦 **{}**", indent, name).ok();

        // Provides
        if !node.provides.is_empty() {
            for p in &node.provides {
                let type_str = p
                    .value_type
                    .as_deref()
                    .map(|t| cstr!(": `{t}`"))
                    .unwrap_or_default();
                let consumers = if p.consumer_count > 0 {
                    cstr!(" → {} consumer(s)", p.consumer_count)
                } else {
                    CompactString::new(" ⚠️ _unused_")
                };
                writeln!(
                    output,
                    "{}  🔹 provide(`\"{}\"`){}{} ",
                    indent, p.key, type_str, consumers
                )
                .ok();
            }
        }

        // Injects
        if !node.injects.is_empty() {
            for i in &node.injects {
                let default_str = if i.has_default { " (has default)" } else { "" };
                let provider_str = if i.provider.is_some() {
                    " ✅"
                } else {
                    " ❌ _no provider_"
                };
                writeln!(
                    output,
                    "{}  🔸 inject(`\"{}\"`){}{} ",
                    indent, i.key, default_str, provider_str
                )
                .ok();
            }
        }

        // Children
        for child in &node.children {
            Self::render_node(output, child, registry, depth + 1);
        }
    }
}
