/// Convert a character to its char code (byte value).
pub fn to_char_code(c: char) -> u8 {
    c as u8
}

/// Convert a byte value to a two-digit uppercase hex string.
pub fn to_hex_string(num: u8) -> String {
    format!("{:02X}", num)
}

/// Convert a number to a hex string with minimum length, zero-padded.
pub fn to_hex_string_of_min_length(num: u16, min_length: usize) -> String {
    let hex = format!("{:X}", num);
    if hex.len() < min_length {
        let padding = "0".repeat(min_length - hex.len());
        format!("{}{}", padding, hex)
    } else {
        hex
    }
}

/// Convert a hex code string (e.g., "20") to the corresponding character.
pub fn char_from_hex_code(hex: &str) -> char {
    u8::from_str_radix(hex, 16).unwrap_or(0) as char
}

/// Copy a string's bytes into a buffer at the given offset. Returns bytes written.
pub fn copy_string_into_buffer(s: &str, buffer: &mut [u8], offset: usize) -> usize {
    let bytes = s.as_bytes();
    let len = bytes.len();
    buffer[offset..offset + len].copy_from_slice(bytes);
    len
}

/// Convert a number to its string representation without scientific notation.
/// This matches pdf-lib's `numberToString` which avoids exponential notation.
pub fn number_to_string(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e20 {
        // Integer-like values
        format!("{}", value as i64)
    } else if value.abs() >= 1e20 || (value != 0.0 && value.abs() < 1e-6) {
        // Very large or very small numbers - format without scientific notation
        format_no_exponent(value)
    } else {
        // Regular floating point
        let s = format!("{}", value);
        // Remove trailing zeros after decimal point but keep at least one digit
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0');
            let trimmed = trimmed.trim_end_matches('.');
            trimmed.to_string()
        } else {
            s
        }
    }
}

fn format_no_exponent(value: f64) -> String {
    // Use a large precision to avoid scientific notation
    let s = format!("{:.50}", value);
    // Trim trailing zeros
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0');
        let trimmed = trimmed.trim_end_matches('.');
        trimmed.to_string()
    } else {
        s
    }
}

/// Create a Vec<u8> from a string (each char's lower byte).
pub fn typed_array_for(s: &str) -> Vec<u8> {
    s.bytes().collect()
}

/// Convert a byte slice to a String (interpreting each byte as a char).
pub fn array_as_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

/// Merge multiple byte slices into one Vec<u8>.
pub fn merge_into_typed_array(parts: &[&[u8]]) -> Vec<u8> {
    let total_len: usize = parts.iter().map(|p| p.len()).sum();
    let mut result = Vec::with_capacity(total_len);
    for part in parts {
        result.extend_from_slice(part);
    }
    result
}

/// Check if bytes start with a UTF-16 BOM (big-endian or little-endian).
pub fn has_utf16_bom(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && ((bytes[0] == 0xFE && bytes[1] == 0xFF) || (bytes[0] == 0xFF && bytes[1] == 0xFE))
}

/// Decode UTF-16 bytes (with BOM) to a String.
pub fn utf16_decode(bytes: &[u8]) -> String {
    if bytes.len() < 2 {
        return String::new();
    }

    let big_endian = bytes[0] == 0xFE && bytes[1] == 0xFF;
    let data = &bytes[2..]; // skip BOM

    let mut code_units: Vec<u16> = Vec::with_capacity(data.len() / 2);
    let mut i = 0;
    while i + 1 < data.len() {
        let unit = if big_endian {
            ((data[i] as u16) << 8) | (data[i + 1] as u16)
        } else {
            ((data[i + 1] as u16) << 8) | (data[i] as u16)
        };
        code_units.push(unit);
        i += 2;
    }

    String::from_utf16_lossy(&code_units)
}

/// Encode a string as UTF-16BE with BOM.
pub fn utf16_encode(text: &str) -> Vec<u16> {
    let mut result = vec![0xFEFF]; // BOM
    for c in text.chars() {
        let mut buf = [0u16; 2];
        let encoded = c.encode_utf16(&mut buf);
        result.extend_from_slice(encoded);
    }
    result
}

