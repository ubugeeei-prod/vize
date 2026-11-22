// Both constructors instantiate this single original-descriptor and File assembly.
{
    use alloc::vec::Vec;
    let descriptor = Vue.observe_descriptor(allocator, source, options);
    let mut issues = Vec::new();
    let view = match descriptor.admitted() {
        Ok(view) => view,
        Err(_) => {
            issues.push(NativeSfcIssue {
                container_index: None,
                span: descriptor.root().map(|root| root.whole_block().span()),
                kind: NativeSfcIssueKind::Descriptor,
            });
            return NativeSfcObservation::new(descriptor, Vec::new(), None, None, issues);
        }
    };
    let mut producer = match crate::vue_file::VueFileProducer::new(allocator, source) {
        Ok(producer) => Some(producer),
        Err(error) => {
            issues.push(NativeSfcIssue {
                container_index: None,
                span: Some(view.root().whole_block().span()),
                kind: NativeSfcIssueKind::FileConstruction(error),
            });
            None
        }
    };
    let mut scripts = Vec::new();
    for selected in [view.ordinary(), view.setup()].into_iter().flatten() {
        scripts.push(script::observe(
            allocator,
            selected,
            producer.as_mut(),
            &mut issues,
        ));
    }
    let mut template = view
        .template()
        .map(|selected| prepare(allocator, selected, view.template_lang(), &mut issues));
    if let Some(template) = template.as_mut() {
        template.construct(producer.as_mut(), &mut issues);
    }
    let file = producer.map(crate::vue_file::VueFileProducer::finish);
    if file.as_ref().is_some_and(Result::is_err) {
        issues.push(NativeSfcIssue {
            container_index: None,
            span: Some(view.root().whole_block().span()),
            kind: NativeSfcIssueKind::FileRejected,
        });
    }
    NativeSfcObservation::new(descriptor, scripts, template, file, issues)
}
