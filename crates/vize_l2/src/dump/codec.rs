//! Closed document grammars, selected before parsing or printing a page.

#[derive(Debug, Clone, Copy)]
pub(super) enum Grammar {
    CurrentV2,
    HistoricalV1,
}

impl Grammar {
    pub(super) const fn header(self) -> &'static str {
        match self {
            Self::CurrentV2 => "l2-dump-v2",
            Self::HistoricalV1 => "disegno",
        }
    }

    pub(super) const fn ops_header(self) -> &'static str {
        match self {
            Self::CurrentV2 => "l2-dump-v2.ops",
            Self::HistoricalV1 => "disegno.ops",
        }
    }
}
