use std::fmt;

/// Trait implemented by all PDF object types.
/// Provides serialization to PDF byte format.
pub trait PdfObjectTrait: fmt::Display {
    /// Return the size in bytes when serialized.
    fn size_in_bytes(&self) -> usize;

    /// Copy serialized bytes into the buffer at the given offset.
    /// Returns the number of bytes written.
    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize;

    /// Serialize to a Vec<u8>.
    fn to_bytes(&self) -> Vec<u8> {
        let size = self.size_in_bytes();
        let mut buffer = vec![0u8; size];
        self.copy_bytes_into(&mut buffer, 0);
        buffer
    }
}

/// Enum representing any PDF object type.
#[derive(Debug, Clone)]
pub enum PdfObject {
    Name(super::PdfName),
    Ref(super::PdfRef),
    Number(super::PdfNumber),
    String(super::PdfString),
    HexString(super::PdfHexString),
    Bool(super::PdfBool),
    Null,
    Array(super::PdfArray),
    Dict(super::PdfDict),
    Stream(super::PdfRawStream),
}

impl fmt::Display for PdfObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfObject::Name(v) => write!(f, "{}", v),
            PdfObject::Ref(v) => write!(f, "{}", v),
            PdfObject::Number(v) => write!(f, "{}", v),
            PdfObject::String(v) => write!(f, "{}", v),
            PdfObject::HexString(v) => write!(f, "{}", v),
            PdfObject::Bool(v) => write!(f, "{}", v),
            PdfObject::Null => write!(f, "null"),
            PdfObject::Array(v) => write!(f, "{}", v),
            PdfObject::Dict(v) => write!(f, "{}", v),
            PdfObject::Stream(v) => write!(f, "{}", v),
        }
    }
}

impl PdfObjectTrait for PdfObject {
    fn size_in_bytes(&self) -> usize {
        match self {
            PdfObject::Name(v) => v.size_in_bytes(),
            PdfObject::Ref(v) => v.size_in_bytes(),
            PdfObject::Number(v) => v.size_in_bytes(),
            PdfObject::String(v) => v.size_in_bytes(),
            PdfObject::HexString(v) => v.size_in_bytes(),
            PdfObject::Bool(v) => v.size_in_bytes(),
            PdfObject::Null => 4,
            PdfObject::Array(v) => v.size_in_bytes(),
            PdfObject::Dict(v) => v.size_in_bytes(),
            PdfObject::Stream(v) => v.size_in_bytes(),
        }
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        match self {
            PdfObject::Name(v) => v.copy_bytes_into(buffer, offset),
            PdfObject::Ref(v) => v.copy_bytes_into(buffer, offset),
            PdfObject::Number(v) => v.copy_bytes_into(buffer, offset),
            PdfObject::String(v) => v.copy_bytes_into(buffer, offset),
            PdfObject::HexString(v) => v.copy_bytes_into(buffer, offset),
            PdfObject::Bool(v) => v.copy_bytes_into(buffer, offset),
            PdfObject::Null => {
                buffer[offset] = b'n';
                buffer[offset + 1] = b'u';
                buffer[offset + 2] = b'l';
                buffer[offset + 3] = b'l';
                4
            }
            PdfObject::Array(v) => v.copy_bytes_into(buffer, offset),
            PdfObject::Dict(v) => v.copy_bytes_into(buffer, offset),
            PdfObject::Stream(v) => v.copy_bytes_into(buffer, offset),
        }
    }
}

// Conversion impls
impl From<super::PdfName> for PdfObject {
    fn from(v: super::PdfName) -> Self { PdfObject::Name(v) }
}
impl From<super::PdfRef> for PdfObject {
    fn from(v: super::PdfRef) -> Self { PdfObject::Ref(v) }
}
impl From<super::PdfNumber> for PdfObject {
    fn from(v: super::PdfNumber) -> Self { PdfObject::Number(v) }
}
impl From<super::PdfString> for PdfObject {
    fn from(v: super::PdfString) -> Self { PdfObject::String(v) }
}
impl From<super::PdfHexString> for PdfObject {
    fn from(v: super::PdfHexString) -> Self { PdfObject::HexString(v) }
}
impl From<super::PdfBool> for PdfObject {
    fn from(v: super::PdfBool) -> Self { PdfObject::Bool(v) }
}
