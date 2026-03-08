use std::fmt;
use crate::utils::{copy_string_into_buffer, number_to_string};
use super::pdf_object::PdfObjectTrait;

/// A PDF numeric object (integer or real).
#[derive(Clone, PartialEq)]
pub struct PdfNumber {
    number_value: f64,
    string_value: String,
}

impl PdfNumber {
    pub fn of(value: f64) -> Self {
        let string_value = number_to_string(value);
        PdfNumber {
            number_value: value,
            string_value,
        }
    }

    pub fn as_number(&self) -> f64 {
        self.number_value
    }
}

impl fmt::Display for PdfNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string_value)
    }
}

impl fmt::Debug for PdfNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PdfNumber({})", self.string_value)
    }
}

impl PdfObjectTrait for PdfNumber {
    fn size_in_bytes(&self) -> usize {
        self.string_value.len()
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        copy_string_into_buffer(&self.string_value, buffer, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_constructed() {
        let _ = PdfNumber::of(21.0);
        let _ = PdfNumber::of(-43.0);
        let _ = PdfNumber::of(-0.1e7);
    }

    #[test]
    fn can_be_cloned() {
        let original = PdfNumber::of(-21.42);
        let clone = original.clone();
        assert_eq!(clone.to_string(), original.to_string());
    }

    #[test]
    fn can_be_converted_to_string() {
        assert_eq!(PdfNumber::of(21.0).to_string(), "21");
        assert_eq!(PdfNumber::of(-43.0).to_string(), "-43");
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(PdfNumber::of(21.0).size_in_bytes(), 2);
        assert_eq!(PdfNumber::of(-43.0).size_in_bytes(), 3);
    }

    #[test]
    fn can_be_serialized() {
        let mut buffer1 = vec![b' '; 8];
        assert_eq!(PdfNumber::of(21.0).copy_bytes_into(&mut buffer1, 3), 2);
        assert_eq!(buffer1, typed_array_for("   21   "));
    }
}
