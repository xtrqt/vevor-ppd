use super::codec::{parse_request, ResponseBuilder};
use super::model::{IppRequest, Operation, Status, ValueTag};
use crate::app::AppState;
use crate::driver::{vevor, LabelOptions, PrintJob, RasterPage};
use crate::output;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use std::sync::Arc;
use tracing::{error, warn};

pub async fn handle_ipp(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    let request = match parse_request(&body) {
        Ok(request) => request,
        Err(err) => {
            warn!(error = %err, "failed to parse IPP request");
            return ipp_response_headers(Vec::new(), StatusCode::BAD_REQUEST);
        }
    };

    let response = match request.operation {
        Operation::GetPrinterAttributes => {
            printer_attributes(&state, &request, Status::SuccessfulOk)
        }
        Operation::ValidateJob => printer_attributes(&state, &request, Status::SuccessfulOk),
        Operation::PrintJob => match print_job(&state, &request).await {
            Ok(()) => printer_attributes(&state, &request, Status::SuccessfulOk),
            Err(PrintError::UnsupportedFormat) => printer_attributes(
                &state,
                &request,
                Status::ClientErrorDocumentFormatNotSupported,
            ),
            Err(PrintError::Internal(err)) => {
                error!(error = %err, "print job failed");
                printer_attributes(&state, &request, Status::ServerErrorInternalError)
            }
        },
        Operation::Unknown(operation) => {
            warn!(operation, "unsupported IPP operation");
            printer_attributes(&state, &request, Status::ServerErrorOperationNotSupported)
        }
    };

    ipp_response_headers(response, StatusCode::OK)
}

fn ipp_response_headers(bytes: Vec<u8>, status: StatusCode) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/ipp"),
    );
    (status, headers, bytes)
}

fn printer_attributes(state: &AppState, request: &IppRequest, status: Status) -> Vec<u8> {
    ResponseBuilder::new(request, status)
        .operation_string(ValueTag::Charset, "attributes-charset", "utf-8")
        .operation_string(
            ValueTag::NaturalLanguage,
            "attributes-natural-language",
            "en",
        )
        .printer_attributes()
        .string(
            ValueTag::Uri,
            "printer-uri-supported",
            &state.config.printer_uri,
        )
        .string(
            ValueTag::NameWithoutLanguage,
            "printer-name",
            &state.config.printer_name,
        )
        .string(
            ValueTag::TextWithoutLanguage,
            "printer-info",
            "Standalone Vevor label printer application",
        )
        .integer(ValueTag::Enum, "printer-state", 3)
        .string(ValueTag::Keyword, "printer-state-reasons", "none")
        .boolean("printer-is-accepting-jobs", true)
        .string(ValueTag::Keyword, "ipp-versions-supported", "2.0")
        .string(ValueTag::Keyword, "operations-supported", "Print-Job")
        .string(ValueTag::Keyword, "operations-supported", "Validate-Job")
        .string(
            ValueTag::Keyword,
            "operations-supported",
            "Get-Printer-Attributes",
        )
        .string(
            ValueTag::MimeMediaType,
            "document-format-supported",
            "image/pwg-raster",
        )
        .string(
            ValueTag::Keyword,
            "print-color-mode-supported",
            "monochrome",
        )
        .string(ValueTag::Keyword, "printer-resolution-supported", "300dpi")
        .finish()
}

async fn print_job(state: &AppState, request: &IppRequest) -> Result<(), PrintError> {
    if request.document.is_empty() {
        return Err(PrintError::UnsupportedFormat);
    }

    // Temporary development bridge: treat the document body as one already-packed
    // monochrome raster stripe. PWG Raster parsing is the next implementation step.
    let page = RasterPage {
        width_px: 8,
        height_px: request.document.len() as u32,
        bytes_per_line: 1,
        data: request.document.clone(),
    };
    let job = PrintJob {
        pages: vec![page],
        options: LabelOptions::default(),
    };
    let bytes = vevor::render(&job).map_err(PrintError::Internal)?;

    output::write_all(&state.config.output_device, &bytes)
        .await
        .map_err(PrintError::Internal)
}

enum PrintError {
    UnsupportedFormat,
    Internal(anyhow::Error),
}
