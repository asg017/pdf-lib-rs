use std::fmt;
use crate::core::objects::pdf_object::PdfObjectTrait;
use crate::core::syntax::CharCodes;
use crate::utils::copy_string_into_buffer;

/// PDF trailer — `startxref` offset and `%%EOF` marker.
#[derive(Debug, Clone)]
pub struct PdfTrailer {
    last_xref_offset: String,
}

impl PdfTrailer {
    pub fn for_last_cross_ref_section_offset(offset: u64) -> Self {
        PdfTrailer {
            last_xref_offset: offset.to_string(),
        }
    }
}

impl fmt::Display for PdfTrailer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "startxref\n{}\n%%EOF", self.last_xref_offset)
    }
}

impl PdfObjectTrait for PdfTrailer {
    fn size_in_bytes(&self) -> usize {
        // "startxref\n" (10) + offset + "\n%%EOF" (6) = 16 + offset.len
        16 + self.last_xref_offset.len()
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let initial_offset = offset;
        let mut off = offset;

        // "startxref\n"
        for &b in b"startxref" {
            buffer[off] = b;
            off += 1;
        }
        buffer[off] = CharCodes::Newline;
        off += 1;

        off += copy_string_into_buffer(&self.last_xref_offset, buffer, off);

        // "\n%%EOF"
        buffer[off] = CharCodes::Newline;
        off += 1;
        buffer[off] = CharCodes::Percent;
        off += 1;
        buffer[off] = CharCodes::Percent;
        off += 1;
        buffer[off] = CharCodes::UpperE;
        off += 1;
        buffer[off] = CharCodes::UpperO;
        off += 1;
        buffer[off] = CharCodes::UpperF;
        off += 1;

        off - initial_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_converted_to_string() {
        assert_eq!(
            PdfTrailer::for_last_cross_ref_section_offset(799).to_string(),
            "startxref\n799\n%%EOF"
        );
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(
            PdfTrailer::for_last_cross_ref_section_offset(1919).size_in_bytes(),
            20
        );
    }

    #[test]
    fn can_be_serialized() {
        let mut buffer = vec![b' '; 21];
        let trailer = PdfTrailer::for_last_cross_ref_section_offset(1);
        trailer.copy_bytes_into(&mut buffer, 3);
        assert_eq!(buffer, typed_array_for("   startxref\n1\n%%EOF "));
    }
}
