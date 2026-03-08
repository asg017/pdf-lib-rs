use crate::core::context::PdfContext;
use crate::core::document::PdfHeader;
use crate::core::errors::{PdfError, Result};
use crate::core::objects::*;
use crate::core::syntax::{is_delimiter, is_digit, is_whitespace, CharCodes};
use super::byte_stream::ByteStream;
use flate2::read::ZlibDecoder;
use std::io::Read;

/// PDF document parser. Parses raw PDF bytes into a PdfContext.
pub struct PdfParser<'a> {
    bytes: ByteStream<'a>,
    context: PdfContext,
    throw_on_invalid_object: bool,
}

impl<'a> PdfParser<'a> {
    pub fn for_bytes(pdf_bytes: &'a [u8]) -> Self {
        Self::for_bytes_with_options(pdf_bytes, false)
    }

    pub fn for_bytes_with_options(pdf_bytes: &'a [u8], throw_on_invalid_object: bool) -> Self {
        PdfParser {
            bytes: ByteStream::of(pdf_bytes),
            context: PdfContext::create(),
            throw_on_invalid_object,
        }
    }

    /// Parse a single PDF object value from the bytes.
    pub fn parse_single_object(mut self) -> Result<PdfObject> {
        self.parse_object()
    }

    /// Parse a complete PDF document and return the PdfContext.
    pub fn parse_document(mut self) -> Result<PdfContext> {
        self.context.header = self.parse_header()?;
        self.skip_binary_comment();

        let mut prev_offset = None;
        while !self.bytes.done() {
            self.parse_document_section()?;
            let offset = self.bytes.offset();
            if prev_offset == Some(offset) {
                let pos = self.bytes.position();
                return Err(PdfError::StalledParser {
                    line: pos.line,
                    column: pos.column,
                    offset: pos.offset,
                });
            }
            prev_offset = Some(offset);
        }

        // Remove object 0 0 R if it was parsed
        self.context.delete(&PdfRef::of(0, 0));

        Ok(self.context)
    }

    fn parse_header(&mut self) -> Result<PdfHeader> {
        self.skip_whitespace();

        // Find %PDF- header
        while !self.bytes.done() {
            if self.matches_keyword(b"%PDF-") {
                break;
            }
            self.bytes.next();
        }

        if self.bytes.done() {
            return Err(PdfError::MissingPdfHeader);
        }

        // Skip past "%PDF-"
        for _ in 0..5 {
            self.bytes.next();
        }

        // Parse version: major.minor
        let major = self.parse_raw_int()? as u8;
        self.expect_byte(CharCodes::Period)?;
        let minor = self.parse_raw_int()? as u8;

        Ok(PdfHeader::for_version(major, minor))
    }

    fn skip_binary_comment(&mut self) {
        self.skip_whitespace();
        // Skip binary comment line (starts with % followed by high bytes)
        if self.bytes.peek() == Some(CharCodes::Percent) {
            while !self.bytes.done() {
                let byte = self.bytes.peek();
                if byte == Some(CharCodes::Newline) || byte == Some(CharCodes::CarriageReturn) {
                    break;
                }
                self.bytes.next();
            }
        }
    }

    fn parse_document_section(&mut self) -> Result<()> {
        self.skip_whitespace_and_comments();

        if self.bytes.done() {
            return Ok(());
        }

        // Try to determine what we're looking at
        if self.matches_keyword(b"xref") {
            self.skip_xref_section();
        } else if self.matches_keyword(b"trailer") {
            self.skip_trailer();
        } else if self.matches_keyword(b"startxref") {
            self.skip_startxref();
        } else if self.matches_keyword(b"%%EOF") {
            self.skip_keyword(b"%%EOF");
            self.skip_junk_after_eof();
        } else {
            self.try_parse_indirect_object()?;
        }

        Ok(())
    }

