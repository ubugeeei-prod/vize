use rkyv::ser::serializers::{AllocScratchError, CompositeSerializerError};
use std::{convert::Infallible, io};

// Errors that can occur during processing/modifying source map
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum SourceMapErrorType {
    // NB: 0 is reserved for OK.
    // The mappings contained a negative line, column, source index, or name index.
    UnexpectedNegativeNumber = 1,

    // The mappings contained a number larger than `u32::MAX`.
    UnexpectedlyBigNumber = 2,

    // Reached EOF while in the middle of parsing a VLQ.
    VlqUnexpectedEof = 3,

    // Encountered an invalid base 64 character while parsing a VLQ.
    VlqInvalidBase64 = 4,

    // VLQ encountered a number that, when decoded, would not fit in a u32.
    VlqOverflow = 5,

    // General IO Error
    IOError = 6,

    // Name out of range
    NameOutOfRange = 7,

    // Source out of range
    SourceOutOfRange = 8,

    // Failed to write buffer
    BufferError = 9,

    // FilePath is invalid
    InvalidFilePath = 10,

    // Failed to convert utf-8 to array
    FromUtf8Error = 11,

    // Failed to serialize to JSON
    JSONError = 12,

    // Failed to parse data url
    #[cfg(feature = "json")]
    DataUrlError = 13,
}

#[derive(Debug)]
pub struct SourceMapError {
    pub error_type: SourceMapErrorType,
    pub reason: Option<String>,
}

impl SourceMapError {
    pub fn new(error_type: SourceMapErrorType) -> Self {
        Self {
            error_type,
            reason: None,
        }
    }

    pub fn new_with_reason(error_type: SourceMapErrorType, reason: &str) -> Self {
        Self {
            error_type,
            reason: Some(String::from(reason)),
        }
    }
}

impl std::error::Error for SourceMapError {}

impl std::fmt::Display for SourceMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Prefix all errors, so it's obvious they originate from this library
        write!(f, "[parcel-sourcemap] ")?;

        // Convert error type into an error message...
        match self.error_type {
            SourceMapErrorType::UnexpectedNegativeNumber => {
                write!(f, "Unexpected Negative Number")?;
            }
            SourceMapErrorType::UnexpectedlyBigNumber => {
                write!(f, "Unexpected Big Number")?;
            }
            SourceMapErrorType::VlqUnexpectedEof => {
                write!(f, "VLQ Unexpected end of file")?;
            }
            SourceMapErrorType::VlqInvalidBase64 => {
                write!(f, "VLQ Invalid Base 64 value")?;
            }
            SourceMapErrorType::VlqOverflow => {
                write!(f, "VLQ Value overflowed, does not fit in u32")?;
            }
            SourceMapErrorType::IOError => {
                write!(f, "IO Error")?;
            }
            SourceMapErrorType::NameOutOfRange => {
                write!(f, "Name out of range")?;
            }
            SourceMapErrorType::SourceOutOfRange => {
                write!(f, "Source out of range")?;
            }
            SourceMapErrorType::InvalidFilePath => {
                write!(f, "Invalid FilePath")?;
            }
            SourceMapErrorType::BufferError => {
                write!(
                    f,
                    "Something went wrong while writing/reading a sourcemap buffer"
                )?;
            }
            SourceMapErrorType::FromUtf8Error => {
                write!(f, "Could not convert utf-8 array to string")?;
            }
            SourceMapErrorType::JSONError => {
                write!(f, "Error reading or writing to JSON")?;
            }
            #[cfg(feature = "json")]
            SourceMapErrorType::DataUrlError => {
                write!(f, "Error parsing data url")?;
            }
        }

        // Add reason to error string if there is one

        if let Some(r) = &self.reason {
            write!(f, ", ")?;
            write!(f, "{}", &r[..])?;
        }

        Ok(())
    }
}

impl From<vlq::Error> for SourceMapError {
    #[inline]
    fn from(e: vlq::Error) -> SourceMapError {
        match e {
            vlq::Error::UnexpectedEof => SourceMapError::new(SourceMapErrorType::VlqUnexpectedEof),
            vlq::Error::InvalidBase64(_) => {
                SourceMapError::new(SourceMapErrorType::VlqInvalidBase64)
            }
            vlq::Error::Overflow => SourceMapError::new(SourceMapErrorType::VlqOverflow),
        }
    }
}

impl From<io::Error> for SourceMapError {
    #[inline]
    fn from(_err: io::Error) -> SourceMapError {
        SourceMapError::new(SourceMapErrorType::IOError)
    }
}

impl From<Infallible> for SourceMapError {
    #[inline]
    fn from(_err: Infallible) -> SourceMapError {
        SourceMapError::new(SourceMapErrorType::BufferError)
    }
}

impl From<CompositeSerializerError<Infallible, AllocScratchError, Infallible>> for SourceMapError {
    #[inline]
    fn from(
        _err: CompositeSerializerError<Infallible, AllocScratchError, Infallible>,
    ) -> SourceMapError {
        SourceMapError::new(SourceMapErrorType::BufferError)
    }
}

impl From<std::string::FromUtf8Error> for SourceMapError {
    #[inline]
    fn from(_err: std::string::FromUtf8Error) -> SourceMapError {
        SourceMapError::new(SourceMapErrorType::FromUtf8Error)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for SourceMapError {
    #[inline]
    fn from(_err: serde_json::Error) -> SourceMapError {
        SourceMapError::new(SourceMapErrorType::JSONError)
    }
}

#[cfg(feature = "json")]
impl From<data_url::DataUrlError> for SourceMapError {
    #[inline]
    fn from(_err: data_url::DataUrlError) -> SourceMapError {
        SourceMapError::new(SourceMapErrorType::DataUrlError)
    }
}
