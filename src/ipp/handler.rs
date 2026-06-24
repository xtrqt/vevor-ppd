use super::codec::{parse_request, ResponseBuilder};
use super::model::{IppRequest, Operation, Status, ValueTag};
use crate::app::AppState;
use crate::driver::{vevor, LabelOptions, PrintJob, RasterPage};
use crate::output;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use std::sync::Arc;
use tracing::{error, info, warn};

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

    info!(
        operation = ?request.operation,
        request_id = request.request_id,
        document_format = request.document_format().unwrap_or("unknown"),
        document_bytes = request.document.len(),
        "received IPP request"
    );

    let response = match request.operation {
        Operation::GetPrinterAttributes => {
            printer_attributes(&state, &request, Status::SuccessfulOk)
        }
        Operation::ValidateJob => printer_attributes(&state, &request, Status::SuccessfulOk),
        Operation::GetJobs => printer_attributes(&state, &request, Status::SuccessfulOk),
        Operation::PrintJob => match request.document_format().as_deref() {
            Some("application/pdf") => printer_attributes(
                &state,
                &request,
                Status::ClientErrorDocumentFormatNotSupported,
            ),
            _ => match print_job(&state, &request).await {
                Ok(()) => job_attributes(&state, &request, Status::SuccessfulOk, 9),
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
        },
        Operation::CreateJob => job_attributes(&state, &request, Status::SuccessfulOk, 3),
        Operation::SendDocument => match send_document(&state, &request).await {
            Ok(()) => job_attributes(&state, &request, Status::SuccessfulOk, 9),
            Err(PrintError::UnsupportedFormat) => {
                job_attributes(&state, &request, Status::SuccessfulOk, 3)
            }
            Err(PrintError::Internal(err)) => {
                error!(error = %err, "send-document failed");
                job_attributes(&state, &request, Status::ServerErrorInternalError, 9)
            }
        },
        Operation::GetJobAttributes => job_attributes(&state, &request, Status::SuccessfulOk, 9),
        Operation::CancelJob => job_attributes(&state, &request, Status::SuccessfulOk, 7),
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
    let admin_url = format!(
        "http://{}:{}/",
        state.config.printer_host,
        state.config.listen_addr.port()
    );

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
        .string(ValueTag::Keyword, "uri-authentication-supported", "none")
        .string(ValueTag::Keyword, "uri-security-supported", "none")
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
        .string(
            ValueTag::TextWithoutLanguage,
            "printer-make-and-model",
            "Vevor Label Printer 300",
        )
        .string(ValueTag::Uri, "printer-more-info", &admin_url)
        .string(ValueTag::Charset, "charset-configured", "utf-8")
        .string(ValueTag::Charset, "charset-supported", "utf-8")
        .string(
            ValueTag::NaturalLanguage,
            "natural-language-configured",
            "en",
        )
        .string(
            ValueTag::NaturalLanguage,
            "generated-natural-language-supported",
            "en",
        )
        .integer(ValueTag::Enum, "printer-state", 3)
        .string(ValueTag::Keyword, "printer-state-reasons", "none")
        .boolean("printer-is-accepting-jobs", true)
        .integer(ValueTag::Integer, "queued-job-count", 0)
        .strings(ValueTag::Keyword, "ipp-versions-supported", &["2.0", "2.1"])
        .integers(
            ValueTag::Enum,
            "operations-supported",
            &[
                Operation::PrintJob.code(),
                Operation::ValidateJob.code(),
                Operation::CreateJob.code(),
                Operation::SendDocument.code(),
                Operation::CancelJob.code(),
                Operation::GetJobAttributes.code(),
                Operation::GetJobs.code(),
                Operation::GetPrinterAttributes.code(),
            ],
        )
        .strings(
            ValueTag::MimeMediaType,
            "document-format-supported",
            &["image/pwg-raster", "image/urf", "application/pdf"],
        )
        .string(
            ValueTag::MimeMediaType,
            "document-format-default",
            "image/pwg-raster",
        )
        .string(ValueTag::Keyword, "pdl-override-supported", "not-attempted")
        .string(ValueTag::Keyword, "compression-supported", "none")
        .string(
            ValueTag::Keyword,
            "print-color-mode-supported",
            "monochrome",
        )
        .string(ValueTag::Keyword, "print-color-mode-default", "monochrome")
        .string(ValueTag::Keyword, "sides-supported", "one-sided")
        .string(ValueTag::Keyword, "sides-default", "one-sided")
        .strings(
            ValueTag::Keyword,
            "media-supported",
            &[
                "oe_w288h432_4x6in",
                "oe_w288h288_4x4in",
                "oe_w144h288_2x4in",
            ],
        )
        .string(ValueTag::Keyword, "media-default", "oe_w288h432_4x6in")
        .resolution("printer-resolution-supported", 300, 300)
        .resolution("printer-resolution-default", 300, 300)
        .finish()
}

fn job_attributes(
    state: &AppState,
    request: &IppRequest,
    status: Status,
    job_state: i32,
) -> Vec<u8> {
    let job_id = request.request_id.max(1) as i32;
    let job_uri = format!("{}/jobs/{}", state.config.printer_uri, job_id);

    ResponseBuilder::new(request, status)
        .operation_string(ValueTag::Charset, "attributes-charset", "utf-8")
        .operation_string(
            ValueTag::NaturalLanguage,
            "attributes-natural-language",
            "en",
        )
        .job_attributes()
        .integer(ValueTag::Integer, "job-id", job_id)
        .string(ValueTag::Uri, "job-uri", &job_uri)
        .integer(ValueTag::Enum, "job-state", job_state)
        .string(ValueTag::Keyword, "job-state-reasons", "none")
        .string(ValueTag::NameWithoutLanguage, "job-name", "Vevor print job")
        .finish()
}

async fn print_job(state: &AppState, request: &IppRequest) -> Result<(), PrintError> {
    if request.document.is_empty() {
        return Err(PrintError::UnsupportedFormat);
    }

    // Development bridge: until the PWG Raster parser lands, accept the document
    // body as packed 1-bit raster rows for low-level driver/output testing.
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

async fn send_document(state: &AppState, request: &IppRequest) -> Result<(), PrintError> {
    if request.document.is_empty() {
        return Err(PrintError::UnsupportedFormat);
    }

    print_job(state, request).await
}

enum PrintError {
    UnsupportedFormat,
    Internal(anyhow::Error),
}
