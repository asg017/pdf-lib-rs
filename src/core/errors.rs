use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("This document is encrypted and cannot be processed")]
    EncryptedPdf,

    #[error("Invalid PDF date string: {0}")]
    InvalidPdfDateString(String),

    #[error("Method not implemented: {class_name}::{method_name}")]
    MethodNotImplemented {
        class_name: String,
        method_name: String,
    },

    #[error("PDF array is not a rectangle (expected 4 elements, got {0})")]
    PdfArrayIsNotRectangle(usize),

    #[error("Missing PDF header")]
    MissingPdfHeader,

    #[error("Missing keyword: {0}")]
    MissingKeyword(String),

    #[error("Parser stalled at position {line}:{column} (offset {offset})")]
    StalledParser {
        line: usize,
        column: usize,
        offset: usize,
    },

    #[error("Cannot reparse: {parser} already called {method}")]
    Reparse { parser: String, method: String },

    #[error("Invalid object at position {line}:{column} (offset {offset})")]
    InvalidObjectParsing {
        line: usize,
        column: usize,
        offset: usize,
    },

    #[error("Unexpected object type")]
    UnexpectedObjectType,

    #[error("Page embedding mismatched context")]
    PageEmbeddingMismatchedContext,
}

pub type Result<T> = std::result::Result<T, PdfError>;
