use std::fmt;
use crate::utils::copy_string_into_buffer;
use super::pdf_object::PdfObjectTrait;

/// A PDF indirect reference (e.g., "5 0 R").
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PdfRef {
    pub object_number: u32,
    pub generation_number: u16,
    tag: String,
}

impl PdfRef {
    pub fn of(object_number: u32, generation_number: u16) -> Self {
        let tag = format!("{} {} R", object_number, generation_number);
        PdfRef {
            object_number,
            generation_number,
            tag,
        }
    }

    /// Shorthand for generation 0 ref.
    pub fn of_num(object_number: u32) -> Self {
        Self::of(object_number, 0)
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }
}

impl fmt::Display for PdfRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

impl fmt::Debug for PdfRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PdfRef({})", self.tag)
    }
}

impl PdfObjectTrait for PdfRef {
    fn size_in_bytes(&self) -> usize {
        self.tag.len()
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        copy_string_into_buffer(&self.tag, buffer, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    #[test]
    fn can_be_constructed() {
        let _ = PdfRef::of(0, 0);
        let _ = PdfRef::of(0, 21);
        let _ = PdfRef::of(94, 0);
        let _ = PdfRef::of(4678, 9120);
    }

    #[test]
    fn returns_same_value_for_same_numbers() {
        assert_eq!(PdfRef::of(0, 0), PdfRef::of(0, 0));
        assert_eq!(PdfRef::of(0, 21), PdfRef::of(0, 21));
        assert_eq!(PdfRef::of(94, 0), PdfRef::of(94, 0));
        assert_eq!(PdfRef::of(4678, 9120), PdfRef::of(4678, 9120));
    }

    #[test]
    fn can_be_converted_to_string() {
        assert_eq!(PdfRef::of(0, 0).to_string(), "0 0 R");
        assert_eq!(PdfRef::of(0, 21).to_string(), "0 21 R");
        assert_eq!(PdfRef::of(94, 0).to_string(), "94 0 R");
        assert_eq!(PdfRef::of(4678, 9120).to_string(), "4678 9120 R");
    }

    #[test]
    fn can_provide_size_in_bytes() {
        assert_eq!(PdfRef::of(0, 0).size_in_bytes(), 5);
        assert_eq!(PdfRef::of(0, 21).size_in_bytes(), 6);
        assert_eq!(PdfRef::of(94, 0).size_in_bytes(), 6);
        assert_eq!(PdfRef::of(4678, 9120).size_in_bytes(), 11);
    }

    #[test]
    fn can_be_serialized() {
        let mut buffer1 = vec![b' '; 9];
        assert_eq!(PdfRef::of(0, 0).copy_bytes_into(&mut buffer1, 3), 5);
        assert_eq!(buffer1, typed_array_for("   0 0 R "));

        let mut buffer2 = vec![b' '; 9];
        assert_eq!(PdfRef::of(0, 21).copy_bytes_into(&mut buffer2, 1), 6);
        assert_eq!(buffer2, typed_array_for(" 0 21 R  "));

        let mut buffer3 = vec![b' '; 9];
        assert_eq!(PdfRef::of(94, 0).copy_bytes_into(&mut buffer3, 2), 6);
        assert_eq!(buffer3, typed_array_for("  94 0 R "));

        let mut buffer4 = vec![b' '; 13];
        assert_eq!(PdfRef::of(4678, 9120).copy_bytes_into(&mut buffer4, 0), 11);
        assert_eq!(buffer4, typed_array_for("4678 9120 R  "));
    }
}
