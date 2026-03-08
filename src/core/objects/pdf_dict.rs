use std::fmt;
use std::collections::BTreeMap;
use crate::core::syntax::CharCodes;
use super::pdf_name::PdfName;
use super::pdf_object::{PdfObject, PdfObjectTrait};

/// A PDF dictionary object, e.g., `<< /Type /Page /MediaBox [ 0 0 612 792 ] >>`.
///
/// Uses BTreeMap for deterministic key ordering (sorted alphabetically).
#[derive(Debug, Clone)]
pub struct PdfDict {
    dict: BTreeMap<String, (PdfName, PdfObject)>,
}

impl PdfDict {
    pub fn new() -> Self {
        PdfDict {
            dict: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, key: PdfName, value: PdfObject) {
        let key_str = key.as_string().to_string();
        self.dict.insert(key_str, (key, value));
    }

    pub fn get(&self, key: &PdfName) -> Option<&PdfObject> {
        self.dict.get(key.as_string()).map(|(_, v)| v)
    }

    pub fn has(&self, key: &PdfName) -> bool {
        self.dict.contains_key(key.as_string())
    }

    pub fn delete(&mut self, key: &PdfName) {
        self.dict.remove(key.as_string());
    }

    pub fn keys(&self) -> Vec<&PdfName> {
        self.dict.values().map(|(k, _)| k).collect()
    }

    pub fn values(&self) -> Vec<&PdfObject> {
        self.dict.values().map(|(_, v)| v).collect()
    }

    pub fn entries(&self) -> Vec<(&PdfName, &PdfObject)> {
        self.dict.values().map(|(k, v)| (k, v)).collect()
    }

    pub fn len(&self) -> usize {
        self.dict.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dict.is_empty()
    }
}

impl Default for PdfDict {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for PdfDict {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl fmt::Display for PdfDict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "<<")?;
        for (key, value) in self.dict.values() {
            writeln!(f, "{} {}", key, value)?;
        }
        write!(f, ">>")
    }
}

impl PdfObjectTrait for PdfDict {
    fn size_in_bytes(&self) -> usize {
        // "<<\n" + entries + ">>"
        let mut size = 5; // "<<\n" + ">>"
        for (key, value) in self.dict.values() {
            size += key.size_in_bytes() + 1 + value.size_in_bytes() + 1; // key + space + value + newline
        }
        size
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let initial_offset = offset;
        let mut off = offset;

        buffer[off] = CharCodes::LessThan;
        off += 1;
        buffer[off] = CharCodes::LessThan;
        off += 1;
        buffer[off] = CharCodes::Newline;
        off += 1;

        for (key, value) in self.dict.values() {
            off += key.copy_bytes_into(buffer, off);
            buffer[off] = CharCodes::Space;
            off += 1;
            off += value.copy_bytes_into(buffer, off);
            buffer[off] = CharCodes::Newline;
            off += 1;
        }

        buffer[off] = CharCodes::GreaterThan;
        off += 1;
        buffer[off] = CharCodes::GreaterThan;
        off += 1;

        off - initial_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::objects::PdfNumber;

    #[test]
    fn can_set_and_get() {
        let mut dict = PdfDict::new();
        let key = PdfName::of("Type");
        dict.set(key.clone(), PdfObject::Name(PdfName::of("Page")));
        assert!(dict.has(&PdfName::of("Type")));
        assert!(!dict.has(&PdfName::of("Missing")));
    }

    #[test]
    fn can_delete() {
        let mut dict = PdfDict::new();
        dict.set(
            PdfName::of("Foo"),
            PdfObject::Number(PdfNumber::of(42.0)),
        );
        assert!(dict.has(&PdfName::of("Foo")));
        dict.delete(&PdfName::of("Foo"));
        assert!(!dict.has(&PdfName::of("Foo")));
    }

    #[test]
    fn can_enumerate_entries() {
        let mut dict = PdfDict::new();
        dict.set(PdfName::of("A"), PdfObject::Number(PdfNumber::of(1.0)));
        dict.set(PdfName::of("B"), PdfObject::Number(PdfNumber::of(2.0)));
        assert_eq!(dict.len(), 2);
        assert_eq!(dict.keys().len(), 2);
    }

    #[test]
    fn can_be_serialized() {
        let mut dict = PdfDict::new();
        dict.set(PdfName::of("Type"), PdfObject::Name(PdfName::of("Page")));
        let size = dict.size_in_bytes();
        let mut buffer = vec![0u8; size];
        dict.copy_bytes_into(&mut buffer, 0);
        let result = String::from_utf8(buffer).unwrap();
        assert!(result.starts_with("<<\n"));
        assert!(result.ends_with(">>"));
        assert!(result.contains("/Type /Page"));
    }
}
