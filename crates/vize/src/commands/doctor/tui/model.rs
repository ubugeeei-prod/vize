//! Application-owned filters and interaction state around Fresco's workspace.

mod filter;

use std::path::Path;

use vize_doctor::{DoctorCategory, DoctorFinding, DoctorReport, FindingSeverity};
use vize_fresco::{
    Cursor, DiagnosticWorkspaceAction, DiagnosticWorkspaceCommandOutcome,
    DiagnosticWorkspaceKeymap, DiagnosticWorkspaceState, Key, KeyEvent, KeyEventKind,
    terminal::CursorShape,
};
use vize_s0::{String, ToCompactString};

use super::super::DoctorSource;
use filter::{cycle, search_document};

pub(super) use filter::{category_label, severity_label};

const CATEGORY_FILTERS: usize = DoctorCategory::ALL.len() + 1;
const SEVERITY_FILTERS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InteractionMode {
    Browse,
    Search,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InteractionOutcome {
    Changed,
    Boundary,
    OpenSource,
    Exit,
}

pub(super) struct DoctorTuiModel<'a> {
    report: &'a DoctorReport,
    workspace: DiagnosticWorkspaceState<usize, usize>,
    keymap: DiagnosticWorkspaceKeymap,
    finding_keys: Vec<usize>,
    evidence_keys: Vec<usize>,
    search_documents: Vec<String>,
    search: String,
    status: String,
    category_filter: usize,
    severity_filter: usize,
    mode: InteractionMode,
}

impl<'a> DoctorTuiModel<'a> {
    pub(super) fn new(report: &'a DoctorReport, width: u16, height: u16) -> Self {
        let search_documents = report.findings().iter().map(search_document).collect();
        let mut model = Self {
            report,
            workspace: DiagnosticWorkspaceState::new(width, height),
            keymap: DiagnosticWorkspaceKeymap::default(),
            finding_keys: Vec::with_capacity(report.findings().len()),
            evidence_keys: Vec::new(),
            search_documents,
            search: String::new(""),
            status: String::from("Ready"),
            category_filter: 0,
            severity_filter: 0,
            mode: InteractionMode::Browse,
        };
        model.rebuild_findings();
        model
    }

    pub(super) fn report(&self) -> &DoctorReport {
        self.report
    }

    pub(super) const fn workspace(&self) -> &DiagnosticWorkspaceState<usize, usize> {
        &self.workspace
    }

    pub(super) fn keymap(&self) -> &DiagnosticWorkspaceKeymap {
        &self.keymap
    }

    pub(super) fn finding_keys(&self) -> &[usize] {
        &self.finding_keys
    }

    pub(super) const fn mode(&self) -> InteractionMode {
        self.mode
    }

    pub(super) fn search(&self) -> &str {
        &self.search
    }

    pub(super) fn status(&self) -> &str {
        &self.status
    }