    fn try_parse_indirect_object(&mut self) -> Result<()> {
        let start_offset = self.bytes.offset();

        // Try to parse: <number> <number> obj
        let obj_num = match self.try_parse_int() {
            Some(n) => n as u32,
            None => {
                // Not an indirect object, skip this byte
                self.bytes.next();
                return Ok(());
            }
        };

        self.skip_whitespace();
        let gen_num = match self.try_parse_int() {
            Some(n) => n as u16,
            None => {
                self.bytes.move_to(start_offset + 1);
                return Ok(());
            }
        };

        self.skip_whitespace();
        if !self.matches_keyword(b"obj") {
            self.bytes.move_to(start_offset + 1);
            return Ok(());
        }
        self.skip_keyword(b"obj");
        self.skip_whitespace_and_comments();

        // Parse the object value
        let object = match self.parse_object() {
            Ok(obj) => obj,
            Err(e) => {
                if self.throw_on_invalid_object {
                    return Err(e);
                }
                eprintln!(
                    "Warning: Trying to parse invalid object: {e}"
                );
                self.skip_to_endobj();
                return Ok(());
            }
        };

        self.skip_whitespace_and_comments();

        // Check for stream
        let final_object = if self.matches_keyword(b"stream") {
            self.parse_stream_after_dict(object)?
        } else {
            object
        };

        let pdf_ref = PdfRef::of(obj_num, gen_num);

        // Handle cross-reference streams and object streams before storing
        if let PdfObject::Stream(ref stream) = final_object {
            if let Some(PdfObject::Name(type_name)) = stream.dict.get(&PdfName::of("Type")) {
                let type_str = type_name.as_string();
                if type_str == "/XRef" {
                    self.extract_trailer_info_from_dict(&stream.dict);
                } else if type_str == "/ObjStm" {
                    self.parse_object_stream(stream);
                }
            }
        }

        self.context.assign(&pdf_ref, final_object);

        self.skip_whitespace_and_comments();
        if self.matches_keyword(b"endobj") {
            self.skip_keyword(b"endobj");
        }

        Ok(())
    }

    fn parse_stream_after_dict(&mut self, dict_object: PdfObject) -> Result<PdfObject> {
        self.skip_keyword(b"stream");

        // Skip stream keyword newline(s)
        if self.bytes.peek() == Some(CharCodes::CarriageReturn) {
            self.bytes.next();
        }
        if self.bytes.peek() == Some(CharCodes::Newline) {
            self.bytes.next();
        }

        // Find endstream
        let start = self.bytes.offset();
        let end;

        // Try to get length from dictionary
        if let PdfObject::Dict(ref dict) = dict_object {
            if let Some(PdfObject::Number(n)) = dict.get(&PdfName::length()) {
                let length = n.as_number() as usize;
                end = start + length;
                self.bytes.move_to(end);
            } else {
                // Scan for endstream
                end = self.find_endstream(start);
            }
        } else {
            end = self.find_endstream(start);
        }

        let contents = self.bytes.slice(start, end).to_vec();
        self.skip_whitespace();
        if self.matches_keyword(b"endstream") {
            self.skip_keyword(b"endstream");
        }

        let dict = if let PdfObject::Dict(d) = dict_object {
            d
        } else {
            PdfDict::new()
        };

        Ok(PdfObject::Stream(PdfRawStream::of(dict, contents)))
    }

    fn find_endstream(&mut self, _start: usize) -> usize {
        let search_start = self.bytes.offset();
        while !self.bytes.done() {
            if self.matches_keyword(b"endstream") {
                return self.bytes.offset();
            }
            self.bytes.next();
        }
        // If we didn't find it, return current position
        self.bytes.move_to(search_start);
        search_start
    }

