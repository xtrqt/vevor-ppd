#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    PrintJob,
    ValidateJob,
    GetPrinterAttributes,
    Unknown(u16),
}

impl From<u16> for Operation {
    fn from(value: u16) -> Self {
        match value {
            0x0002 => Self::PrintJob,
            0x0004 => Self::ValidateJob,
            0x000b => Self::GetPrinterAttributes,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IppRequest {
    pub version_major: u8,
    pub version_minor: u8,
    pub operation: Operation,
    pub request_id: u32,
    pub document: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    SuccessfulOk = 0x0000,
    ClientErrorDocumentFormatNotSupported = 0x040a,
    ServerErrorOperationNotSupported = 0x0501,
    ServerErrorInternalError = 0x0500,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueTag {
    Boolean = 0x22,
    Enum = 0x23,
    TextWithoutLanguage = 0x41,
    NameWithoutLanguage = 0x42,
    Keyword = 0x44,
    Uri = 0x45,
    Charset = 0x47,
    NaturalLanguage = 0x48,
    MimeMediaType = 0x49,
}
