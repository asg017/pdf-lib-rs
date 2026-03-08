use super::CharCodes;

/// Check if a byte is a PDF whitespace character.
pub fn is_whitespace(byte: u8) -> bool {
    matches!(
        byte,
        CharCodes::Null
            | CharCodes::Tab
            | CharCodes::Newline
            | CharCodes::FormFeed
            | CharCodes::CarriageReturn
            | CharCodes::Space
    )
}

/// Check if a byte is a PDF delimiter character.
pub fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        CharCodes::LeftParen
            | CharCodes::RightParen
            | CharCodes::LessThan
            | CharCodes::GreaterThan
            | CharCodes::LeftSquareBracket
            | CharCodes::RightSquareBracket
            | CharCodes::LeftCurly
            | CharCodes::RightCurly
            | CharCodes::ForwardSlash
            | CharCodes::Percent
    )
}

/// Check if a byte is an "irregular" character (whitespace, delimiter, or hash).
pub fn is_irregular(byte: u8) -> bool {
    is_whitespace(byte) || is_delimiter(byte) || byte == CharCodes::Hash
}

/// Check if a byte is a digit (0-9).
pub fn is_digit(byte: u8) -> bool {
    (CharCodes::Zero..=CharCodes::Nine).contains(&byte)
}

/// Check if a byte is a numeric prefix (+, -, .).
pub fn is_numeric_prefix(byte: u8) -> bool {
    matches!(byte, CharCodes::Period | CharCodes::Plus | CharCodes::Minus)
}

/// Check if a byte is numeric (digit or numeric prefix).
pub fn is_numeric(byte: u8) -> bool {
    is_digit(byte) || is_numeric_prefix(byte)
}

/// Check if a character code is a "regular" character for PDF names.
/// Regular characters are ! through ~ excluding irregular characters.
pub fn is_regular_char(code: u8) -> bool {
    (CharCodes::ExclamationPoint..=CharCodes::Tilde).contains(&code) && !is_irregular(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert!(is_whitespace(0));   // Null
        assert!(is_whitespace(9));   // Tab
        assert!(is_whitespace(10));  // Newline
        assert!(is_whitespace(12));  // FormFeed
        assert!(is_whitespace(13));  // CarriageReturn
        assert!(is_whitespace(32));  // Space
        assert!(!is_whitespace(65)); // 'A'
    }

    #[test]
    fn test_delimiter() {
        assert!(is_delimiter(b'('));
        assert!(is_delimiter(b')'));
        assert!(is_delimiter(b'<'));
        assert!(is_delimiter(b'>'));
        assert!(is_delimiter(b'['));
        assert!(is_delimiter(b']'));
        assert!(is_delimiter(b'{'));
        assert!(is_delimiter(b'}'));
        assert!(is_delimiter(b'/'));
        assert!(is_delimiter(b'%'));
        assert!(!is_delimiter(b'A'));
    }

    #[test]
    fn test_is_digit() {
        for d in b'0'..=b'9' {
            assert!(is_digit(d));
        }
        assert!(!is_digit(b'A'));
        assert!(!is_digit(b' '));
    }
}
