use crate::core::context::PdfContext;
use crate::core::document::{PdfCrossRefSection, PdfTrailer, PdfTrailerDict};
use crate::core::objects::*;
use crate::core::objects::pdf_object::PdfObjectTrait;
use crate::core::syntax::CharCodes;
use crate::utils::copy_string_into_buffer;

/// Serializes a PdfContext back to PDF bytes.
pub struct PdfWriter;

impl PdfWriter {
    /// Serialize the context to a PDF byte buffer.
    pub fn serialize_to_buffer(context: &PdfContext) -> Vec<u8> {
        let objects = context.enumerate_indirect_objects();

        // First pass: compute size
        let header_size = context.header.size_in_bytes();
        let mut total_size = header_size + 2; // header + 2 newlines

        // Track offsets for xref
        let mut object_sizes = Vec::new();
        for (pdf_ref, object) in &objects {
            let obj_num_str = pdf_ref.object_number.to_string();
            let gen_num_str = pdf_ref.generation_number.to_string();
            let obj_header_size = obj_num_str.len() + 1 + gen_num_str.len() + 5; // "num gen obj\n"
            let obj_footer_size = 9; // "\nendobj\n\n"
            let obj_content_size = object.size_in_bytes();
            let obj_total = obj_header_size + obj_content_size + obj_footer_size;
            object_sizes.push(obj_total);
            total_size += obj_total;
        }

        // Build xref section
        let mut xref = PdfCrossRefSection::create();
        let mut offset = header_size + 2;
        for (i, (pdf_ref, _)) in objects.iter().enumerate() {
            xref.add_entry(
                PdfRef::of(pdf_ref.object_number, pdf_ref.generation_number),
                offset as u64,
            );
            offset += object_sizes[i];
        }
        let xref_offset = offset;

        // Build trailer dict
        let mut trailer_dict = PdfDict::new();
        trailer_dict.set(
            PdfName::of("Size"),
            PdfObject::Number(PdfNumber::of((context.largest_object_number + 1) as f64)),
        );
        if let Some(root) = &context.trailer_info.root {
            trailer_dict.set(PdfName::of("Root"), root.clone());
        }
        if let Some(info) = &context.trailer_info.info {
            trailer_dict.set(PdfName::of("Info"), info.clone());
        }

        let trailer_dict_obj = PdfTrailerDict::of(trailer_dict);
        let trailer = PdfTrailer::for_last_cross_ref_section_offset(xref_offset as u64);

        total_size += xref.size_in_bytes();
        total_size += trailer_dict_obj.size_in_bytes();
        total_size += 1; // newline between trailer dict and startxref
        total_size += trailer.size_in_bytes();

        // Second pass: write
        let mut buffer = vec![0u8; total_size];
        let mut off = 0;

        // Header
        off += context.header.copy_bytes_into(&mut buffer, off);
        buffer[off] = CharCodes::Newline;
        off += 1;
        buffer[off] = CharCodes::Newline;
        off += 1;

        // Objects
        for (pdf_ref, object) in &objects {
            let obj_num_str = pdf_ref.object_number.to_string();
            off += copy_string_into_buffer(&obj_num_str, &mut buffer, off);
            buffer[off] = CharCodes::Space;
            off += 1;

            let gen_num_str = pdf_ref.generation_number.to_string();
            off += copy_string_into_buffer(&gen_num_str, &mut buffer, off);
            buffer[off] = CharCodes::Space;
            off += 1;

            buffer[off] = b'o';
            off += 1;
            buffer[off] = b'b';
            off += 1;
            buffer[off] = b'j';
            off += 1;
            buffer[off] = CharCodes::Newline;
            off += 1;

            off += object.copy_bytes_into(&mut buffer, off);

            buffer[off] = CharCodes::Newline;
            off += 1;
            for &b in b"endobj" {
                buffer[off] = b;
                off += 1;
            }
            buffer[off] = CharCodes::Newline;
            off += 1;
            buffer[off] = CharCodes::Newline;
            off += 1;
        }

        // Xref
        off += xref.copy_bytes_into(&mut buffer, off);

        // Trailer dict
        off += trailer_dict_obj.copy_bytes_into(&mut buffer, off);
        buffer[off] = CharCodes::Newline;
        off += 1;

        // Startxref + %%EOF
        off += trailer.copy_bytes_into(&mut buffer, off);

        buffer.truncate(off);
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::parser::PdfParser;

    #[test]
    fn can_serialize_empty_context() {
        let context = PdfContext::create();
        let bytes = PdfWriter::serialize_to_buffer(&context);
        assert!(bytes.starts_with(b"%PDF-1.7"));
        assert!(bytes.windows(5).any(|w| w == b"%%EOF"));
    }

    #[test]
    fn can_serialize_context_with_objects() {
        let mut context = PdfContext::create();
        context.register(PdfObject::Number(PdfNumber::of(42.0)));
        context.register(PdfObject::Name(PdfName::of("Foo")));
        let bytes = PdfWriter::serialize_to_buffer(&context);

        let output = String::from_utf8_lossy(&bytes);
        assert!(output.contains("1 0 obj"));
        assert!(output.contains("42"));
        assert!(output.contains("2 0 obj"));
        assert!(output.contains("/Foo"));
        assert!(output.contains("endobj"));
        assert!(output.contains("xref"));
        assert!(output.contains("%%EOF"));
    }

    #[test]
    fn roundtrip_parse_serialize_parse() {
        let mut context = PdfContext::create();

        let mut page_dict = PdfDict::new();
        page_dict.set(PdfName::of("Type"), PdfObject::Name(PdfName::of("Page")));
        let page_ref = context.register(PdfObject::Dict(page_dict));

        let mut catalog = PdfDict::new();
        catalog.set(PdfName::of("Type"), PdfObject::Name(PdfName::of("Catalog")));
        catalog.set(PdfName::of("Pages"), PdfObject::Ref(page_ref));
        let catalog_ref = context.register(PdfObject::Dict(catalog));

        context.trailer_info.root = Some(PdfObject::Ref(catalog_ref));

        let bytes = PdfWriter::serialize_to_buffer(&context);
        let parser = PdfParser::for_bytes(&bytes);
        let context2 = parser.parse_document().unwrap();

        assert_eq!(context2.object_count(), context.object_count());
    }
}
