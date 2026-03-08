use std::fmt;
use crate::core::syntax::is_regular_char;
use crate::utils::{copy_string_into_buffer, to_hex_string};
use super::pdf_object::PdfObjectTrait;

/// Check if a byte is an uppercase hex digit (0-9, A-F).
fn is_uppercase_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'A'..=b'F').contains(&b)
}

/// Decode hex codes in a PDF name (e.g., "#20" → " ").
/// Only decodes `#XX` where XX are uppercase hex digits [0-9A-F],
/// matching pdf-lib's regex: `/#([\dABCDEF]{2})/g`.
fn decode_name(name: &str) -> String {
    let mut result = String::new();
    let bytes = name.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#'
            && i + 2 < bytes.len()
            && is_uppercase_hex(bytes[i + 1])
            && is_uppercase_hex(bytes[i + 2])
        {
            let hex = &name[i + 1..i + 3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// A PDF Name object (e.g., /Type, /Page).
///
/// Names are interned — calling `PdfName::of("Foo")` twice returns equal values.
/// Hex codes in names (like `#20` for space) are decoded on construction.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PdfName {
    /// The encoded form including the leading slash, e.g., "/Foo#20Bar"
    encoded_name: String,
}

impl PdfName {
    /// Create a PdfName from a raw name string (without leading slash).
    /// Hex codes like `#20` are decoded.
    pub fn of(name: &str) -> Self {
        let decoded_value = decode_name(name);

        let mut encoded_name = String::from("/");
        for ch in decoded_value.chars() {
            let code = ch as u8;
            if is_regular_char(code) {
                encoded_name.push(ch);
            } else {
                encoded_name.push('#');
                encoded_name.push_str(&to_hex_string(code));
            }
        }

        PdfName { encoded_name }
    }

    /// Get the encoded name string (with leading slash).
    pub fn as_string(&self) -> &str {
        &self.encoded_name
    }

    /// Decode the name to raw bytes (without the leading slash).
    pub fn as_bytes_decoded(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let chars: Vec<char> = self.encoded_name[1..].chars().collect(); // skip leading /
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '#' && i + 2 < chars.len() {
                let hex: String = [chars[i + 1], chars[i + 2]].iter().collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    bytes.push(byte);
                    i += 3;
                    continue;
                }
            }
            bytes.push(chars[i] as u8);
            i += 1;
        }
        bytes
    }

    /// Decode the name to a text string.
    pub fn decode_text(&self) -> String {
        let bytes = self.as_bytes_decoded();
        String::from_utf8_lossy(&bytes).to_string()
    }

    // Common PDF name constants
    pub fn length() -> Self { Self::of("Length") }
    pub fn flate_decode() -> Self { Self::of("FlateDecode") }
    pub fn resources() -> Self { Self::of("Resources") }
    pub fn font() -> Self { Self::of("Font") }
    pub fn x_object() -> Self { Self::of("XObject") }
    pub fn contents() -> Self { Self::of("Contents") }
    pub fn r#type() -> Self { Self::of("Type") }
    pub fn parent() -> Self { Self::of("Parent") }
    pub fn media_box() -> Self { Self::of("MediaBox") }
    pub fn page() -> Self { Self::of("Page") }
    pub fn annots() -> Self { Self::of("Annots") }
    pub fn rotate() -> Self { Self::of("Rotate") }
    pub fn title() -> Self { Self::of("Title") }
    pub fn author() -> Self { Self::of("Author") }
    pub fn subject() -> Self { Self::of("Subject") }
    pub fn creator() -> Self { Self::of("Creator") }
    pub fn keywords() -> Self { Self::of("Keywords") }
    pub fn producer() -> Self { Self::of("Producer") }
    pub fn creation_date() -> Self { Self::of("CreationDate") }
    pub fn mod_date() -> Self { Self::of("ModDate") }
}

impl fmt::Display for PdfName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encoded_name)
    }
}

impl fmt::Debug for PdfName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PdfName({})", self.encoded_name)
    }
}

