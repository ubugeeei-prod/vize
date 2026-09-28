//! Explicit historical codecs for published extension contracts.
//!
//! Current dumps never select these codecs from their input text.

/// The frozen tree grammar used by the published `s2-page@1` contract.
pub mod v1 {
    use core::fmt;

    use vize_davinci::dump::{Dump, Error, Mode};

    use crate::dump::{Page as CurrentPage, parse, print};

    /// A published version1 wire page over the unchanged owned L2 tree.
    ///
    /// This wrapper has no artifact-key producer. Current dumps and keys
    /// always use [`CurrentPage`]'s version2 grammar.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct Page(CurrentPage);

    impl Page {
        /// Select the historical wire writer without copying the tree.
        #[must_use]
        pub fn from_current(page: CurrentPage) -> Self {
            Self(page)
        }

        /// Recover the identical owned tree after historical acceptance.
        #[must_use]
        pub fn into_current(self) -> CurrentPage {
            self.0
        }
    }

    impl Dump for Page {
        fn print<W: fmt::Write>(&self, w: &mut W, mode: Mode) -> fmt::Result {
            print::print(&self.0, w, print::Style::historical_v1(mode))
        }

        fn parse(input: &str) -> Result<Self, Error> {
            parse::parse_historical_v1(input).map(Self)
        }
    }
}
