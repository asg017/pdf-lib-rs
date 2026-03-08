use std::collections::HashMap;
use crate::core::document::PdfHeader;
use crate::core::objects::{PdfObject, PdfRef};

/// PdfContext holds all indirect objects in a PDF document.
/// It is the central registry for the document's object graph.
#[derive(Debug)]
pub struct PdfContext {
    pub largest_object_number: u32,
    pub header: PdfHeader,
    pub trailer_info: TrailerInfo,
    indirect_objects: HashMap<PdfRefKey, PdfObject>,
}

/// Key for storing PdfRef in a HashMap.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PdfRefKey {
    object_number: u32,
    generation_number: u16,
}

impl From<&PdfRef> for PdfRefKey {
    fn from(r: &PdfRef) -> Self {
        PdfRefKey {
            object_number: r.object_number,
            generation_number: r.generation_number,
        }
    }
}

/// Trailer information.
#[derive(Debug, Default)]
pub struct TrailerInfo {
    pub root: Option<PdfObject>,
    pub encrypt: Option<PdfObject>,
    pub info: Option<PdfObject>,
    pub id: Option<PdfObject>,
}

impl PdfContext {
    pub fn create() -> Self {
        PdfContext {
            largest_object_number: 0,
            header: PdfHeader::for_version(1, 7),
            trailer_info: TrailerInfo::default(),
            indirect_objects: HashMap::new(),
        }
    }

    /// Assign an object to a reference.
    pub fn assign(&mut self, pdf_ref: &PdfRef, object: PdfObject) {
        if pdf_ref.object_number > self.largest_object_number {
            self.largest_object_number = pdf_ref.object_number;
        }
        self.indirect_objects.insert(PdfRefKey::from(pdf_ref), object);
    }

    /// Get the next available reference.
    pub fn next_ref(&mut self) -> PdfRef {
        self.largest_object_number += 1;
        PdfRef::of(self.largest_object_number, 0)
    }

    /// Register an object and return its reference.
    pub fn register(&mut self, object: PdfObject) -> PdfRef {
        let pdf_ref = self.next_ref();
        self.assign(&pdf_ref, object);
        pdf_ref
    }

    /// Look up an indirect object by reference.
    pub fn lookup(&self, pdf_ref: &PdfRef) -> Option<&PdfObject> {
        self.indirect_objects.get(&PdfRefKey::from(pdf_ref))
    }

    /// Delete an indirect object.
    pub fn delete(&mut self, pdf_ref: &PdfRef) -> bool {
        self.indirect_objects.remove(&PdfRefKey::from(pdf_ref)).is_some()
    }

    /// Enumerate all indirect objects, sorted by object number.
    pub fn enumerate_indirect_objects(&self) -> Vec<(PdfRef, &PdfObject)> {
        let mut entries: Vec<_> = self
            .indirect_objects
            .iter()
            .map(|(k, v)| (PdfRef::of(k.object_number, k.generation_number), v))
            .collect();
        entries.sort_by_key(|(r, _)| r.object_number);
        entries
    }

    /// Get number of indirect objects.
    pub fn object_count(&self) -> usize {
        self.indirect_objects.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::objects::{PdfNumber, PdfName};

    #[test]
    fn can_create_context() {
        let ctx = PdfContext::create();
        assert_eq!(ctx.largest_object_number, 0);
        assert_eq!(ctx.object_count(), 0);
    }

    #[test]
    fn can_assign_and_lookup() {
        let mut ctx = PdfContext::create();
        let r = PdfRef::of(1, 0);
        ctx.assign(&r, PdfObject::Number(PdfNumber::of(42.0)));
        assert!(ctx.lookup(&r).is_some());
        assert_eq!(ctx.largest_object_number, 1);
    }

    #[test]
    fn can_register_objects() {
        let mut ctx = PdfContext::create();
        let r1 = ctx.register(PdfObject::Name(PdfName::of("Foo")));
        let r2 = ctx.register(PdfObject::Name(PdfName::of("Bar")));
        assert_eq!(r1.object_number, 1);
        assert_eq!(r2.object_number, 2);
        assert_eq!(ctx.object_count(), 2);
    }

    #[test]
    fn can_delete_objects() {
        let mut ctx = PdfContext::create();
        let r = ctx.register(PdfObject::Number(PdfNumber::of(1.0)));
        assert!(ctx.delete(&r));
        assert!(ctx.lookup(&r).is_none());
    }

    #[test]
    fn can_enumerate_objects() {
        let mut ctx = PdfContext::create();
        ctx.register(PdfObject::Number(PdfNumber::of(1.0)));
        ctx.register(PdfObject::Number(PdfNumber::of(2.0)));
        let objects = ctx.enumerate_indirect_objects();
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].0.object_number, 1);
        assert_eq!(objects[1].0.object_number, 2);
    }
}