impl PdfObjectTrait for PdfName {
    fn size_in_bytes(&self) -> usize {
        self.encoded_name.len()
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        copy_string_into_buffer(&self.encoded_name, buffer, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_constructed() {
        let _ = PdfName::of("foobar");
        let _ = PdfName::of("A;Name_With-***Characters?");
        let _ = PdfName::of("paired#28#29parentheses");
    }

    #[test]
    fn returns_equal_value_for_same_input() {
        assert_eq!(PdfName::of("foobar"), PdfName::of("foobar"));
        assert_eq!(
            PdfName::of("A;Name_With-***Characters?"),
            PdfName::of("A;Name_With-***Characters?")
        );
        assert_eq!(
            PdfName::of("paired#28#29parentheses"),
            PdfName::of("paired#28#29parentheses")
        );
    }

    #[test]
    fn decodes_hex_codes_in_values() {
        assert_eq!(PdfName::of("Lime#20Green"), PdfName::of("Lime Green"));
        assert_eq!(
            PdfName::of("paired#28#29parentheses"),
            PdfName::of("paired()parentheses")
        );
        assert_eq!(
            PdfName::of("The_Key_of_F#23_Minor"),
            PdfName::of("The_Key_of_F#_Minor")
        );
        assert_eq!(PdfName::of("A#42"), PdfName::of("AB"));
        assert_eq!(PdfName::of("Identity#2DH"), PdfName::of("Identity-H"));

        assert_eq!(PdfName::of("#40"), PdfName::of("@"));
        assert_eq!(PdfName::of("#41"), PdfName::of("A"));
        assert_eq!(PdfName::of("#42"), PdfName::of("B"));
        assert_eq!(PdfName::of("#43"), PdfName::of("C"));
        assert_eq!(PdfName::of("#44"), PdfName::of("D"));
        assert_eq!(PdfName::of("#45"), PdfName::of("E"));
        assert_eq!(PdfName::of("#46"), PdfName::of("F"));
        assert_eq!(PdfName::of("#47"), PdfName::of("G"));
        assert_eq!(PdfName::of("#48"), PdfName::of("H"));
        assert_eq!(PdfName::of("#49"), PdfName::of("I"));
        assert_eq!(PdfName::of("#4A"), PdfName::of("J"));
        assert_eq!(PdfName::of("#4B"), PdfName::of("K"));
        assert_eq!(PdfName::of("#4C"), PdfName::of("L"));
        assert_eq!(PdfName::of("#4D"), PdfName::of("M"));
        assert_eq!(PdfName::of("#4E"), PdfName::of("N"));
        assert_eq!(PdfName::of("#4F"), PdfName::of("O"));
    }

    #[test]
    fn encodes_hashes_whitespace_and_delimiters_when_serialized() {
        assert_eq!(PdfName::of("Foo#").to_string(), "/Foo#23");

        assert_eq!(PdfName::of("Foo\0").to_string(), "/Foo#00");
        assert_eq!(PdfName::of("Foo\t").to_string(), "/Foo#09");
        assert_eq!(PdfName::of("Foo\n").to_string(), "/Foo#0A");
        assert_eq!(PdfName::of("Foo\x0C").to_string(), "/Foo#0C");
        assert_eq!(PdfName::of("Foo\r").to_string(), "/Foo#0D");
        assert_eq!(PdfName::of("Foo ").to_string(), "/Foo#20");

        assert_eq!(PdfName::of("Foo(").to_string(), "/Foo#28");
        assert_eq!(PdfName::of("Foo)").to_string(), "/Foo#29");
        assert_eq!(PdfName::of("Foo<").to_string(), "/Foo#3C");
        assert_eq!(PdfName::of("Foo>").to_string(), "/Foo#3E");
        assert_eq!(PdfName::of("Foo[").to_string(), "/Foo#5B");
        assert_eq!(PdfName::of("Foo]").to_string(), "/Foo#5D");
        assert_eq!(PdfName::of("Foo{").to_string(), "/Foo#7B");
        assert_eq!(PdfName::of("Foo}").to_string(), "/Foo#7D");
        assert_eq!(PdfName::of("Foo/").to_string(), "/Foo#2F");
        assert_eq!(PdfName::of("Foo%").to_string(), "/Foo#25");
    }

    #[test]
    fn can_be_converted_to_string() {
        assert_eq!(PdfName::of("foobar").to_string(), "/foobar");
        assert_eq!(PdfName::of("Lime Green").to_string(), "/Lime#20Green");
        assert_eq!(
            PdfName::of("\0\t\n\x0C\r ").to_string(),
            "/#00#09#0A#0C#0D#20"
        );
        assert_eq!(PdfName::of("Foo#Bar").to_string(), "/Foo#23Bar");
        assert_eq!(
            PdfName::of("paired()parentheses").to_string(),
            "/paired#28#29parentheses"
        );
        // "The_Key_of_F#23_Minor" → decoded to "The_Key_of_F#_Minor" → re-encoded
        assert_eq!(
            PdfName::of("The_Key_of_F#23_Minor").to_string(),
            "/The_Key_of_F#23_Minor"
        );
        assert_eq!(PdfName::of("A#42").to_string(), "/AB");
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(PdfName::of("foobar").size_in_bytes(), 7);
        assert_eq!(PdfName::of("Lime Green").size_in_bytes(), 13);
        assert_eq!(PdfName::of("\0\t\n\x0C\r ").size_in_bytes(), 19);
        assert_eq!(PdfName::of("Foo#Bar").size_in_bytes(), 10);
        assert_eq!(PdfName::of("paired()parentheses").size_in_bytes(), 24);
        assert_eq!(PdfName::of("The_Key_of_F#23_Minor").size_in_bytes(), 22);
        assert_eq!(PdfName::of("A#42").size_in_bytes(), 3);
    }

    #[test]
    fn can_be_serialized() {
        let mut buffer1 = vec![b' '; 23];
        let written = PdfName::of("\0\t\n\x0C\r ").copy_bytes_into(&mut buffer1, 3);
        assert_eq!(written, 19);
        assert_eq!(buffer1, typed_array_for("   /#00#09#0A#0C#0D#20 "));

        let mut buffer2 = vec![b' '; 17];
        let written = PdfName::of("Lime Green").copy_bytes_into(&mut buffer2, 1);
        assert_eq!(written, 13);
        assert_eq!(buffer2, typed_array_for(" /Lime#20Green   "));

        let mut buffer3 = vec![b' '; 7];
        let written = PdfName::of("A#42").copy_bytes_into(&mut buffer3, 4);
        assert_eq!(written, 3);
        assert_eq!(buffer3, typed_array_for("    /AB"));
    }
}
