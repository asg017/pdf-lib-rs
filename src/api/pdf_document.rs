use crate::core::context::PdfContext;
use crate::core::errors::{PdfError, Result};
use crate::core::objects::*;
use crate::core::parser::PdfParser;
use crate::core::writers::PdfWriter;

/// Options for loading a PDF document.
#[derive(Debug, Default)]
pub struct LoadOptions {
    /// If true, encrypted PDFs will be loaded without error.
    pub ignore_encryption: bool,
    /// If true, invalid objects will cause an error instead of a warning.
    pub throw_on_invalid_object: bool,
}

/// A high-level representation of a PDF document.
///
/// This is the main entry point for creating and modifying PDF documents.
pub struct PdfDocument {
    context: PdfContext,
    is_encrypted: bool,
}

impl PdfDocument {
    /// Create a new, empty PDF document.
    pub fn create() -> Self {
        let mut context = PdfContext::create();

        // Create the minimal document structure:
        // 1. Page tree root
        let mut pages_dict = PdfDict::new();
        pages_dict.set(PdfName::of("Type"), PdfObject::Name(PdfName::of("Pages")));
        pages_dict.set(PdfName::of("Kids"), PdfObject::Array(PdfArray::new()));
        pages_dict.set(PdfName::of("Count"), PdfObject::Number(PdfNumber::of(0.0)));
        let pages_ref = context.register(PdfObject::Dict(pages_dict));

        // 2. Catalog
        let mut catalog_dict = PdfDict::new();
        catalog_dict.set(PdfName::of("Type"), PdfObject::Name(PdfName::of("Catalog")));
        catalog_dict.set(PdfName::of("Pages"), PdfObject::Ref(pages_ref));
        let catalog_ref = context.register(PdfObject::Dict(catalog_dict));

        context.trailer_info.root = Some(PdfObject::Ref(catalog_ref));

        PdfDocument {
            context,
            is_encrypted: false,
        }
    }

    /// Load an existing PDF document from bytes.
    pub fn load(bytes: &[u8]) -> Result<Self> {
        Self::load_with_options(bytes, LoadOptions::default())
    }

    /// Load an existing PDF document from bytes with options.
    pub fn load_with_options(bytes: &[u8], options: LoadOptions) -> Result<Self> {
        let parser = PdfParser::for_bytes_with_options(bytes, options.throw_on_invalid_object);
        let context = parser.parse_document()?;

        let is_encrypted = context.trailer_info.encrypt.is_some();

        if is_encrypted && !options.ignore_encryption {
            return Err(PdfError::EncryptedPdf);
        }

        Ok(PdfDocument {
            context,
            is_encrypted,
        })
    }

    /// Save the document to PDF bytes.
    pub fn save(&self) -> Vec<u8> {
        PdfWriter::serialize_to_buffer(&self.context)
    }

    /// Returns true if the document is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }

    /// Get the number of pages in the document.
    pub fn get_page_count(&self) -> usize {
        self.get_page_refs().len()
    }

    /// Get the page indices (0-based).
    pub fn get_page_indices(&self) -> Vec<usize> {
        (0..self.get_page_count()).collect()
    }

    /// Add a new blank page to the end of the document with the given size.
    pub fn add_page(&mut self, size: [f64; 2]) -> PdfRef {
        let pages_ref = self.get_pages_ref();

        // Create the page
        let mut page_dict = PdfDict::new();
        page_dict.set(PdfName::of("Type"), PdfObject::Name(PdfName::of("Page")));
        page_dict.set(PdfName::of("Parent"), PdfObject::Ref(pages_ref.clone()));

        let mut media_box = PdfArray::new();
        media_box.push(PdfObject::Number(PdfNumber::of(0.0)));
        media_box.push(PdfObject::Number(PdfNumber::of(0.0)));
        media_box.push(PdfObject::Number(PdfNumber::of(size[0])));
        media_box.push(PdfObject::Number(PdfNumber::of(size[1])));
        page_dict.set(PdfName::of("MediaBox"), PdfObject::Array(media_box));

        let page_ref = self.context.register(PdfObject::Dict(page_dict));

        // Add to page tree
        self.add_page_ref_to_tree(&pages_ref, &page_ref);

        page_ref
    }