    pub(super) fn category_label(&self) -> &'static str {
        self.category_value().map_or("all", category_label)
    }

    pub(super) fn severity_label(&self) -> &'static str {
        self.severity_value().map_or("all", severity_label)
    }

    pub(super) fn selected_finding(&self) -> Option<&DoctorFinding> {
        self.workspace
            .findings()
            .selected_key()
            .and_then(|index| self.report.findings().get(*index))
    }

    pub(super) fn selected_finding_key(&self) -> Option<usize> {
        self.workspace.findings().selected_key().copied()
    }

    pub(super) fn selected_evidence_key(&self) -> Option<usize> {
        self.workspace.evidence().selected_key().copied()
    }

    pub(super) fn resize(&mut self, width: u16, height: u16) {
        let _ = self.workspace.resize(width, height);
    }

    pub(super) fn set_detail_rows(&mut self, rows: usize) {
        let _ = self.workspace.set_detail_content_rows(rows);
    }

    pub(super) fn set_status(&mut self, status: &str) {
        self.status = String::from(status);
    }

    pub(super) fn place_cursor(&self, cursor: &mut Cursor) {
        if self.mode != InteractionMode::Search || self.workspace.layout().height() < 2 {
            cursor.hide();
            return;
        }
        let width = self.workspace.layout().width();
        if width == 0 {
            cursor.hide();
            return;
        }
        let x =
            (2 + vize_fresco::TextWidth::width(&self.search)).min(usize::from(width - 1)) as u16;
        cursor.move_to(x, 1);
        cursor.set_shape(CursorShape::Bar);
        cursor.show();
    }

    pub(super) fn handle_paste(&mut self, text: &str) -> InteractionOutcome {
        if self.mode != InteractionMode::Search {
            return InteractionOutcome::Boundary;
        }
        self.search
            .extend(text.chars().filter(|character| !character.is_control()));
        self.rebuild_findings();
        InteractionOutcome::Changed
    }

    pub(super) fn handle_key(&mut self, event: &KeyEvent) -> InteractionOutcome {
        if event.kind == KeyEventKind::Release {
            return InteractionOutcome::Boundary;
        }
        if event.is_ctrl_c() {
            return InteractionOutcome::Exit;
        }
        match self.mode {
            InteractionMode::Search => return self.handle_search_key(event),
            InteractionMode::Help => {
                if event.is_escape()
                    || self.keymap.resolve(event)
                        == Some(vize_fresco::DiagnosticWorkspaceCommand::Help)
                {
                    self.mode = InteractionMode::Browse;
                    return InteractionOutcome::Changed;
                }
            }
            InteractionMode::Browse => {}
        }

        let Some(command) = self.keymap.resolve(event) else {
            return InteractionOutcome::Boundary;
        };
        let selected = self.selected_finding_key();
        match self
            .workspace
            .apply_command(command, &self.finding_keys, &self.evidence_keys)
        {
            DiagnosticWorkspaceCommandOutcome::Changed => {
                if selected != self.selected_finding_key() {
                    self.sync_evidence();
                }
                InteractionOutcome::Changed
            }
            DiagnosticWorkspaceCommandOutcome::Boundary => {
                self.set_status("Navigation boundary");
                InteractionOutcome::Boundary
            }
            DiagnosticWorkspaceCommandOutcome::Dispatch(action) => self.dispatch(action),
        }
    }

    pub(super) fn source_position(&self, sources: &[DoctorSource]) -> (u64, u64) {
        let Some(finding) = self.selected_finding() else {
            return (1, 1);
        };
        let Some(source) = sources
            .iter()
            .find(|source| source.path == Path::new(finding.primary.path.as_str()))
        else {
            return (1, u64::from(finding.primary.start).saturating_add(1));
        };
        let mut offset = (finding.primary.start as usize).min(source.source.len());
        while offset > 0 && !source.source.is_char_boundary(offset) {
            offset -= 1;
        }
        let prefix = &source.source[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u64 + 1;
        let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
        let column = prefix[line_start..].chars().count() as u64 + 1;
        (line, column)
    }

    fn handle_search_key(&mut self, event: &KeyEvent) -> InteractionOutcome {
        if event.is_escape() || event.is_enter() {
            self.mode = InteractionMode::Browse;
            return InteractionOutcome::Changed;
        }
        if event.is_backspace() {
            if self.search.pop().is_some() {
                self.rebuild_findings();
                return InteractionOutcome::Changed;
            }
            return InteractionOutcome::Boundary;
        }
        let plain_or_shift = !event.modifiers.ctrl
            && !event.modifiers.alt
            && !event.modifiers.super_key
            && !event.modifiers.hyper
            && !event.modifiers.meta;
        if plain_or_shift
            && let Key::Char(character) = event.key
            && !character.is_control()
        {
            self.search.push(character);
            self.rebuild_findings();
            return InteractionOutcome::Changed;
        }
        InteractionOutcome::Boundary
    }

    fn dispatch(&mut self, action: DiagnosticWorkspaceAction) -> InteractionOutcome {
        match action {
            DiagnosticWorkspaceAction::NextCategory => self.cycle_category(true),
            DiagnosticWorkspaceAction::PreviousCategory => self.cycle_category(false),
            DiagnosticWorkspaceAction::NextSeverity => self.cycle_severity(true),
            DiagnosticWorkspaceAction::PreviousSeverity => self.cycle_severity(false),
            DiagnosticWorkspaceAction::Search => self.mode = InteractionMode::Search,
            DiagnosticWorkspaceAction::Help => self.mode = InteractionMode::Help,
            DiagnosticWorkspaceAction::OpenSource => return InteractionOutcome::OpenSource,
            DiagnosticWorkspaceAction::Exit => return InteractionOutcome::Exit,
        }
        InteractionOutcome::Changed
    }

    fn cycle_category(&mut self, forward: bool) {
        self.category_filter = cycle(self.category_filter, CATEGORY_FILTERS, forward);
        self.rebuild_findings();
    }

    fn cycle_severity(&mut self, forward: bool) {
        self.severity_filter = cycle(self.severity_filter, SEVERITY_FILTERS, forward);
        self.rebuild_findings();
    }

    fn rebuild_findings(&mut self) {
        self.finding_keys.clear();
        let query = self.search.to_lowercase();
        for (index, finding) in self.report.findings().iter().enumerate() {
            let category_matches = self
                .category_value()
                .is_none_or(|category| finding.category == category);
            let severity_matches = self
                .severity_value()
                .is_none_or(|severity| finding.assessment.severity == severity);
            let search_matches =
                query.is_empty() || self.search_documents[index].contains(query.as_str());
            if category_matches && severity_matches && search_matches {
                self.finding_keys.push(index);
            }
        }
        let _ = self.workspace.reconcile_findings(&self.finding_keys);
        self.sync_evidence();
        self.status = self.finding_keys.len().to_compact_string();
        self.status.push_str(" matching finding(s)");
    }

    fn sync_evidence(&mut self) {
        self.evidence_keys.clear();
        if let Some(finding) = self.selected_finding() {
            self.evidence_keys.extend(0..finding.evidence.len());
        }
        let _ = self.workspace.reconcile_evidence(&self.evidence_keys);
    }

    fn category_value(&self) -> Option<DoctorCategory> {
        self.category_filter
            .checked_sub(1)
            .and_then(|index| DoctorCategory::ALL.get(index).copied())
    }

    fn severity_value(&self) -> Option<FindingSeverity> {
        match self.severity_filter {
            0 => None,
            1 => Some(FindingSeverity::Error),
            2 => Some(FindingSeverity::Warning),
            _ => Some(FindingSeverity::Notice),
        }
    }
}
