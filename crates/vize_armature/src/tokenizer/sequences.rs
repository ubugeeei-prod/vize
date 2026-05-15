const CDATA: &[u8] = b"CDATA[";
const CDATA_END: &[u8] = b"]]>";
const COMMENT_END: &[u8] = b"-->";
const TITLE_END: &[u8] = b"</title";
const TEXTAREA_END: &[u8] = b"</textarea";
const SCRIPT_END: &[u8] = b"</script";
const STYLE_END: &[u8] = b"</style";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Sequence {
    Cdata,
    CdataEnd,
    CommentEnd,
    TitleEnd,
    TextareaEnd,
    ScriptEnd,
    StyleEnd,
}

impl Sequence {
    #[inline]
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::Cdata => CDATA,
            Self::CdataEnd => CDATA_END,
            Self::CommentEnd => COMMENT_END,
            Self::TitleEnd => TITLE_END,
            Self::TextareaEnd => TEXTAREA_END,
            Self::ScriptEnd => SCRIPT_END,
            Self::StyleEnd => STYLE_END,
        }
    }
}
