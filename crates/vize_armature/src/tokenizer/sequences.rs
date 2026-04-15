const CDATA: [u8; 6] = [0x43, 0x44, 0x41, 0x54, 0x41, 0x5b]; // CDATA[

const CDATA_END: [u8; 3] = [0x5d, 0x5d, 0x3e]; // ]]>

const COMMENT_END: [u8; 3] = [0x2d, 0x2d, 0x3e]; // `-->`

const SCRIPT_END: [u8; 8] = [0x3c, 0x2f, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74]; // `</script`

const STYLE_END: [u8; 7] = [0x3c, 0x2f, 0x73, 0x74, 0x79, 0x6c, 0x65]; // `</style`

const TITLE_END: [u8; 7] = [0x3c, 0x2f, 0x74, 0x69, 0x74, 0x6c, 0x65]; // `</title`

const TEXTAREA_END: [u8; 10] = [0x3c, 0x2f, 0x74, 0x65, 0x78, 0x74, 0x61, 0x72, 0x65, 0x61]; // `</textarea`

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sequences {
    Cdata,
    CdataEnd,
    CommentEnd,
    ScriptEnd,
    StyleEnd,
    TitleEnd,
    TextareaEnd,
}

impl Sequences {
    #[inline]
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::Cdata => &CDATA[..],
            Self::CdataEnd => &CDATA_END[..],
            Self::CommentEnd => &COMMENT_END[..],
            Self::ScriptEnd => &SCRIPT_END[..],
            Self::StyleEnd => &STYLE_END[..],
            Self::TitleEnd => &TITLE_END[..],
            Self::TextareaEnd => &TEXTAREA_END[..],
        }
    }
}
