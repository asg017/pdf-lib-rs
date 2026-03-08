use std::fmt;
use crate::core::objects::PdfDict;
use crate::core::objects::pdf_object::PdfObjectTrait;
use crate::core::syntax::CharCodes;

/// PDF trailer dictionary — `trailer` keyword followed by a dictionary.
#[derive(Debug, Clone)]
pub struct PdfTrailerDict {
    pub dict: PdfDict,
}

impl PdfTrailerDict {
    pub fn of(dict: PdfDict) -> Self {
        PdfTrailerDict { dict }
    }
}

impl fmt::Display for PdfTrailerDict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "trailer\n{}", self.dict)
    }
}

impl PdfObjectTrait for PdfTrailerDict {
    fn size_in_bytes(&self) -> usize {
        // "trailer\n" (8) + dict
        8 + self.dict.size_in_bytes()
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let initial_offset = offset;
        let mut off = offset;

        // "trailer\n"
        buffer[off] = CharCodes::LowerT;
        off += 1;
        buffer[off] = CharCodes::LowerR;
        off += 1;
        buffer[off] = CharCodes::LowerA;
        off += 1;
        buffer[off] = CharCodes::LowerI;
        off += 1;
        buffer[off] = CharCodes::LowerL;
        off += 1;
        buffer[off] = CharCodes::LowerE;
        off += 1;
        buffer[off] = CharCodes::LowerR;
        off += 1;
        buffer[off] = CharCodes::Newline;
        off += 1;

        off += self.dict.copy_bytes_into(buffer, off);

        off - initial_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::objects::{PdfName, PdfObject};
    use crate::utils::typed_array_for;

    fn make_test_dict() -> PdfDict {
        let mut dict = PdfDict::new();
        dict.set(PdfName::of("Foo"), PdfObject::Name(PdfName::of("Bar")));
        dict
    }

    #[test]
    fn can_be_converted_to_string() {
        let td = PdfTrailerDict::of(make_test_dict());
        assert_eq!(td.to_string(), "trailer\n<<\n/Foo /Bar\n>>");
    }

    #[test]
    fn can_provide_size_in_bytes() {
        let td = PdfTrailerDict::of(make_test_dict());
        assert_eq!(td.size_in_bytes(), 23);
    }

    #[test]
    fn can_be_serialized() {
        let td = PdfTrailerDict::of(make_test_dict());
        let mut buffer = vec![b' '; 27];
        assert_eq!(td.copy_bytes_into(&mut buffer, 3), 23);
        assert_eq!(buffer, typed_array_for("   trailer\n<<\n/Foo /Bar\n>> "));
    }
}
