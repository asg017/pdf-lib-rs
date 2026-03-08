use std::fmt;
use crate::core::objects::PdfRef;
use crate::core::objects::pdf_object::PdfObjectTrait;
use crate::core::syntax::CharCodes;
use crate::utils::copy_string_into_buffer;

/// An entry in a PDF cross-reference section.
#[derive(Debug, Clone)]
pub struct XRefEntry {
    pub pdf_ref: PdfRef,
    pub offset: u64,
    pub deleted: bool,
}

/// A PDF cross-reference section.
///
/// Entries should be added in order of ascending object number.
#[derive(Debug, Clone)]
pub struct PdfCrossRefSection {
    subsections: Vec<Vec<XRefEntry>>,
    chunk_idx: usize,
    chunk_length: usize,
}

impl PdfCrossRefSection {
    /// Create a new cross-reference section with the standard first entry (0 65535 f).
    pub fn create() -> Self {
        let first_entry = XRefEntry {
            pdf_ref: PdfRef::of(0, 65535),
            offset: 0,
            deleted: true,
        };
        PdfCrossRefSection {
            subsections: vec![vec![first_entry]],
            chunk_idx: 0,
            chunk_length: 1,
        }
    }

    /// Create an empty cross-reference section with no entries.
    pub fn create_empty() -> Self {
        PdfCrossRefSection {
            subsections: Vec::new(),
            chunk_idx: 0,
            chunk_length: 0,
        }
    }

    /// Add an in-use entry.
    pub fn add_entry(&mut self, pdf_ref: PdfRef, offset: u64) {
        self.append(XRefEntry {
            pdf_ref,
            offset,
            deleted: false,
        });
    }

    /// Add a deleted (free) entry.
    pub fn add_deleted_entry(&mut self, pdf_ref: PdfRef, next_free_object_number: u64) {
        self.append(XRefEntry {
            pdf_ref,
            offset: next_free_object_number,
            deleted: true,
        });
    }

    fn append(&mut self, curr_entry: XRefEntry) {
        if self.chunk_length == 0 {
            self.subsections.push(vec![curr_entry]);
            self.chunk_idx = 0;
            self.chunk_length = 1;
            return;
        }

        let chunk = &self.subsections[self.chunk_idx];
        let prev_entry = &chunk[self.chunk_length - 1];

        if curr_entry.pdf_ref.object_number - prev_entry.pdf_ref.object_number > 1 {
            self.subsections.push(vec![curr_entry]);
            self.chunk_idx += 1;
            self.chunk_length = 1;
        } else {
            self.subsections[self.chunk_idx].push(curr_entry);
            self.chunk_length += 1;
        }
    }

    fn pad_start(s: &str, length: usize, pad_char: char) -> String {
        if s.len() >= length {
            s.to_string()
        } else {
            let padding: String = std::iter::repeat_n(pad_char, length - s.len()).collect();
            format!("{}{}", padding, s)
        }
    }
}

impl fmt::Display for PdfCrossRefSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "xref")?;
        for subsection in &self.subsections {
            let first_obj_num = subsection[0].pdf_ref.object_number;
            writeln!(f, "{} {}", first_obj_num, subsection.len())?;
            for entry in subsection {
                let offset_str = Self::pad_start(&entry.offset.to_string(), 10, '0');
                let gen_str = Self::pad_start(&entry.pdf_ref.generation_number.to_string(), 5, '0');
                let flag = if entry.deleted { 'f' } else { 'n' };
                writeln!(f, "{} {} {} ", offset_str, gen_str, flag)?;
            }
        }
        Ok(())
    }
}

impl PdfObjectTrait for PdfCrossRefSection {
    fn size_in_bytes(&self) -> usize {
        let mut size = 5; // "xref\n"
        for subsection in &self.subsections {
            let first_obj_num_len = subsection[0].pdf_ref.object_number.to_string().len();
            let range_len = subsection.len().to_string().len();
            size += first_obj_num_len + 1 + range_len + 1; // "obj_num range\n"
            size += 20 * subsection.len(); // Each entry is 20 bytes
        }
        size
    }

