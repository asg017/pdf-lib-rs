use std::fmt;
use super::pdf_object::PdfObjectTrait;

/// A PDF boolean object (`true` or `false`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfBool {
    value: bool,
}

impl PdfBool {
    pub const TRUE: PdfBool = PdfBool { value: true };
    pub const FALSE: PdfBool = PdfBool { value: false };

    pub fn as_boolean(&self) -> bool {
        self.value
    }
}

impl fmt::Display for PdfBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PdfObjectTrait for PdfBool {
    fn size_in_bytes(&self) -> usize {
        if self.value { 4 } else { 5 }
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        if self.value {
            buffer[offset] = b't';
            buffer[offset + 1] = b'r';
            buffer[offset + 2] = b'u';
            buffer[offset + 3] = b'e';
            4
        } else {
            buffer[offset] = b'f';
            buffer[offset + 1] = b'a';
            buffer[offset + 2] = b'l';
            buffer[offset + 3] = b's';
            buffer[offset + 4] = b'e';
            5
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_converted_to_boolean() {
        assert!(PdfBool::TRUE.as_boolean());
        assert!(!PdfBool::FALSE.as_boolean());
    }

    #[test]
    fn can_be_cloned() {
        assert_eq!(PdfBool::TRUE.clone(), PdfBool::TRUE);
        assert_eq!(PdfBool::FALSE.clone(), PdfBool::FALSE);
    }

    #[test]
    fn can_be_converted_to_string() {
        assert_eq!(PdfBool::TRUE.to_string(), "true");
        assert_eq!(PdfBool::FALSE.to_string(), "false");
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(PdfBool::TRUE.size_in_bytes(), 4);
        assert_eq!(PdfBool::FALSE.size_in_bytes(), 5);
    }

    #[test]
    fn can_be_serialized_when_true() {
        let mut buffer = vec![b' '; 8];
        assert_eq!(PdfBool::TRUE.copy_bytes_into(&mut buffer, 3), 4);
        assert_eq!(buffer, typed_array_for("   true "));
    }

    #[test]
    fn can_be_serialized_when_false() {
        let mut buffer = vec![b' '; 9];
        assert_eq!(PdfBool::FALSE.copy_bytes_into(&mut buffer, 1), 5);
        assert_eq!(buffer, typed_array_for(" false   "));
    }
}
