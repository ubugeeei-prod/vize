//! Observe the owning L2 artifact while projecting the existing SDK document.
use super::{
    Allocator, HostError, PluginDocument, SfcParseOptions, SourceRoot, Walk, lower_source_block,
    parse_sfc,
};
use vize_l1_to_l2::lower::Lowered;

impl PluginDocument {
    /// Build a plugin document and run a read-only pass on its original template.
    /// Both products use the complete SFC's coordinate frame and dense op IDs.
    /// A file without a template supplies an empty document and no pass result.
    pub fn with_template_pass<R>(
        source: &str,
        filename: &str,
        pass: impl FnOnce(&Lowered<'_>) -> R,
    ) -> Result<(Self, Option<R>), HostError> {
        let descriptor = parse_sfc(source, SfcParseOptions::default())
            .map_err(|error| HostError::Split(error.message.to_string()))?;
        let mut document = Self {
            filename: filename.to_owned(),
            source: source.to_owned(),
            production: Default::default(),
            nodes: Vec::new(),
            scopes: Vec::new(),
        };
        let Some(template) = descriptor.template else {
            return Ok((document, None));
        };
        if !vize_l1::container::lang::is_html_template(template.lang.as_deref())
            || template.src.is_some()
        {
            return Err(HostError::Split("JS plugin visits require an inline HTML template; preprocess template dialects through their compiler integration".into()));
        }
        let Some(content) = source.get(template.loc.start..template.loc.end) else {
            return Ok((document, None));
        };
        let root = SourceRoot::new(source).map_err(|_| HostError::Split("too large".into()))?;
        let block = root
            .block(content, template.loc.start as u32)
            .map_err(|_| HostError::Split("template is not a source slice".into()))?;
        let allocator = Allocator::new();
        let (tree, errors) = vize_l1::markup::parse(&allocator, content);
        let lowered = lower_source_block(&allocator, &tree, &errors, block);
        let result = pass(&lowered);
        let mut walk = Walk {
            nodes: Vec::with_capacity(lowered.op_count as usize),
        };
        walk.region(&lowered.artifact.root, None);
        document.nodes = walk.nodes;
        document.scopes = lowered
            .artifact
            .scopes
            .iter()
            .map(|(id, scope)| {
                let names = scope
                    .bindings
                    .iter()
                    .map(|binding| binding.name.to_string())
                    .collect();
                (id.index(), names)
            })
            .collect();
        document.scopes.sort_unstable_by_key(|(id, _)| *id);
        Ok((document, Some(result)))
    }
}