    /// Insert a new blank page at the given index.
    pub fn insert_page(&mut self, index: usize, size: [f64; 2]) -> PdfRef {
        let pages_ref = self.get_pages_ref();

        let mut page_dict = PdfDict::new();
        page_dict.set(PdfName::of("Type"), PdfObject::Name(PdfName::of("Page")));
        page_dict.set(PdfName::of("Parent"), PdfObject::Ref(pages_ref.clone()));

        let mut media_box = PdfArray::new();
        media_box.push(PdfObject::Number(PdfNumber::of(0.0)));
        media_box.push(PdfObject::Number(PdfNumber::of(0.0)));
        media_box.push(PdfObject::Number(PdfNumber::of(size[0])));
        media_box.push(PdfObject::Number(PdfNumber::of(size[1])));
        page_dict.set(PdfName::of("MediaBox"), PdfObject::Array(media_box));

        let page_ref = self.context.register(PdfObject::Dict(page_dict));

        // Insert at index in Kids array
        self.insert_page_ref_in_tree(&pages_ref, &page_ref, index);

        page_ref
    }

    /// Remove a page at the given index.
    pub fn remove_page(&mut self, index: usize) {
        let pages_ref = self.get_pages_ref();
        if let Some(PdfObject::Dict(pages_dict)) = self.context.lookup(&pages_ref).cloned() {
            if let Some(PdfObject::Array(mut kids)) = pages_dict.get(&PdfName::of("Kids")).cloned() {
                if index < kids.size() {
                    kids.remove(index);
                    let new_count = kids.size() as f64;
                    let mut new_pages_dict = pages_dict.clone();
                    new_pages_dict.set(PdfName::of("Kids"), PdfObject::Array(kids));
                    new_pages_dict.set(PdfName::of("Count"), PdfObject::Number(PdfNumber::of(new_count)));
                    self.context.assign(&pages_ref, PdfObject::Dict(new_pages_dict));
                }
            }
        }
    }

    /// Copy pages from another document. Returns the new page refs.
    pub fn copy_pages(&mut self, src_doc: &PdfDocument, indices: &[usize]) -> Vec<PdfRef> {
        let src_page_refs = src_doc.get_page_refs();
        let pages_ref = self.get_pages_ref();
        let mut new_refs = Vec::new();

        for &idx in indices {
            if idx >= src_page_refs.len() {
                continue;
            }
            let src_page_ref = &src_page_refs[idx];

            // Deep-copy the page object
            if let Some(src_page) = src_doc.context.lookup(src_page_ref) {
                let mut page = src_page.clone();

                // Update the Parent reference to our page tree
                if let PdfObject::Dict(ref mut dict) = page {
                    dict.set(PdfName::of("Parent"), PdfObject::Ref(pages_ref.clone()));
                }

                let new_ref = self.context.register(page);
                self.add_page_ref_to_tree(&pages_ref, &new_ref);
                new_refs.push(new_ref);
            }
        }

        new_refs
    }

    /// Set the document title.
    pub fn set_title(&mut self, title: &str) {
        self.set_info_field("Title", title);
    }

    /// Set the document author.
    pub fn set_author(&mut self, author: &str) {
        self.set_info_field("Author", author);
    }

    /// Set the document subject.
    pub fn set_subject(&mut self, subject: &str) {
        self.set_info_field("Subject", subject);
    }

    /// Set the document keywords.
    pub fn set_keywords(&mut self, keywords: &[&str]) {
        self.set_info_field("Keywords", &keywords.join(", "));
    }

    /// Set the document creator.
    pub fn set_creator(&mut self, creator: &str) {
        self.set_info_field("Creator", creator);
    }

    /// Set the document producer.
    pub fn set_producer(&mut self, producer: &str) {
        self.set_info_field("Producer", producer);
    }

    /// Get the document title, if any.
    pub fn get_title(&self) -> Option<String> {
        self.get_info_field("Title")
    }

    /// Get the document author, if any.
    pub fn get_author(&self) -> Option<String> {
        self.get_info_field("Author")
    }

    /// Get direct access to the context (for advanced use).
    pub fn context(&self) -> &PdfContext {
        &self.context
    }

    /// Get mutable access to the context.
    pub fn context_mut(&mut self) -> &mut PdfContext {
        &mut self.context
    }

    // --- Private helpers ---

