#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    PrintJob,
    CreateJob,
    ValidateJob,
    GetJobs,
    GetPrinterAttributes,
    GetJobAttributes,
    SendDocument,
    CancelJob,
    Unknown(u16),
}

impl From<u16> for Operation {
    fn from(value: u16) -> Self {
        match value {
            0x0002 => Self::PrintJob,
            0x0005 => Self::CreateJob,
            0x0004 => Self::ValidateJob,
            0x000a => Self::GetJobs,
            0x000b => Self::GetPrinterAttributes,
            0x0009 => Self::GetJobAttributes,
            0x0006 => Self::SendDocument,
            0x0008 => Self::CancelJob,
            other => Self::Unknown(other),
        }
    }
}

impl Operation {
    pub fn code(self) -> i32 {
        match self {
            Self::PrintJob => 0x0002,
            Self::ValidateJob => 0x0004,
            Self::CreateJob => 0x0005,
            Self::SendDocument => 0x0006,
            Self::CancelJob => 0x0008,
            Self::GetJobAttributes => 0x0009,
            Self::GetJobs => 0x000a,
            Self::GetPrinterAttributes => 0x000b,
            Self::Unknown(value) => value as i32,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IppRequest {
    pub version_major: u8,
    pub version_minor: u8,
    pub operation: Operation,
    pub request_id: u32,
    pub attributes: Vec<IppAttribute>,
    pub document: Vec<u8>,
}

impl IppRequest {
    pub fn document_format(&self) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name == "document-format")
            .and_then(|attribute| std::str::from_utf8(&attribute.value).ok())
    }

    pub fn get_attribute(&self, name: &str) -> Option<&[u8]> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name == name)
            .map(|attribute| attribute.value.as_slice())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IppAttribute {
    pub name: String,
    pub value: Vec<u8>,
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
    Integer = 0x21,
    Boolean = 0x22,
    Enum = 0x23,
    Resolution = 0x32,
    TextWithoutLanguage = 0x41,
    NameWithoutLanguage = 0x42,
    Keyword = 0x44,
    Uri = 0x45,
    Charset = 0x47,
    NaturalLanguage = 0x48,
    MimeMediaType = 0x49,
}
