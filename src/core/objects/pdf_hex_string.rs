use std::fmt;
use crate::core::syntax::CharCodes;
use crate::utils::{
    copy_string_into_buffer, has_utf16_bom, pdf_doc_encoding_decode, to_hex_string_of_min_length,
    utf16_decode, utf16_encode,
};
use super::pdf_object::PdfObjectTrait;

/// A PDF hexadecimal string object, e.g., `<4E6F76>`.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfHexString {
    value: String,
}

impl PdfHexString {
    pub fn of(value: &str) -> Self {
        PdfHexString {
            value: value.to_string(),
        }
    }

    /// Create from text, encoding as UTF-16BE with BOM.
    pub fn from_text(text: &str) -> Self {
        let encoded = utf16_encode(text);
        let mut hex = String::new();
        for unit in &encoded {
            hex.push_str(&to_hex_string_of_min_length(*unit, 4));
        }
        PdfHexString { value: hex }
    }

    /// Get the raw hex string value.
    pub fn as_string(&self) -> &str {
        &self.value
    }

    /// Convert hex string to raw bytes.
    pub fn as_bytes_decoded(&self) -> Vec<u8> {
        // Append a zero if odd number of digits (PDF spec 7.3.4.3)
        let hex = if self.value.len() % 2 == 1 {
            format!("{}0", self.value)
        } else {
            self.value.clone()
        };

        let mut bytes = Vec::with_capacity(hex.len() / 2);
        let mut i = 0;
        while i + 1 < hex.len() {
            if let Ok(byte) = u8::from_str_radix(&hex[i..i + 2], 16) {
                bytes.push(byte);
            } else {
                bytes.push(0);
            }
            i += 2;
        }
        bytes
    }

    /// Decode hex string to text, handling UTF-16 and PDFDocEncoding.
    pub fn decode_text(&self) -> String {
        let bytes = self.as_bytes_decoded();
        if has_utf16_bom(&bytes) {
            utf16_decode(&bytes)
        } else {
            pdf_doc_encoding_decode(&bytes)
        }
    }
}

impl fmt::Display for PdfHexString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.value)
    }
}

impl PdfObjectTrait for PdfHexString {
    fn size_in_bytes(&self) -> usize {
        self.value.len() + 2
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let mut off = offset;
        buffer[off] = CharCodes::LessThan;
        off += 1;
        off += copy_string_into_buffer(&self.value, buffer, off);
        buffer[off] = CharCodes::GreaterThan;
        self.value.len() + 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_constructed() {
        let _ = PdfHexString::of("4E6F762073686D6F7A2");
        let _ = PdfHexString::of("901FA3");
        let _ = PdfHexString::of("901FA");
    }

    #[test]
    fn can_be_constructed_from_text() {
        assert_eq!(PdfHexString::from_text("").to_string(), "<FEFF>");
    }

    #[test]
    fn can_handle_even_hex_digits() {
        let hex = "FEFF0045006700670020D83CDF73";
        let bytes = PdfHexString::of(hex).as_bytes_decoded();
        assert_eq!(
            bytes,
            vec![0xFE, 0xFF, 0x00, 0x45, 0x00, 0x67, 0x00, 0x67, 0x00, 0x20, 0xD8, 0x3C, 0xDF, 0x73]
        );
    }

    #[test]
    fn can_handle_odd_hex_digits() {
        let hex = "6145627300623";
        let bytes = PdfHexString::of(hex).as_bytes_decoded();
        assert_eq!(bytes, vec![0x61, 0x45, 0x62, 0x73, 0x00, 0x62, 0x30]);
    }

    #[test]
    fn can_decode_utf16be_string() {
        let hex = "FEFF0045006700670020D83CDF73";
        assert_eq!(PdfHexString::of(hex).decode_text(), "Egg 🍳");
    }

    #[test]
    fn can_decode_utf16le_string() {
        let hex = "FFFE45006700670020003CD873DF";
        assert_eq!(PdfHexString::of(hex).decode_text(), "Egg 🍳");
    }

    #[test]
    fn can_decode_pdfdocencoded_string() {
        let hex = "61456273006236";
        assert_eq!(PdfHexString::of(hex).decode_text(), "aEbs\0b6");
    }

    #[test]
    fn can_get_raw_string() {
        assert_eq!(PdfHexString::of("901FA").as_string(), "901FA");
    }

    #[test]
    fn can_be_cloned() {
        let original = PdfHexString::of("901FA");
        let clone = original.clone();
        assert_eq!(clone.to_string(), original.to_string());
    }

    #[test]
    fn can_be_converted_to_string() {
        assert_eq!(
            PdfHexString::of("4E6F762073686D6F7A2").to_string(),
            "<4E6F762073686D6F7A2>"
        );
        assert_eq!(PdfHexString::of("901FA3").to_string(), "<901FA3>");
        assert_eq!(PdfHexString::of("901FA").to_string(), "<901FA>");
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(PdfHexString::of("4E6F762073686D6F7A2").size_in_bytes(), 21);
        assert_eq!(PdfHexString::of("901FA3").size_in_bytes(), 8);
        assert_eq!(PdfHexString::of("901FA").size_in_bytes(), 7);
    }

    #[test]
    fn can_be_serialized() {
        let mut buffer = vec![b' '; 11];
        assert_eq!(PdfHexString::of("901FA").copy_bytes_into(&mut buffer, 3), 7);
        assert_eq!(buffer, typed_array_for("   <901FA> "));
    }
}
