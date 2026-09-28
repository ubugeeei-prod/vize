const CDATA: &[u8; 6] = b"CDATA[";
const CDATA_END: &[u8; 3] = b"]]>";
const COMMENT_END: &[u8; 3] = b"-->";
const TITLE_END: &[u8; 7] = b"</title";
const TEXTAREA_END: &[u8; 10] = b"</textarea";
const SCRIPT_END: &[u8; 8] = b"</script";
const STYLE_END: &[u8; 7] = b"</style";

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

    /// The first byte of [`Self::bytes`].
    #[inline]
    pub fn first_byte(self) -> u8 {
        match self {
            Self::Cdata => CDATA[0],
            Self::CdataEnd => CDATA_END[0],
            Self::CommentEnd => COMMENT_END[0],
            Self::TitleEnd => TITLE_END[0],
            Self::TextareaEnd => TEXTAREA_END[0],
            Self::ScriptEnd => SCRIPT_END[0],
            Self::StyleEnd => STYLE_END[0],
        }
    }
}
