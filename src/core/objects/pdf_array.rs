use std::fmt;
use crate::core::syntax::CharCodes;
use super::pdf_object::{PdfObject, PdfObjectTrait};

/// A PDF array object, e.g., `[ 1 2 3 ]`.
#[derive(Debug, Clone)]
pub struct PdfArray {
    array: Vec<PdfObject>,
}

impl PdfArray {
    pub fn new() -> Self {
        PdfArray { array: Vec::new() }
    }

    pub fn size(&self) -> usize {
        self.array.len()
    }

    pub fn push(&mut self, object: PdfObject) {
        self.array.push(object);
    }

    pub fn insert(&mut self, index: usize, object: PdfObject) {
        self.array.insert(index, object);
    }

    pub fn remove(&mut self, index: usize) -> PdfObject {
        self.array.remove(index)
    }

    pub fn get(&self, index: usize) -> Option<&PdfObject> {
        self.array.get(index)
    }

    pub fn set(&mut self, index: usize, object: PdfObject) {
        self.array[index] = object;
    }

    pub fn as_slice(&self) -> &[PdfObject] {
        &self.array
    }
}

impl Default for PdfArray {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for PdfArray {
    fn eq(&self, _other: &Self) -> bool {
        // Arrays don't have value equality in the same way
        false
    }
}

impl fmt::Display for PdfArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ ")?;
        for item in &self.array {
            write!(f, "{} ", item)?;
        }
        write!(f, "]")
    }
}

impl PdfObjectTrait for PdfArray {
    fn size_in_bytes(&self) -> usize {
        // "[ " + each item + " " + "]"
        let mut size = 3; // "[ " and "]"
        for item in &self.array {
            size += item.size_in_bytes() + 1; // item + space
        }
        size
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let initial_offset = offset;
        let mut off = offset;

        buffer[off] = CharCodes::LeftSquareBracket;
        off += 1;
        buffer[off] = CharCodes::Space;
        off += 1;

        for item in &self.array {
            off += item.copy_bytes_into(buffer, off);
            buffer[off] = CharCodes::Space;
            off += 1;
        }

        buffer[off] = CharCodes::RightSquareBracket;
        off += 1;

        off - initial_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::objects::{PdfNumber, PdfName};

    #[test]
    fn can_push_and_get() {
        let mut arr = PdfArray::new();
        arr.push(PdfObject::Number(PdfNumber::of(42.0)));
        arr.push(PdfObject::Name(PdfName::of("Foo")));
        assert_eq!(arr.size(), 2);
    }

    #[test]
    fn can_be_converted_to_string() {
        let mut arr = PdfArray::new();
        arr.push(PdfObject::Number(PdfNumber::of(1.0)));
        arr.push(PdfObject::Number(PdfNumber::of(2.0)));
        assert_eq!(arr.to_string(), "[ 1 2 ]");
    }

    #[test]
    fn can_insert_and_remove() {
        let mut arr = PdfArray::new();
        arr.push(PdfObject::Number(PdfNumber::of(1.0)));
        arr.push(PdfObject::Number(PdfNumber::of(3.0)));
        arr.insert(1, PdfObject::Number(PdfNumber::of(2.0)));
        assert_eq!(arr.size(), 3);
        assert_eq!(arr.to_string(), "[ 1 2 3 ]");

        arr.remove(1);
        assert_eq!(arr.to_string(), "[ 1 3 ]");
    }

    #[test]
    fn can_provide_size_in_bytes() {
        let arr = PdfArray::new();
        assert_eq!(arr.size_in_bytes(), 3); // "[ ]"

        let mut arr2 = PdfArray::new();
        arr2.push(PdfObject::Number(PdfNumber::of(1.0)));
        // "[ 1 ]" = 5 bytes
        assert_eq!(arr2.size_in_bytes(), 5);
    }

    #[test]
    fn can_be_serialized() {
        let mut arr = PdfArray::new();
        arr.push(PdfObject::Number(PdfNumber::of(1.0)));
        arr.push(PdfObject::Number(PdfNumber::of(2.0)));
        let size = arr.size_in_bytes();
        let mut buffer = vec![0u8; size];
        arr.copy_bytes_into(&mut buffer, 0);
        assert_eq!(buffer, b"[ 1 2 ]");
    }
}
