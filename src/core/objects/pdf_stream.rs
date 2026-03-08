use std::fmt;
use crate::core::syntax::CharCodes;
use super::pdf_dict::PdfDict;
use super::pdf_name::PdfName;
use super::pdf_number::PdfNumber;
use super::pdf_object::{PdfObject, PdfObjectTrait};

/// Base trait for PDF stream objects.
/// A stream consists of a dictionary followed by binary content.
#[allow(dead_code)]
pub trait PdfStreamTrait: PdfObjectTrait {
    fn dict(&self) -> &PdfDict;
    fn dict_mut(&mut self) -> &mut PdfDict;
    fn get_contents(&self) -> &[u8];
    fn get_contents_size(&self) -> usize {
        self.get_contents().len()
    }

    /// Update the /Length entry in the stream dictionary.
    fn update_dict(&mut self) {
        let size = self.get_contents_size();
        self.dict_mut().set(
            PdfName::length(),
            PdfObject::Number(PdfNumber::of(size as f64)),
        );
    }
}

/// Placeholder for PdfStream in the object hierarchy.
/// Concrete implementations use PdfRawStream.
#[derive(Debug, Clone)]
pub struct PdfStream {
    pub dict: PdfDict,
}

impl PdfStream {
    pub fn new(dict: PdfDict) -> Self {
        PdfStream { dict }
    }
}

impl PartialEq for PdfStream {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl fmt::Display for PdfStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\nstream\n...data...\nendstream", self.dict)
    }
}

impl PdfObjectTrait for PdfStream {
    fn size_in_bytes(&self) -> usize {
        self.dict.size_in_bytes() + 18 // dict + \nstream\n...\nendstream
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let initial_offset = offset;
        let mut off = offset;
        off += self.dict.copy_bytes_into(buffer, off);
        buffer[off] = CharCodes::Newline;
        off += 1;
        for &b in b"stream\n" {
            buffer[off] = b;
            off += 1;
        }
        for &b in b"\nendstream" {
            buffer[off] = b;
            off += 1;
        }
        off - initial_offset
    }
}
