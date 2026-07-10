use super::ReliefRenderOp;
use vize_relief::TemplateChildNode;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ReliefChildren<'a> {
    children: &'a [TemplateChildNode<'a>],
}

impl<'a> ReliefChildren<'a> {
    pub const fn new(children: &'a [TemplateChildNode<'a>]) -> Self {
        Self { children }
    }

    pub fn rendered(self) -> impl Iterator<Item = (ReliefRenderOp<'a>, &'a TemplateChildNode<'a>)> {
        self.children.iter().filter_map(|child| {
            let operation = ReliefRenderOp::from_template_child(child);
            if matches!(
                operation,
                ReliefRenderOp::Comment {
                    is_directive: true,
                    ..
                }
            ) {
                None
            } else {
                Some((operation, child))
            }
        })
    }
}
