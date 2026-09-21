//! The naive evaluator's control-flow graph: explicit nodes and edges,
//! McCabe's `E − N + 2` measured only after every node is proven to lie on
//! an entry-to-exit path.

#[derive(Default)]
pub struct Graph {
    nodes: u32,
    edges: Vec<(u32, u32)>,
}

impl Graph {
    pub fn node(&mut self) -> u32 {
        self.nodes += 1;
        self.nodes - 1
    }

    pub fn edge(&mut self, from: u32, to: u32) {
        self.edges.push((from, to));
    }

    /// `from → decision → (then → join | join)`: one binary decision.
    pub fn diamond(&mut self, from: u32) -> u32 {
        let decision = self.node();
        let then = self.node();
        let join = self.node();
        self.edge(from, decision);
        self.edge(decision, then);
        self.edge(then, join);
        self.edge(decision, join);
        join
    }

    /// McCabe's number, after proving every node lies on an entry→exit path.
    pub fn mccabe(&self, entry: u32, exit: u32) -> u32 {
        let forward = self.reach(entry, false);
        let backward = self.reach(exit, true);
        assert!(
            forward.iter().zip(&backward).all(|(f, b)| *f && *b),
            "naive CFG is not entry-to-exit connected"
        );
        let edges = u32::try_from(self.edges.len()).expect("small graph");
        edges + 2 - self.nodes
    }

    fn reach(&self, start: u32, reverse: bool) -> Vec<bool> {
        let mut next: Vec<Vec<u32>> = vec![Vec::new(); self.nodes as usize];
        for &(from, to) in &self.edges {
            let (here, there) = if reverse { (to, from) } else { (from, to) };
            next[here as usize].push(there);
        }
        let mut seen = vec![false; self.nodes as usize];
        let mut work = vec![start];
        while let Some(node) = work.pop() {
            if !std::mem::replace(&mut seen[node as usize], true) {
                work.extend(&next[node as usize]);
            }
        }
        seen
    }
}
