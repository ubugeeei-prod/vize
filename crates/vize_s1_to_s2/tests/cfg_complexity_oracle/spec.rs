//! The naive evaluator: `complexity-metrics.md` read a second time, over a
//! different representation, with different algorithms.
//!
//! - Input is the **owned folio** (`S2Folio`), not the arena tree, and
//!   every expression is **re-parsed** from its folio text into a fresh
//!   arena, not read from the retained AST.
//! - Cyclomatic complexity is `E − N + 2` of an **explicit control-flow
//!   graph** (a decision node per condition, a header with a back edge per
//!   loop, a diamond per operator and per `?:`), checked connected from
//!   entry to exit before it is measured.
//! - Nesting is **counted over an ancestor list** (a frame stack for
//!   regions, parent links for AST nodes), and an operator tree's runs come
//!   from sorting its nodes by operator position — no inherited counters.

use vize_s2::folio::{FolioBinding, FolioExpr, FolioIf, FolioName, FolioOp, S2Folio};

use super::ast::expression_facts;
use super::graph::Graph;

/// One breakdown row, comparable across both implementations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row {
    pub start: u32,
    pub end: u32,
    pub kind: &'static str,
    pub op: Option<u32>,
    pub nesting: u32,
    pub cyclomatic: u32,
    pub cognitive: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Naive {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub unknown: u32,
    pub max_nesting: u32,
    pub rows: Vec<Row>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Frame {
    Branch,
    ForBody,
    ScopedBody,
}

struct Eval {
    graph: Graph,
    frames: Vec<Frame>,
    rows: Vec<Row>,
    next_id: u32,
    max_nesting: u32,
}

/// Evaluate the metric over one template's folio.
pub fn evaluate(folio: &S2Folio) -> Naive {
    let mut eval = Eval {
        graph: Graph::default(),
        frames: Vec::new(),
        rows: Vec::new(),
        next_id: 0,
        max_nesting: 0,
    };
    let entry = eval.graph.node();
    let end = eval.region(&folio.ops, entry);
    let exit = eval.graph.node();
    eval.graph.edge(end, exit);
    let cyclomatic = eval.graph.mccabe(entry, exit);
    eval.rows.sort();
    Naive {
        cyclomatic,
        cognitive: eval.rows.iter().map(|row| row.cognitive).sum(),
        unknown: eval.rows.iter().filter(|row| row.kind == "unknown").count() as u32,
        max_nesting: eval.max_nesting,
        rows: eval.rows,
    }
}

impl Eval {
    fn nesting(&self) -> u32 {
        self.frames.len() as u32
    }

    fn mint(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id - 1
    }

    fn row(
        &mut self,
        kind: &'static str,
        span: (u32, u32),
        op: u32,
        nesting: u32,
        cyc: u32,
        cog: u32,
    ) {
        self.rows.push(Row {
            start: span.0,
            end: span.1,
            kind,
            op: Some(op),
            nesting,
            cyclomatic: cyc,
            cognitive: cog,
        });
    }

    fn region(&mut self, ops: &[FolioOp], mut cur: u32) -> u32 {
        for op in ops {
            self.max_nesting = self.max_nesting.max(self.nesting());
            cur = self.op(op, cur);
        }
        cur
    }

    fn inside(&mut self, frame: Frame, ops: &[FolioOp], cur: u32) -> u32 {
        self.frames.push(frame);
        let end = self.region(ops, cur);
        self.frames.pop();
        end
    }

    fn op(&mut self, op: &FolioOp, cur: u32) -> u32 {
        let id = self.mint();
        let line = self.graph.node();
        self.graph.edge(cur, line);
        match op {
            FolioOp::Element(element) => {
                let (cur, scoped) = self.owner(&element.bindings, line);
                self.children(&element.children, scoped, cur)
            }
            FolioOp::Component(component) => {
                let (cur, scoped) = self.owner(&component.bindings, line);
                self.children(&component.children, scoped, cur)
            }
            FolioOp::Text(_) | FolioOp::Comment(_) => line,
            FolioOp::Interpolation(interpolation) => self.expr(&interpolation.expression, id, line),
            FolioOp::If(if_op) => self.if_chain(if_op, id, line),
            FolioOp::For(for_op) => {
                let source = &for_op.binding.source;
                let n = self.nesting();
                self.row("v-for", span_of(source), id, n, 1, 1 + n);
                let cur = self.expr(source, id, line);
                let header = self.graph.node();
                self.graph.edge(cur, header);
                let body = self.graph.node();
                self.graph.edge(header, body);
                let body_end = self.inside(Frame::ForBody, &for_op.ops, body);
                self.graph.edge(body_end, header);
                let exit = self.graph.node();
                self.graph.edge(header, exit);
                exit
            }
            FolioOp::Slot(slot) => {
                let cur = self.name(&slot.name, id, line);
                let (cur, _) = self.owner(&slot.bindings, cur);
                self.region(&slot.fallback, cur)
            }
        }
    }

    fn children(&mut self, children: &[FolioOp], scoped: bool, cur: u32) -> u32 {
        if scoped {
            self.inside(Frame::ScopedBody, children, cur)
        } else {
            self.region(children, cur)
        }
    }

    fn if_chain(&mut self, if_op: &FolioIf, id: u32, mut cur: u32) -> u32 {
        let join = self.graph.node();
        let mut closed = false;
        for (index, branch) in if_op.branches.iter().enumerate() {
            let n = self.nesting();
            match &branch.condition {
                Some(condition) => {
                    let (kind, cog) = if index == 0 {
                        ("v-if", 1 + n)
                    } else {
                        ("v-else-if", 1)
                    };
                    self.row(kind, span_of(condition), id, n, 1, cog);
                    cur = self.expr(condition, id, cur);
                    let decision = self.graph.node();
                    self.graph.edge(cur, decision);
                    let taken = self.graph.node();
                    self.graph.edge(decision, taken);
                    let end = self.inside(Frame::Branch, &branch.ops, taken);
                    self.graph.edge(end, join);
                    let fallthrough = self.graph.node();
                    self.graph.edge(decision, fallthrough);
                    cur = fallthrough;
                }
                None => {
                    self.row("v-else", (branch.span.start, branch.span.end), id, n, 0, 1);
                    let end = self.inside(Frame::Branch, &branch.ops, cur);
                    self.graph.edge(end, join);
                    closed = true;
                }
            }
        }
        if !closed {
            self.graph.edge(cur, join);
        }
        join
    }

    fn owner(&mut self, bindings: &[FolioBinding], mut cur: u32) -> (u32, bool) {
        let mut scoped = false;
        for binding in bindings {
            let id = self.mint();
            let n = self.nesting();
            match binding {
                FolioBinding::Bind(bind) => {
                    cur = self.opt_name(bind.name.as_ref(), id, cur);
                    cur = self.opt_expr(bind.value.as_ref(), id, cur);
                }
                FolioBinding::On(on) => {
                    cur = self.opt_name(on.name.as_ref(), id, cur);
                    cur = self.opt_expr(on.handler.as_ref(), id, cur);
                }
                FolioBinding::Model(model) => {
                    cur = self.opt_name(model.argument.as_ref(), id, cur);
                    cur = self.expr(&model.contract.read, id, cur);
                }
                FolioBinding::SlotContent(content) => {
                    cur = self.opt_name(content.name.as_ref(), id, cur);
                    if content.params.is_some() {
                        self.row(
                            "scoped-slot",
                            (content.span.start, content.span.end),
                            id,
                            n,
                            0,
                            0,
                        );
                        scoped = true;
                    }
                }
                FolioBinding::VueSlotScope(scope) => {
                    if scope.params.is_some() {
                        self.row(
                            "scoped-slot",
                            (scope.span.start, scope.span.end),
                            id,
                            n,
                            0,
                            0,
                        );
                        scoped = true;
                    }
                }
                FolioBinding::VueDirective(directive) => {
                    cur = self.opt_name(directive.argument.as_ref(), id, cur);
                    cur = self.opt_expr(directive.value.as_ref(), id, cur);
                }
                FolioBinding::VueSync(sync) => cur = self.expr(&sync.value, id, cur),
                FolioBinding::VueMemo(memo) => cur = self.expr(&memo.value, id, cur),
                FolioBinding::VueShow(show) => cur = self.expr(&show.value, id, cur),
                FolioBinding::VueHtml(html) => cur = self.opt_expr(html.value.as_ref(), id, cur),
                FolioBinding::VueText(text) => cur = self.opt_expr(text.value.as_ref(), id, cur),
                FolioBinding::VueCssBind(_)
                | FolioBinding::VueOnce(_)
                | FolioBinding::VueCloak(_) => {}
            }
        }
        (cur, scoped)
    }

    fn name(&mut self, name: &FolioName, id: u32, cur: u32) -> u32 {
        match name {
            FolioName::Static(_) => cur,
            FolioName::Dynamic(expr) => self.expr(expr, id, cur),
        }
    }

    fn opt_name(&mut self, name: Option<&FolioName>, id: u32, cur: u32) -> u32 {
        name.map_or(cur, |name| self.name(name, id, cur))
    }

    fn opt_expr(&mut self, expr: Option<&FolioExpr>, id: u32, cur: u32) -> u32 {
        expr.map_or(cur, |expr| self.expr(expr, id, cur))
    }

    fn expr(&mut self, expr: &FolioExpr, id: u32, mut cur: u32) -> u32 {
        let n = self.nesting();
        let FolioExpr::Js { source, span } = expr else {
            self.row("unknown", span_of(expr), id, n, 0, 0);
            return cur;
        };
        for fact in expression_facts(source.as_str()) {
            let at = (span.start + fact.start, span.start + fact.end);
            let nesting = n + fact.enclosing_conditionals;
            let cognitive = if fact.kind == "conditional" {
                1 + nesting
            } else {
                fact.runs
            };
            self.row(fact.kind, at, id, nesting, fact.decisions, cognitive);
            for _ in 0..fact.decisions {
                cur = self.graph.diamond(cur);
            }
        }
        cur
    }
}

fn span_of(expr: &FolioExpr) -> (u32, u32) {
    let span = match expr {
        FolioExpr::Js { span, .. }
        | FolioExpr::Foreign { span, .. }
        | FolioExpr::Opaque { span, .. }
        | FolioExpr::Filter { span, .. } => span,
    };
    (span.start, span.end)
}
