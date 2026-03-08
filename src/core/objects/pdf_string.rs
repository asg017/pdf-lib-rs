use std::fmt;
use crate::core::syntax::CharCodes;
use crate::utils::{
    copy_string_into_buffer, has_utf16_bom, pdf_doc_encoding_decode, utf16_decode,
};
use super::pdf_object::PdfObjectTrait;

/// A PDF literal string object, e.g., `(Hello World)`.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfString {
    value: String,
}

impl PdfString {
    pub fn of(value: &str) -> Self {
        PdfString {
            value: value.to_string(),
        }
    }

    /// Get the raw string value (without parentheses).
    pub fn as_string(&self) -> &str {
        &self.value
    }

    /// Convert the string to raw bytes, interpreting escape sequences.
    /// Characters are treated as Latin-1 code points (0..=255), not UTF-8.
    pub fn as_bytes_decoded(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let chars: Vec<u8> = self.value.chars().map(|c| c as u8).collect();
        let mut i = 0;
        let mut escaped = false;
        let mut octal = String::new();

        while i < chars.len() {
            let byte = chars[i];
            let next_byte = chars.get(i + 1).copied();

            if !escaped {
                if byte == CharCodes::BackSlash {
                    escaped = true;
                } else {
                    bytes.push(byte);
                }
            } else {
                match byte {
                    CharCodes::Newline | CharCodes::CarriageReturn => {
                        // Escaped line break - ignore
                        escaped = false;
                    }
                    b'n' => {
                        bytes.push(CharCodes::Newline);
                        escaped = false;
                    }
                    b'r' => {
                        bytes.push(CharCodes::CarriageReturn);
                        escaped = false;
                    }
                    b't' => {
                        bytes.push(CharCodes::Tab);
                        escaped = false;
                    }
                    b'b' => {
                        bytes.push(CharCodes::Backspace);
                        escaped = false;
                    }
                    b'f' => {
                        bytes.push(CharCodes::FormFeed);
                        escaped = false;
                    }
                    CharCodes::LeftParen => {
                        bytes.push(CharCodes::LeftParen);
                        escaped = false;
                    }
                    CharCodes::RightParen => {
                        bytes.push(CharCodes::RightParen);
                        escaped = false;
                    }
                    CharCodes::BackSlash => {
                        bytes.push(CharCodes::BackSlash);
                        escaped = false;
                    }
                    b'0'..=b'7' => {
                        octal.push(byte as char);
                        if octal.len() == 3
                            || !matches!(next_byte, Some(b'0'..=b'7'))
                        {
                            if let Ok(val) = u8::from_str_radix(&octal, 8) {
                                bytes.push(val);
                            }
                            octal.clear();
                            escaped = false;
                        }
                    }
                    _ => {
                        bytes.push(byte);
                        escaped = false;
                    }
                }
            }
            i += 1;
        }

        bytes
    }

    /// Decode the string to a text String, handling UTF-16 and PDFDocEncoding.
    pub fn decode_text(&self) -> String {
        let bytes = self.as_bytes_decoded();
        if has_utf16_bom(&bytes) {
            utf16_decode(&bytes)
        } else {
            pdf_doc_encoding_decode(&bytes)
        }
    }
}

impl fmt::Display for PdfString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.value)
    }
}

impl PdfObjectTrait for PdfString {
    fn size_in_bytes(&self) -> usize {
        self.value.len() + 2
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let mut off = offset;
        buffer[off] = CharCodes::LeftParen;
        off += 1;
        off += copy_string_into_buffer(&self.value, buffer, off);
        buffer[off] = CharCodes::RightParen;
        self.value.len() + 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_constructed() {
        let _ = PdfString::of("foobar");
        let _ = PdfString::of(" (foo(bar))");
        let _ = PdfString::of(")b\\a/z(");
    }

    #[test]
    fn can_be_converted_to_raw_string() {
        assert_eq!(PdfString::of("foobar").as_string(), "foobar");
    }

    #[test]
    fn can_be_cloned() {
        let original = PdfString::of(")b\\a/z(");
        let clone = original.clone();
        assert_eq!(clone.to_string(), original.to_string());
    }

    #[test]
    fn can_be_converted_to_string() {
        assert_eq!(PdfString::of("foobar").to_string(), "(foobar)");
    }

    #[test]
    fn does_not_escape_backslashes() {
        assert_eq!(
            PdfString::of("Foo\\Bar\\Qux").to_string(),
            "(Foo\\Bar\\Qux)"
        );
    }

    #[test]
    fn does_not_escape_nested_parenthesis() {
        assert_eq!(
            PdfString::of("(Foo((Bar))Qux)").to_string(),
            "((Foo((Bar))Qux))"
        );
    }

    #[test]
    fn can_interpret_escaped_octal_codes() {
        let literal =
            "\\376\\377\\000\\105\\000\\147\\000\\147\\000\\040\\330\\074\\337\\163";
        let bytes = PdfString::of(literal).as_bytes_decoded();
        assert_eq!(
            bytes,
            vec![
                0o376, 0o377, 0o000, 0o105, 0o000, 0o147, 0o000, 0o147, 0o000, 0o040,
                0o330, 0o074, 0o337, 0o163,
            ]
        );
    }

    #[test]
    fn can_interpret_eols_and_line_breaks() {
        let literal = "a\nb\rc\\\nd\\\re";
        let bytes = PdfString::of(literal).as_bytes_decoded();
        assert_eq!(
            bytes,
            vec![
                b'a', b'\n', b'b', b'\r', b'c', b'd', b'e',
            ]
        );
    }

    #[test]
    fn can_interpret_invalid_escapes() {
        let literal = "a\nb\rc\\xd\\;";
        let bytes = PdfString::of(literal).as_bytes_decoded();
        assert_eq!(
            bytes,
            vec![b'a', b'\n', b'b', b'\r', b'c', b'x', b'd', b';']
        );
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(PdfString::of("foobar").size_in_bytes(), 8);
        assert_eq!(PdfString::of(" (foo(bar))").size_in_bytes(), 13);
        assert_eq!(PdfString::of(")b\\a/z(").size_in_bytes(), 9);
    }

    #[test]
    fn can_be_serialized() {
        let mut buffer = vec![b' '; 20];
        assert_eq!(
            PdfString::of(")(b\\a/))z(").copy_bytes_into(&mut buffer, 3),
            12
        );
        assert_eq!(buffer, typed_array_for("   ()(b\\a/))z()     "));
    }

    #[test]
    fn can_decode_utf16be_strings() {
        let literal =
            "\\376\\377\\000\\105\\000\\147\\000\\147\\000\\040\\330\\074\\337\\163";
        let text = PdfString::of(literal).decode_text();
        assert_eq!(text, "Egg 🍳");
    }

    #[test]
    fn can_decode_pdfdocencoded_strings() {
        let literal = "a\\105b\\163\\0b6";
        let text = PdfString::of(literal).decode_text();
        assert_eq!(text, "aEbs\0b6");
    }
}
