use super::pdf_object::PdfObjectTrait;

/// The PDF null singleton. Use `PDF_NULL` constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfNull;

/// The singleton PDF null value.
pub const PDF_NULL: PdfNull = PdfNull;

impl std::fmt::Display for PdfNull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}

impl PdfObjectTrait for PdfNull {
    fn size_in_bytes(&self) -> usize {
        4
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        buffer[offset] = b'n';
        buffer[offset + 1] = b'u';
        buffer[offset + 2] = b'l';
        buffer[offset + 3] = b'l';
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_converted_to_string() {
        assert_eq!(PDF_NULL.to_string(), "null");
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(PDF_NULL.size_in_bytes(), 4);
    }

    #[test]
    fn can_be_serialized() {
        let mut buffer = vec![b' '; 8];
        assert_eq!(PDF_NULL.copy_bytes_into(&mut buffer, 3), 4);
        assert_eq!(buffer, typed_array_for("   null "));
    }
}
