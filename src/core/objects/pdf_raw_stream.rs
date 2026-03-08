use std::fmt;
use crate::core::syntax::CharCodes;
use crate::utils::array_as_string;
use super::pdf_dict::PdfDict;
use super::pdf_name::PdfName;
use super::pdf_number::PdfNumber;
use super::pdf_object::{PdfObject, PdfObjectTrait};

/// A PDF stream with raw byte contents.
#[derive(Debug, Clone)]
pub struct PdfRawStream {
    pub dict: PdfDict,
    pub contents: Vec<u8>,
}

impl PdfRawStream {
    pub fn of(dict: PdfDict, contents: Vec<u8>) -> Self {
        PdfRawStream { dict, contents }
    }

    pub fn as_uint8_array(&self) -> Vec<u8> {
        self.contents.clone()
    }

    pub fn get_contents_string(&self) -> String {
        array_as_string(&self.contents)
    }

    #[allow(dead_code)]
    fn update_dict(&mut self) {
        self.dict.set(
            PdfName::length(),
            PdfObject::Number(PdfNumber::of(self.contents.len() as f64)),
        );
    }
}

impl PartialEq for PdfRawStream {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl fmt::Display for PdfRawStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\nstream\n{}\nendstream",
            self.dict,
            self.get_contents_string()
        )
    }
}

impl PdfObjectTrait for PdfRawStream {
    fn size_in_bytes(&self) -> usize {
        // dict + "\nstream\n" + contents + "\nendstream"
        self.dict.size_in_bytes() + self.contents.len() + 18
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let initial_offset = offset;
        let mut off = offset;

        // Write dict
        off += self.dict.copy_bytes_into(buffer, off);

        // \nstream\n
        buffer[off] = CharCodes::Newline;
        off += 1;
        for &b in b"stream" {
            buffer[off] = b;
            off += 1;
        }
        buffer[off] = CharCodes::Newline;
        off += 1;

        // Contents
        buffer[off..off + self.contents.len()].copy_from_slice(&self.contents);
        off += self.contents.len();

        // \nendstream
        buffer[off] = CharCodes::Newline;
        off += 1;
        for &b in b"endstream" {
            buffer[off] = b;
            off += 1;
        }

        off - initial_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_be_constructed() {
        let dict = PdfDict::new();
        let contents = vec![1, 2, 3, 4, 5];
        let stream = PdfRawStream::of(dict, contents.clone());
        assert_eq!(stream.contents, contents);
    }

    #[test]
    fn can_get_contents() {
        let dict = PdfDict::new();
        let stream = PdfRawStream::of(dict, b"hello".to_vec());
        assert_eq!(stream.get_contents_string(), "hello");
        assert_eq!(stream.as_uint8_array(), b"hello".to_vec());
    }

    #[test]
    fn can_be_serialized() {
        let dict = PdfDict::new();
        let stream = PdfRawStream::of(dict, b"data".to_vec());
        let size = stream.size_in_bytes();
        let mut buffer = vec![0u8; size];
        stream.copy_bytes_into(&mut buffer, 0);
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("stream\n"));
        assert!(result.contains("data"));
        assert!(result.contains("endstream"));
    }
}