    fn get_catalog_ref(&self) -> Option<PdfRef> {
        if let Some(PdfObject::Ref(r)) = &self.context.trailer_info.root {
            Some(r.clone())
        } else {
            None
        }
    }

    fn get_pages_ref(&self) -> PdfRef {
        if let Some(catalog_ref) = self.get_catalog_ref() {
            if let Some(PdfObject::Dict(catalog)) = self.context.lookup(&catalog_ref) {
                if let Some(PdfObject::Ref(pages_ref)) = catalog.get(&PdfName::of("Pages")) {
                    return pages_ref.clone();
                }
            }
        }
        // Fallback: should not happen in a well-formed document
        PdfRef::of(1, 0)
    }

    /// Get the refs for each page (public for inspection).
    pub fn get_page_refs(&self) -> Vec<PdfRef> {
        let pages_ref = self.get_pages_ref();
        self.collect_page_refs(&pages_ref)
    }

    fn collect_page_refs(&self, node_ref: &PdfRef) -> Vec<PdfRef> {
        let mut result = Vec::new();
        if let Some(PdfObject::Dict(dict)) = self.context.lookup(node_ref) {
            if let Some(PdfObject::Name(type_name)) = dict.get(&PdfName::of("Type")) {
                let type_str = type_name.as_string();
                if type_str == "/Page" {
                    result.push(node_ref.clone());
                } else if type_str == "/Pages" {
                    if let Some(PdfObject::Array(kids)) = dict.get(&PdfName::of("Kids")) {
                        for i in 0..kids.size() {
                            if let Some(PdfObject::Ref(kid_ref)) = kids.get(i) {
                                result.extend(self.collect_page_refs(kid_ref));
                            }
                        }
                    }
                }
            }
        }
        result
    }

    fn add_page_ref_to_tree(&mut self, pages_ref: &PdfRef, page_ref: &PdfRef) {
        if let Some(PdfObject::Dict(pages_dict)) = self.context.lookup(pages_ref).cloned() {
            let mut kids = if let Some(PdfObject::Array(k)) = pages_dict.get(&PdfName::of("Kids")) {
                k.clone()
            } else {
                PdfArray::new()
            };

            kids.push(PdfObject::Ref(page_ref.clone()));
            let new_count = kids.size() as f64;

            let mut new_dict = pages_dict.clone();
            new_dict.set(PdfName::of("Kids"), PdfObject::Array(kids));
            new_dict.set(PdfName::of("Count"), PdfObject::Number(PdfNumber::of(new_count)));
            self.context.assign(pages_ref, PdfObject::Dict(new_dict));
        }
    }

    fn insert_page_ref_in_tree(&mut self, pages_ref: &PdfRef, page_ref: &PdfRef, index: usize) {
        if let Some(PdfObject::Dict(pages_dict)) = self.context.lookup(pages_ref).cloned() {
            let mut kids = if let Some(PdfObject::Array(k)) = pages_dict.get(&PdfName::of("Kids")) {
                k.clone()
            } else {
                PdfArray::new()
            };

            let insert_idx = index.min(kids.size());
            kids.insert(insert_idx, PdfObject::Ref(page_ref.clone()));
            let new_count = kids.size() as f64;

            let mut new_dict = pages_dict.clone();
            new_dict.set(PdfName::of("Kids"), PdfObject::Array(kids));
            new_dict.set(PdfName::of("Count"), PdfObject::Number(PdfNumber::of(new_count)));
            self.context.assign(pages_ref, PdfObject::Dict(new_dict));
        }
    }

    fn get_or_create_info_dict(&mut self) -> PdfRef {
        // Check if Info dict already exists
        if let Some(PdfObject::Ref(info_ref)) = &self.context.trailer_info.info {
            return info_ref.clone();
        }

        // Create new Info dictionary
        let info_dict = PdfDict::new();
        let info_ref = self.context.register(PdfObject::Dict(info_dict));
        self.context.trailer_info.info = Some(PdfObject::Ref(info_ref.clone()));
        info_ref
    }

    fn set_info_field(&mut self, field: &str, value: &str) {
        let info_ref = self.get_or_create_info_dict();
        if let Some(PdfObject::Dict(info_dict)) = self.context.lookup(&info_ref).cloned() {
            let mut new_dict = info_dict;
            new_dict.set(
                PdfName::of(field),
                PdfObject::HexString(PdfHexString::from_text(value)),
            );
            self.context.assign(&info_ref, PdfObject::Dict(new_dict));
        }
    }