    fn copy_bytes_into(&self, buffer: &mut [u8], offset: usize) -> usize {
        let initial_offset = offset;
        let mut off = offset;

        // "xref\n"
        buffer[off] = CharCodes::LowerX;
        off += 1;
        buffer[off] = CharCodes::LowerR;
        off += 1;
        buffer[off] = CharCodes::LowerE;
        off += 1;
        buffer[off] = CharCodes::LowerF;
        off += 1;
        buffer[off] = CharCodes::Newline;
        off += 1;

        for subsection in &self.subsections {
            let first_obj_num = subsection[0].pdf_ref.object_number.to_string();
            off += copy_string_into_buffer(&first_obj_num, buffer, off);
            buffer[off] = CharCodes::Space;
            off += 1;

            let range_length = subsection.len().to_string();
            off += copy_string_into_buffer(&range_length, buffer, off);
            buffer[off] = CharCodes::Newline;
            off += 1;

            for entry in subsection {
                let entry_offset = Self::pad_start(&entry.offset.to_string(), 10, '0');
                off += copy_string_into_buffer(&entry_offset, buffer, off);
                buffer[off] = CharCodes::Space;
                off += 1;

                let entry_gen =
                    Self::pad_start(&entry.pdf_ref.generation_number.to_string(), 5, '0');
                off += copy_string_into_buffer(&entry_gen, buffer, off);
                buffer[off] = CharCodes::Space;
                off += 1;

                buffer[off] = if entry.deleted { CharCodes::LowerF } else { CharCodes::LowerN };
                off += 1;

                buffer[off] = CharCodes::Space;
                off += 1;
                buffer[off] = CharCodes::Newline;
                off += 1;
            }
        }

        off - initial_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::typed_array_for;

    fn make_xref1() -> PdfCrossRefSection {
        let mut xref = PdfCrossRefSection::create();
        xref.add_entry(PdfRef::of(1, 0), 21);
        xref.add_deleted_entry(PdfRef::of(2, 1), 24);
        xref.add_entry(PdfRef::of(3, 0), 192188923);
        xref.add_entry(PdfRef::of(4, 0), 129219);
        xref
    }

    fn make_xref2() -> PdfCrossRefSection {
        let mut xref = PdfCrossRefSection::create();
        xref.add_entry(PdfRef::of(3, 0), 21);
        xref.add_deleted_entry(PdfRef::of(4, 1), 24);
        xref.add_entry(PdfRef::of(6, 0), 192188923);
        xref.add_entry(PdfRef::of(7, 0), 129219);
        xref
    }

    #[test]
    fn can_be_converted_to_string_single_subsection() {
        let xref1 = make_xref1();
        let expected = "xref\n\
                        0 5\n\
                        0000000000 65535 f \n\
                        0000000021 00000 n \n\
                        0000000024 00001 f \n\
                        0192188923 00000 n \n\
                        0000129219 00000 n \n";
        assert_eq!(xref1.to_string(), expected);
    }

    #[test]
    fn can_be_converted_to_string_multiple_subsections() {
        let xref2 = make_xref2();
        let expected = "xref\n\
                        0 1\n\
                        0000000000 65535 f \n\
                        3 2\n\
                        0000000021 00000 n \n\
                        0000000024 00001 f \n\
                        6 2\n\
                        0192188923 00000 n \n\
                        0000129219 00000 n \n";
        assert_eq!(xref2.to_string(), expected);
    }

    #[test]
    fn can_provide_size_in_bytes_single_subsection() {
        let xref1 = make_xref1();
        assert_eq!(xref1.size_in_bytes(), 109);
    }

    #[test]
    fn can_provide_size_in_bytes_multiple_subsections() {
        let xref2 = make_xref2();
        assert_eq!(xref2.size_in_bytes(), 117);
    }

    #[test]
    fn can_be_serialized_single_subsection() {
        let xref1 = make_xref1();
        let mut buffer = vec![b' '; 113];
        assert_eq!(xref1.copy_bytes_into(&mut buffer, 3), 109);
        let expected_str = "   xref\n\
                            0 5\n\
                            0000000000 65535 f \n\
                            0000000021 00000 n \n\
                            0000000024 00001 f \n\
                            0192188923 00000 n \n\
                            0000129219 00000 n \n ";
        assert_eq!(buffer, typed_array_for(expected_str));
    }

    #[test]
    fn can_be_serialized_multiple_subsections() {
        let xref2 = make_xref2();
        let mut buffer = vec![b' '; 121];
        assert_eq!(xref2.copy_bytes_into(&mut buffer, 3), 117);
        let expected_str = "   xref\n\
                            0 1\n\
                            0000000000 65535 f \n\
                            3 2\n\
                            0000000021 00000 n \n\
                            0000000024 00001 f \n\
                            6 2\n\
                            0192188923 00000 n \n\
                            0000129219 00000 n \n ";
        assert_eq!(buffer, typed_array_for(expected_str));
    }
}
