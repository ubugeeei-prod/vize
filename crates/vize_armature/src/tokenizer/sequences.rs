const CDATA: &[u8] = b"CDATA[";
const CDATA_END: &[u8] = b"]]>";
const COMMENT_END: &[u8] = b"-->";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Sequence {
    Cdata,
    CdataEnd,
    CommentEnd,
}

impl Sequence {
    #[inline]
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::Cdata => CDATA,
            Self::CdataEnd => CDATA_END,
            Self::CommentEnd => COMMENT_END,
        }
    }
}
