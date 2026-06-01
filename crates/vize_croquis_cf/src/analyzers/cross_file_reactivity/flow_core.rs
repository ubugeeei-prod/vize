use super::analyzer::CrossFileReactivityAnalyzer;

impl<'a> CrossFileReactivityAnalyzer<'a> {
    pub(super) fn track_cross_file_flows(&mut self) {
        // Track composable import flows
        self.track_composable_flows();

        // Track provide/inject flows
        self.track_provide_inject_flows();

        // Track props flows
        self.track_props_flows();
    }
}