    fn get_info_field(&self, field: &str) -> Option<String> {
        if let Some(PdfObject::Ref(info_ref)) = &self.context.trailer_info.info {
            if let Some(PdfObject::Dict(info_dict)) = self.context.lookup(info_ref) {
                match info_dict.get(&PdfName::of(field)) {
                    Some(PdfObject::String(s)) => return Some(s.decode_text()),
                    Some(PdfObject::HexString(s)) => return Some(s.decode_text()),
                    _ => return None,
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::sizes::PageSizes;

    #[test]
    fn can_create_empty_document() {
        let doc = PdfDocument::create();
        assert_eq!(doc.get_page_count(), 0);
        assert!(!doc.is_encrypted());
    }

    #[test]
    fn can_add_pages() {
        let mut doc = PdfDocument::create();
        doc.add_page(PageSizes::LETTER);
        doc.add_page(PageSizes::A4);
        assert_eq!(doc.get_page_count(), 2);
    }

    #[test]
    fn can_insert_page() {
        let mut doc = PdfDocument::create();
        doc.add_page(PageSizes::LETTER);
        doc.add_page(PageSizes::LETTER);
        doc.insert_page(1, PageSizes::A4);
        assert_eq!(doc.get_page_count(), 3);
    }

    #[test]
    fn can_remove_page() {
        let mut doc = PdfDocument::create();
        doc.add_page(PageSizes::LETTER);
        doc.add_page(PageSizes::A4);
        assert_eq!(doc.get_page_count(), 2);
        doc.remove_page(0);
        assert_eq!(doc.get_page_count(), 1);
    }

    #[test]
    fn can_set_and_get_metadata() {
        let mut doc = PdfDocument::create();
        doc.set_title("Test Document");
        doc.set_author("Test Author");
        assert_eq!(doc.get_title(), Some("Test Document".to_string()));
        assert_eq!(doc.get_author(), Some("Test Author".to_string()));
    }

    #[test]
    fn can_save_and_reload() {
        let mut doc = PdfDocument::create();
        doc.add_page(PageSizes::LETTER);
        doc.add_page(PageSizes::A4);
        doc.set_title("Roundtrip Test");

        let bytes = doc.save();

        let doc2 = PdfDocument::load(&bytes).unwrap();
        assert_eq!(doc2.get_page_count(), 2);
        assert_eq!(doc2.get_title(), Some("Roundtrip Test".to_string()));
    }

    #[test]
    fn can_copy_pages_between_documents() {
        let mut doc1 = PdfDocument::create();
        doc1.add_page(PageSizes::LETTER);
        doc1.add_page(PageSizes::A4);
        doc1.add_page(PageSizes::LEGAL);

        let mut doc2 = PdfDocument::create();
        let copied = doc2.copy_pages(&doc1, &[0, 2]);
        assert_eq!(copied.len(), 2);
        assert_eq!(doc2.get_page_count(), 2);
    }

    #[test]
    fn can_load_real_pdf() {
        let bytes = std::fs::read("test_assets/pdfs/normal.pdf").unwrap();
        let doc = PdfDocument::load(&bytes).unwrap();
        assert!(doc.get_page_count() > 0);
        assert!(!doc.is_encrypted());
    }

    #[test]
    fn throws_for_encrypted_pdf() {
        let bytes = std::fs::read("test_assets/pdfs/encrypted_old.pdf").unwrap();
        let result = PdfDocument::load(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn allows_encrypted_pdf_with_ignore_flag() {
        let bytes = std::fs::read("test_assets/pdfs/encrypted_old.pdf").unwrap();
        let result = PdfDocument::load_with_options(
            &bytes,
            LoadOptions {
                ignore_encryption: true,
                ..Default::default()
            },
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_encrypted());
    }

    #[test]
    fn roundtrip_load_save_load() {
        let bytes = std::fs::read("test_assets/pdfs/normal.pdf").unwrap();
        let doc = PdfDocument::load(&bytes).unwrap();
        let page_count = doc.get_page_count();

        let saved_bytes = doc.save();
        let doc2 = PdfDocument::load(&saved_bytes).unwrap();
        assert_eq!(doc2.get_page_count(), page_count);
    }
}
