use std::fmt;
use crate::core::syntax::CharCodes;
use crate::core::objects::pdf_object::PdfObjectTrait;
use crate::utils::copy_string_into_buffer;

/// PDF file header, e.g., `%PDF-1.7` followed by a binary comment.
#[derive(Debug, Clone)]
pub struct PdfHeader {
    major: String,
    minor: String,
}

impl PdfHeader {
    pub fn for_version(major: u8, minor: u8) -> Self {
        PdfHeader {
            major: major.to_string(),
            minor: minor.to_string(),
        }
    }

    pub fn major(&self) -> &str {
        &self.major
    }

    pub fn minor(&self) -> &str {
        &self.minor
    }
}

impl fmt::Display for PdfHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bc = 129u8 as char;
        write!(f, "%PDF-{}.{}\n%{}{}{}{}", self.major, self.minor, bc, bc, bc, bc)
    }
}

impl PdfObjectTrait for PdfHeader {
    fn size_in_bytes(&self) -> usize {
        // "%PDF-" (5) + major + "." (1) + minor + "\n%" (2) + 4 binary chars = 12 + major.len + minor.len
        12 + self.major.len() + self.minor.len()
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let initial_offset = offset;
        let mut off = offset;

        buffer[off] = CharCodes::Percent;
        off += 1;
        buffer[off] = CharCodes::UpperP;
        off += 1;
        buffer[off] = CharCodes::UpperD;
        off += 1;
        buffer[off] = CharCodes::UpperF;
        off += 1;
        buffer[off] = CharCodes::Minus;
        off += 1;

        off += copy_string_into_buffer(&self.major, buffer, off);
        buffer[off] = CharCodes::Period;
        off += 1;
        off += copy_string_into_buffer(&self.minor, buffer, off);
        buffer[off] = CharCodes::Newline;
        off += 1;

        buffer[off] = CharCodes::Percent;
        off += 1;
        buffer[off] = 129;
        off += 1;
        buffer[off] = 129;
        off += 1;
        buffer[off] = 129;
        off += 1;
        buffer[off] = 129;
        off += 1;

        off - initial_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_constructed() {
        let _ = PdfHeader::for_version(1, 2);
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(PdfHeader::for_version(81, 79).size_in_bytes(), 16);
    }

    #[test]
    fn can_be_serialized() {
        let mut buffer = vec![b' '; 20];
        assert_eq!(PdfHeader::for_version(79, 81).copy_bytes_into(&mut buffer, 3), 16);
        let mut expected = typed_array_for("   %PDF-79.81\n%");
        expected.push(129);
        expected.push(129);
        expected.push(129);
        expected.push(129);
        expected.push(b' ');
        assert_eq!(buffer, expected);
    }
}