    /// Parse a single PDF object value.
    fn parse_object(&mut self) -> Result<PdfObject> {
        self.skip_whitespace_and_comments();

        match self.bytes.peek() {
            None => Err(PdfError::UnexpectedObjectType),
            Some(b) => match b {
                CharCodes::ForwardSlash => self.parse_name(),
                CharCodes::LessThan => {
                    if self.bytes.peek_ahead(1) == Some(CharCodes::LessThan) {
                        self.parse_dict()
                    } else {
                        self.parse_hex_string()
                    }
                }
                CharCodes::LeftParen => self.parse_literal_string(),
                CharCodes::LeftSquareBracket => self.parse_array(),
                b't' if self.matches_keyword(b"true") => {
                    self.skip_keyword(b"true");
                    Ok(PdfObject::Bool(PdfBool::TRUE))
                }
                b'f' if self.matches_keyword(b"false") => {
                    self.skip_keyword(b"false");
                    Ok(PdfObject::Bool(PdfBool::FALSE))
                }
                b'n' if self.matches_keyword(b"null") => {
                    self.skip_keyword(b"null");
                    Ok(PdfObject::Null)
                }
                _ if is_digit(b) || b == CharCodes::Plus || b == CharCodes::Minus || b == CharCodes::Period => {
                    self.parse_number_or_ref()
                }
                _ => {
                    let pos = self.bytes.position();
                    Err(PdfError::InvalidObjectParsing {
                        line: pos.line,
                        column: pos.column,
                        offset: pos.offset,
                    })
                }
            },
        }
    }

    fn parse_name(&mut self) -> Result<PdfObject> {
        self.bytes.next(); // skip /
        let mut name = String::new();
        while !self.bytes.done() {
            let b = self.bytes.peek().unwrap();
            if is_whitespace(b) || is_delimiter(b) {
                break;
            }
            name.push(self.bytes.next().unwrap() as char);
        }
        Ok(PdfObject::Name(PdfName::of(&name)))
    }

    fn parse_hex_string(&mut self) -> Result<PdfObject> {
        self.bytes.next(); // skip <
        let mut hex = String::new();
        while !self.bytes.done() {
            let b = self.bytes.peek().unwrap();
            if b == CharCodes::GreaterThan {
                self.bytes.next();
                break;
            }
            if !is_whitespace(b) {
                hex.push(b as char);
            }
            self.bytes.next();
        }
        Ok(PdfObject::HexString(PdfHexString::of(&hex)))
    }

    fn parse_literal_string(&mut self) -> Result<PdfObject> {
        self.bytes.next(); // skip (
        let mut value = String::new();
        let mut depth = 1;
        let mut escaped = false;

        while !self.bytes.done() && depth > 0 {
            let b = self.bytes.next().unwrap();

            if escaped {
                value.push(b as char);
                escaped = false;
                continue;
            }

            match b {
                CharCodes::BackSlash => {
                    value.push(b as char);
                    escaped = true;
                }
                CharCodes::LeftParen => {
                    depth += 1;
                    value.push(b as char);
                }
                CharCodes::RightParen => {
                    depth -= 1;
                    if depth > 0 {
                        value.push(b as char);
                    }
                }
                _ => {
                    value.push(b as char);
                }
            }
        }

        Ok(PdfObject::String(PdfString::of(&value)))
    }

    fn parse_array(&mut self) -> Result<PdfObject> {
        self.bytes.next(); // skip [
        let mut array = PdfArray::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.bytes.done() {
                break;
            }
            if self.bytes.peek() == Some(CharCodes::RightSquareBracket) {
                self.bytes.next();
                break;
            }
            let obj = self.parse_object()?;
            array.push(obj);
        }

