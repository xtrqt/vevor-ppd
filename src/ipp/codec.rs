use super::model::{IppRequest, Operation, Status, ValueTag};
use anyhow::{bail, ensure, Result};

const TAG_OPERATION_ATTRIBUTES: u8 = 0x01;
const TAG_PRINTER_ATTRIBUTES: u8 = 0x04;
const TAG_END: u8 = 0x03;

pub fn parse_request(bytes: &[u8]) -> Result<IppRequest> {
    ensure!(bytes.len() >= 8, "IPP request is too short");

    let version_major = bytes[0];
    let version_minor = bytes[1];
    let operation_id = u16::from_be_bytes([bytes[2], bytes[3]]);
    let request_id = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

    let mut cursor = 8;
    while cursor < bytes.len() {
        if bytes[cursor] == TAG_END {
            cursor += 1;
            break;
        }

        let tag = bytes[cursor];
        cursor += 1;
        if matches!(tag, TAG_OPERATION_ATTRIBUTES | TAG_PRINTER_ATTRIBUTES) {
            continue;
        }

        ensure!(
            cursor + 2 <= bytes.len(),
            "missing IPP attribute name length"
        );
        let name_len = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        ensure!(
            cursor + name_len <= bytes.len(),
            "truncated IPP attribute name"
        );
        cursor += name_len;

        ensure!(
            cursor + 2 <= bytes.len(),
            "missing IPP attribute value length"
        );
        let value_len = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        ensure!(
            cursor + value_len <= bytes.len(),
            "truncated IPP attribute value"
        );
        cursor += value_len;
    }

    if cursor > bytes.len() {
        bail!("invalid IPP request");
    }

    Ok(IppRequest {
        version_major,
        version_minor,
        operation: Operation::from(operation_id),
        request_id,
        document: bytes[cursor..].to_vec(),
    })
}

pub struct ResponseBuilder {
    bytes: Vec<u8>,
}

impl ResponseBuilder {
    pub fn new(request: &IppRequest, status: Status) -> Self {
        let mut bytes = Vec::new();
        bytes.push(request.version_major);
        bytes.push(request.version_minor);
        bytes.extend_from_slice(&(status as u16).to_be_bytes());
        bytes.extend_from_slice(&request.request_id.to_be_bytes());
        bytes.push(TAG_OPERATION_ATTRIBUTES);
        Self { bytes }
    }

    pub fn operation_string(mut self, tag: ValueTag, name: &str, value: &str) -> Self {
        self.attr(tag, name, value.as_bytes());
        self
    }

    pub fn printer_attributes(mut self) -> Self {
        self.bytes.push(TAG_PRINTER_ATTRIBUTES);
        self
    }

    pub fn string(mut self, tag: ValueTag, name: &str, value: &str) -> Self {
        self.attr(tag, name, value.as_bytes());
        self
    }

    pub fn strings(mut self, tag: ValueTag, name: &str, values: &[&str]) -> Self {
        for (index, value) in values.iter().enumerate() {
            self.attr(tag, repeated_name(name, index), value.as_bytes());
        }
        self
    }

    pub fn integer(mut self, tag: ValueTag, name: &str, value: i32) -> Self {
        self.attr(tag, name, &value.to_be_bytes());
        self
    }

    pub fn integers(mut self, tag: ValueTag, name: &str, values: &[i32]) -> Self {
        for (index, value) in values.iter().enumerate() {
            self.attr(tag, repeated_name(name, index), &value.to_be_bytes());
        }
        self
    }

    pub fn resolution(mut self, name: &str, cross_feed: i32, feed: i32) -> Self {
        let mut value = Vec::with_capacity(9);
        value.extend_from_slice(&cross_feed.to_be_bytes());
        value.extend_from_slice(&feed.to_be_bytes());
        value.push(3); // dots per inch
        self.attr(ValueTag::Resolution, name, &value);
        self
    }

    pub fn boolean(mut self, name: &str, value: bool) -> Self {
        self.attr(ValueTag::Boolean, name, &[u8::from(value)]);
        self
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.bytes.push(TAG_END);
        self.bytes
    }

    fn attr(&mut self, tag: ValueTag, name: &str, value: &[u8]) {
        self.bytes.push(tag as u8);
        self.bytes
            .extend_from_slice(&(name.len() as u16).to_be_bytes());
        self.bytes.extend_from_slice(name.as_bytes());
        self.bytes
            .extend_from_slice(&(value.len() as u16).to_be_bytes());
        self.bytes.extend_from_slice(value);
    }
}

fn repeated_name(name: &str, index: usize) -> &str {
    if index == 0 {
        name
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_request_with_document_tail() {
        let mut bytes = vec![0x02, 0x00, 0x00, 0x02, 0, 0, 0, 7, TAG_END];
        bytes.extend_from_slice(b"doc");

        let request = parse_request(&bytes).expect("parse request");
        assert_eq!(request.operation, Operation::PrintJob);
        assert_eq!(request.request_id, 7);
        assert_eq!(request.document, b"doc");
    }
}