/// Decode bytes using PDFDocEncoding to a String.
/// PDFDocEncoding is essentially Latin-1 for bytes 0x00-0xFF,
/// with some special mappings in the 0x80-0x9F range.
pub fn pdf_doc_encoding_decode(bytes: &[u8]) -> String {
    // For simplicity, use a direct byte-to-char mapping.
    // PDFDocEncoding maps 0x00-0x7F to Unicode directly,
    // and 0xA0-0xFF to Unicode directly (Latin-1 supplement).
    // The 0x80-0x9F range has special mappings, and some bytes
    // in 0x00-0x1F are undefined (map to replacement char).
    bytes.iter().map(|&b| {
        match b {
            // Standard ASCII range
            0x00..=0x7F => b as char,
            // 0x80-0x9F: Special PDFDocEncoding mappings
            0x80 => '\u{2022}', // BULLET
            0x81 => '\u{2020}', // DAGGER
            0x82 => '\u{2021}', // DOUBLE DAGGER
            0x83 => '\u{2026}', // HORIZONTAL ELLIPSIS
            0x84 => '\u{2014}', // EM DASH
            0x85 => '\u{2013}', // EN DASH
            0x86 => '\u{0192}', // LATIN SMALL F WITH HOOK
            0x87 => '\u{2044}', // FRACTION SLASH
            0x88 => '\u{2039}', // SINGLE LEFT-POINTING ANGLE QUOTATION MARK
            0x89 => '\u{203A}', // SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
            0x8A => '\u{2212}', // MINUS SIGN
            0x8B => '\u{2030}', // PER MILLE SIGN
            0x8C => '\u{201E}', // DOUBLE LOW-9 QUOTATION MARK
            0x8D => '\u{201C}', // LEFT DOUBLE QUOTATION MARK
            0x8E => '\u{201D}', // RIGHT DOUBLE QUOTATION MARK
            0x8F => '\u{2018}', // LEFT SINGLE QUOTATION MARK
            0x90 => '\u{2019}', // RIGHT SINGLE QUOTATION MARK
            0x91 => '\u{201A}', // SINGLE LOW-9 QUOTATION MARK
            0x92 => '\u{2122}', // TRADE MARK SIGN
            0x93 => '\u{FB01}', // LATIN SMALL LIGATURE FI
            0x94 => '\u{FB02}', // LATIN SMALL LIGATURE FL
            0x95 => '\u{0141}', // LATIN CAPITAL LETTER L WITH STROKE
            0x96 => '\u{0152}', // LATIN CAPITAL LIGATURE OE
            0x97 => '\u{0160}', // LATIN CAPITAL LETTER S WITH CARON
            0x98 => '\u{0178}', // LATIN CAPITAL LETTER Y WITH DIAERESIS
            0x99 => '\u{017D}', // LATIN CAPITAL LETTER Z WITH CARON
            0x9A => '\u{0131}', // LATIN SMALL LETTER DOTLESS I
            0x9B => '\u{0142}', // LATIN SMALL LETTER L WITH STROKE
            0x9C => '\u{0153}', // LATIN SMALL LIGATURE OE
            0x9D => '\u{0161}', // LATIN SMALL LETTER S WITH CARON
            0x9E => '\u{017E}', // LATIN SMALL LETTER Z WITH CARON
            0x9F => '\u{FFFD}', // REPLACEMENT CHARACTER (undefined)
            // 0xA0 is non-breaking space, mapped via Latin-1
            0xA0 => '\u{00A0}',
            0xA1 => '\u{00A1}',
            0xA2..=0xAC => b as char,
            0xAD => '\u{00AD}', // soft hyphen
            0xAE..=0xFF => b as char,
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_char_code() {
        assert_eq!(to_char_code('A'), 65);
        assert_eq!(to_char_code(' '), 32);
        assert_eq!(to_char_code('\n'), 10);
    }

    #[test]
    fn test_to_hex_string() {
        assert_eq!(to_hex_string(0), "00");
        assert_eq!(to_hex_string(255), "FF");
        assert_eq!(to_hex_string(16), "10");
        assert_eq!(to_hex_string(9), "09");
    }

    #[test]
    fn test_char_from_hex_code() {
        assert_eq!(char_from_hex_code("20"), ' ');
        assert_eq!(char_from_hex_code("41"), 'A');
        assert_eq!(char_from_hex_code("42"), 'B');
    }

    #[test]
    fn test_copy_string_into_buffer() {
        let mut buf = vec![b' '; 10];
        let written = copy_string_into_buffer("hello", &mut buf, 2);
        assert_eq!(written, 5);
        assert_eq!(&buf, b"  hello   ");
    }

    #[test]
    fn test_number_to_string_integers() {
        assert_eq!(number_to_string(21.0), "21");
        assert_eq!(number_to_string(-43.0), "-43");
        assert_eq!(number_to_string(0.0), "0");
    }

    #[test]
    fn test_typed_array_for() {
        assert_eq!(typed_array_for("ABC"), vec![65, 66, 67]);
        assert_eq!(typed_array_for("   "), vec![32, 32, 32]);
    }

    #[test]
    fn test_has_utf16_bom() {
        assert!(has_utf16_bom(&[0xFE, 0xFF, 0x00, 0x41])); // BE
        assert!(has_utf16_bom(&[0xFF, 0xFE, 0x41, 0x00])); // LE
        assert!(!has_utf16_bom(&[0x41, 0x42]));
        assert!(!has_utf16_bom(&[0xFE]));
    }

    #[test]
    fn test_utf16_decode_be() {
        // "Egg " in UTF-16BE with BOM
        let bytes = vec![0xFE, 0xFF, 0x00, 0x45, 0x00, 0x67, 0x00, 0x67, 0x00, 0x20];
        assert_eq!(utf16_decode(&bytes), "Egg ");
    }

    #[test]
    fn test_utf16_decode_le() {
        // "Egg " in UTF-16LE with BOM
        let bytes = vec![0xFF, 0xFE, 0x45, 0x00, 0x67, 0x00, 0x67, 0x00, 0x20, 0x00];
        assert_eq!(utf16_decode(&bytes), "Egg ");
    }

    #[test]
    fn test_utf16_encode() {
        let encoded = utf16_encode("");
        assert_eq!(encoded, vec![0xFEFF]); // just BOM

        let encoded = utf16_encode("A");
        assert_eq!(encoded, vec![0xFEFF, 0x0041]);
    }

    #[test]
    fn test_pdf_doc_encoding_decode_ascii() {
        let bytes = vec![0x61, 0x45, 0x62, 0x73]; // "aEbs"
        assert_eq!(pdf_doc_encoding_decode(&bytes), "aEbs");
    }
}