        Ok(PdfObject::Array(array))
    }

    fn parse_dict(&mut self) -> Result<PdfObject> {
        self.bytes.next(); // skip <
        self.bytes.next(); // skip <
        let mut dict = PdfDict::new();

        loop {
            self.skip_whitespace_and_comments();
            if self.bytes.done() {
                break;
            }
            // Check for >>
            if self.bytes.peek() == Some(CharCodes::GreaterThan)
                && self.bytes.peek_ahead(1) == Some(CharCodes::GreaterThan)
            {
                self.bytes.next();
                self.bytes.next();
                break;
            }

            // Parse key (must be a name)
            let key_obj = self.parse_object()?;
            let key = match key_obj {
                PdfObject::Name(n) => n,
                _ => continue, // skip invalid key
            };

            self.skip_whitespace_and_comments();

            // Parse value
            if self.bytes.done() {
                break;
            }
            // Check if we're at >> (missing value)
            if self.bytes.peek() == Some(CharCodes::GreaterThan)
                && self.bytes.peek_ahead(1) == Some(CharCodes::GreaterThan)
            {
                break;
            }
            let value = self.parse_object()?;
            dict.set(key, value);
        }

        Ok(PdfObject::Dict(dict))
    }

    fn parse_number_or_ref(&mut self) -> Result<PdfObject> {
        let start = self.bytes.offset();
        let number = self.parse_raw_number()?;

        // Check if this is a reference: <int> <int> R
        let after_num = self.bytes.offset();
        self.skip_whitespace();

        if let Some(gen) = self.try_parse_int() {
            self.skip_whitespace();
            if self.bytes.peek() == Some(CharCodes::UpperR) {
                self.bytes.next();
                return Ok(PdfObject::Ref(PdfRef::of(
                    number as u32,
                    gen as u16,
                )));
            }
            // Not a ref, restore position
            self.bytes.move_to(after_num);
        } else {
            self.bytes.move_to(after_num);
        }

        // It's just a number, but we already consumed it
        // Need to check if it was actually an int
        let _ = start;
        Ok(PdfObject::Number(PdfNumber::of(number)))
    }

    fn parse_raw_int(&mut self) -> Result<i64> {
        let mut value = String::new();
        while !self.bytes.done() {
            let b = self.bytes.peek().unwrap();
            if !is_digit(b) {
                break;
            }
            value.push(self.bytes.next().unwrap() as char);
        }
        if value.is_empty() {
            let pos = self.bytes.position();
            return Err(PdfError::InvalidObjectParsing {
                line: pos.line,
                column: pos.column,
                offset: pos.offset,
            });
        }
        value.parse::<i64>().map_err(|_| {
            let pos = self.bytes.position();
            PdfError::InvalidObjectParsing {
                line: pos.line,
                column: pos.column,
                offset: pos.offset,
            }
        })
    }

    fn try_parse_int(&mut self) -> Option<i64> {
        let start = self.bytes.offset();
        let mut value = String::new();
        while !self.bytes.done() {
            let b = self.bytes.peek().unwrap();
            if !is_digit(b) {
                break;
            }
            value.push(self.bytes.next().unwrap() as char);
        }
        if value.is_empty() {
            self.bytes.move_to(start);
            return None;
        }
        match value.parse::<i64>() {
            Ok(n) => Some(n),
            Err(_) => {
                self.bytes.move_to(start);
                None
            }
        }
    }

    fn parse_raw_number(&mut self) -> Result<f64> {
        let mut value = String::new();

        // Parse sign and integer part
        while !self.bytes.done() {
            let b = self.bytes.peek().unwrap();
            if is_digit(b) || b == CharCodes::Plus || b == CharCodes::Minus || b == CharCodes::Period {
                value.push(self.bytes.next().unwrap() as char);
                if b == CharCodes::Period {
                    break;
                }
            } else {
                break;
            }
        }

        // Parse decimal part
        while !self.bytes.done() {
            let b = self.bytes.peek().unwrap();
            if !is_digit(b) {
                break;
            }
            value.push(self.bytes.next().unwrap() as char);
        }

        if value.is_empty() || value == "." || value == "+" || value == "-" {
            let pos = self.bytes.position();
            return Err(PdfError::InvalidObjectParsing {
                line: pos.line,
                column: pos.column,
                offset: pos.offset,
            });
        }

        value.parse::<f64>().map_err(|_| {
            let pos = self.bytes.position();
            PdfError::InvalidObjectParsing {
                line: pos.line,
                column: pos.column,
                offset: pos.offset,
            }
        })
    }

    fn expect_byte(&mut self, expected: u8) -> Result<()> {
        match self.bytes.next() {
            Some(b) if b == expected => Ok(()),
            _ => {
                let pos = self.bytes.position();
                Err(PdfError::InvalidObjectParsing {
                    line: pos.line,
                    column: pos.column,
                    offset: pos.offset,
                })
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.bytes.done() {
            if let Some(b) = self.bytes.peek() {
                if is_whitespace(b) {
                    self.bytes.next();
                } else {
                    break;
                }
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            if self.bytes.peek() == Some(CharCodes::Percent) {
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while !self.bytes.done() {
            let b = self.bytes.peek().unwrap();
            if b == CharCodes::Newline || b == CharCodes::CarriageReturn {
                return;
            }
            self.bytes.next();
        }
    }

    fn matches_keyword(&self, keyword: &[u8]) -> bool {
        let remaining = self.bytes.remaining();
        if remaining.len() < keyword.len() {
            return false;
        }
        &remaining[..keyword.len()] == keyword
    }

    fn skip_keyword(&mut self, keyword: &[u8]) {
        for _ in 0..keyword.len() {
            self.bytes.next();
        }
    }

    fn skip_to_endobj(&mut self) {
        while !self.bytes.done() {
            if self.matches_keyword(b"endobj") {
                self.skip_keyword(b"endobj");
                return;
            }
            self.bytes.next();
        }
    }

    fn skip_xref_section(&mut self) {
        self.skip_keyword(b"xref");
        self.skip_whitespace();

        // Skip subsections
        while !self.bytes.done() {
            if self.matches_keyword(b"trailer")
                || self.matches_keyword(b"startxref")
                || self.matches_keyword(b"%%EOF")
            {
                break;
            }
            // Try to parse subsection header: first_obj count
            if let Some(_first_obj) = self.try_parse_int() {
                self.skip_whitespace();
                if let Some(count) = self.try_parse_int() {
                    self.skip_whitespace();
                    // Skip count entries (each 20 bytes but we'll just scan lines)
                    for _ in 0..count {
                        self.skip_line();
                        self.skip_whitespace();
                    }
                }
            } else {
                self.bytes.next();
            }
        }
    }

    fn skip_line(&mut self) {
        while !self.bytes.done() {
            let b = self.bytes.peek().unwrap();
            self.bytes.next();
            if b == CharCodes::Newline || b == CharCodes::CarriageReturn {
                return;
            }
        }
    }

    fn skip_trailer(&mut self) {
        self.skip_keyword(b"trailer");
        self.skip_whitespace_and_comments();

        // Parse trailer dictionary
        if let Ok(PdfObject::Dict(dict)) = self.parse_object() {
            // Extract trailer info
            if let Some(root) = dict.get(&PdfName::of("Root")) {
                self.context.trailer_info.root = Some(root.clone());
            }
            if let Some(encrypt) = dict.get(&PdfName::of("Encrypt")) {
                self.context.trailer_info.encrypt = Some(encrypt.clone());
            }
            if let Some(info) = dict.get(&PdfName::of("Info")) {
                self.context.trailer_info.info = Some(info.clone());
            }
            if let Some(id) = dict.get(&PdfName::of("ID")) {
                self.context.trailer_info.id = Some(id.clone());
            }
        }
    }

    fn extract_trailer_info_from_dict(&mut self, dict: &PdfDict) {
        if self.context.trailer_info.root.is_none() {
            if let Some(root) = dict.get(&PdfName::of("Root")) {
                self.context.trailer_info.root = Some(root.clone());
            }
        }
        if self.context.trailer_info.encrypt.is_none() {
            if let Some(encrypt) = dict.get(&PdfName::of("Encrypt")) {
                self.context.trailer_info.encrypt = Some(encrypt.clone());
            }
        }
        if self.context.trailer_info.info.is_none() {
            if let Some(info) = dict.get(&PdfName::of("Info")) {
                self.context.trailer_info.info = Some(info.clone());
            }
        }
        if self.context.trailer_info.id.is_none() {
            if let Some(id) = dict.get(&PdfName::of("ID")) {
                self.context.trailer_info.id = Some(id.clone());
            }
        }
    }

    fn decompress_stream(&self, stream: &PdfRawStream) -> Option<Vec<u8>> {
        let filter = stream.dict.get(&PdfName::of("Filter"));
        match filter {
            Some(PdfObject::Name(n)) if n.as_string() == "/FlateDecode" => {
                let mut decoder = ZlibDecoder::new(&stream.contents[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).ok()?;
                Some(decompressed)
            }
            None => Some(stream.contents.clone()),
            _ => None, // Unsupported filter
        }
    }

    fn parse_object_stream(&mut self, stream: &PdfRawStream) {
        let n = match stream.dict.get(&PdfName::of("N")) {
            Some(PdfObject::Number(n)) => n.as_number() as usize,
            _ => return,
        };
        let first = match stream.dict.get(&PdfName::of("First")) {
            Some(PdfObject::Number(n)) => n.as_number() as usize,
            _ => return,
        };

        let decompressed = match self.decompress_stream(stream) {
            Some(d) => d,
            None => return,
        };

        // Parse the header: N pairs of (obj_number, offset)
        let header_bytes = &decompressed[..first.min(decompressed.len())];
        let header_str = String::from_utf8_lossy(header_bytes);
        let nums: Vec<usize> = header_str
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if nums.len() < n * 2 {
            return;
        }

        let mut obj_entries = Vec::with_capacity(n);
        for i in 0..n {
            let obj_num = nums[i * 2] as u32;
            let offset = nums[i * 2 + 1];
            obj_entries.push((obj_num, first + offset));
        }

        // Parse each object from the decompressed data
        for (obj_num, offset) in obj_entries {
            if offset >= decompressed.len() {
                continue;
            }
            let sub_parser = PdfParser::for_bytes(&decompressed[offset..]);
            if let Ok(obj) = sub_parser.parse_single_object() {
                let pdf_ref = PdfRef::of(obj_num, 0);
                if self.context.lookup(&pdf_ref).is_none() {
                    self.context.assign(&pdf_ref, obj);
                }
            }
        }
    }

    fn skip_startxref(&mut self) {
        self.skip_keyword(b"startxref");
        self.skip_whitespace();
        let _ = self.try_parse_int();
    }

    fn skip_junk_after_eof(&mut self) {
        // Skip anything after %%EOF until we find another PDF structure
        while !self.bytes.done() {
            self.skip_whitespace();
            if self.bytes.done() {
                break;
            }
            if self.matches_keyword(b"%PDF-")
                || self.matches_keyword(b"xref")
                || self.matches_keyword(b"trailer")
                || self.matches_keyword(b"startxref")
            {
                break;
            }
            // Check if it looks like an indirect object (digit at start of line)
            if let Some(b) = self.bytes.peek() {
                if is_digit(b) {
                    break;
                }
            }
            self.bytes.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throws_error_when_pdf_missing_header() {
        let input = b"I_AM_NOT_A_HEADER\n1 0 obj\n(foobar)\nendobj\n";
        let parser = PdfParser::for_bytes(input);
        assert!(parser.parse_document().is_err());
    }

    #[test]
    fn does_not_throw_when_endobj_missing() {
        let input = b"%PDF-1.7\n1 0 obj\n(foobar)\nfoo\n";
        let parser = PdfParser::for_bytes(input);
        let context = parser.parse_document().unwrap();
        assert!(context.lookup(&PdfRef::of(1, 0)).is_some());
        if let Some(PdfObject::String(s)) = context.lookup(&PdfRef::of(1, 0)) {
            assert_eq!(s.as_string(), "foobar");
        }
    }

    #[test]
    fn handles_invalid_binary_comments_after_header() {
        let mut input = Vec::new();
        input.extend_from_slice(b"%PDF-1.7\n");
        input.extend_from_slice(&[128, 1, 2, 3, 4, 5, 129, 130, 131, CharCodes::Newline]);
        input.extend_from_slice(b"1 0 obj\n(foobar)\nendobj");
        let parser = PdfParser::for_bytes(&input);
        let context = parser.parse_document().unwrap();
        assert_eq!(context.enumerate_indirect_objects().len(), 1);
    }

    #[test]
    fn parses_basic_objects() {
        let input = b"%PDF-1.7\n\
            1 0 obj\n42\nendobj\n\
            2 0 obj\n/Foo\nendobj\n\
            3 0 obj\n(Hello)\nendobj\n\
            4 0 obj\ntrue\nendobj\n\
            5 0 obj\nnull\nendobj\n\
            6 0 obj\n<48656C6C6F>\nendobj\n\
            7 0 obj\n[1 2 3]\nendobj\n\
            8 0 obj\n<< /Type /Page >>\nendobj\n";
        let parser = PdfParser::for_bytes(input);
        let context = parser.parse_document().unwrap();
        assert_eq!(context.enumerate_indirect_objects().len(), 8);
    }

    #[test]
    fn parses_indirect_references() {
        let input = b"%PDF-1.7\n\
            1 0 obj\n<< /Ref 2 0 R >>\nendobj\n\
            2 0 obj\n42\nendobj\n";
        let parser = PdfParser::for_bytes(input);
        let context = parser.parse_document().unwrap();
        assert_eq!(context.enumerate_indirect_objects().len(), 2);
    }

    #[test]
    fn parses_pdf_with_xref_and_trailer() {
        let input = b"%PDF-1.7\n\
            1 0 obj\n(foobar)\nendobj\n\
            xref\n0 2\n\
            0000000000 65535 f \n\
            0000000009 00000 n \n\
            trailer\n<< /Size 2 /Root 1 0 R >>\n\
            startxref\n34\n%%EOF\n";
        let parser = PdfParser::for_bytes(input);
        let context = parser.parse_document().unwrap();
        assert_eq!(context.enumerate_indirect_objects().len(), 1);
    }

    #[test]
    fn does_not_stall_with_junk_after_eof() {
        let input = b"%PDF-1.7\n\
            1 0 obj\n(foobar)\nendobj\n\
            startxref\n127\n%%EOF\n\
            @@@@@@@@@@@@@@@@@@\n";
        let parser = PdfParser::for_bytes(input);
        let context = parser.parse_document().unwrap();
        assert_eq!(context.enumerate_indirect_objects().len(), 1);
    }

    #[test]
    fn parses_streams() {
        let input = b"%PDF-1.7\n\
            1 0 obj\n<< /Length 4 >>\nstream\ntest\nendstream\nendobj\n";
        let parser = PdfParser::for_bytes(input);
        let context = parser.parse_document().unwrap();
        if let Some(PdfObject::Stream(stream)) = context.lookup(&PdfRef::of(1, 0)) {
            assert_eq!(stream.contents, b"test");
        } else {
            panic!("Expected stream object");
        }
    }

    #[test]
    fn loads_real_pdf() {
        let pdf_bytes = std::fs::read("test_assets/pdfs/normal.pdf").unwrap();
        let parser = PdfParser::for_bytes(&pdf_bytes);
        let context = parser.parse_document().unwrap();
        assert!(context.object_count() > 0);
    }
}
