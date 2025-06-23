use libc::{c_char, size_t};
use std::ffi::CStr;
use std::ptr;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::{Library, World};
use typst_pdf::PdfOptions;
use typst_pdf::pdf;
use typst_utils::LazyHash;

struct SimpleWorld {
    source: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl World for SimpleWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> typst::diag::FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(typst::diag::FileError::NotFound(std::path::PathBuf::new()))
        }
    }

    fn file(&self, _id: FileId) -> typst::diag::FileResult<Bytes> {
        Err(typst::diag::FileError::NotFound(std::path::PathBuf::new()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compile_typst(input: *const c_char, output_len: *mut size_t) -> *mut u8 {
    // Check for null pointers
    if input.is_null() || output_len.is_null() {
        if !output_len.is_null() {
            unsafe {
                *output_len = 0;
            }
        }
        return ptr::null_mut();
    }

    // Convert C string to Rust string
    let c_str = unsafe { CStr::from_ptr(input) };
    let input_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            if !output_len.is_null() {
                unsafe {
                    *output_len = 0;
                }
            }
            return ptr::null_mut();
        }
    };

    let result = (|| {
        // Load Noto Sans font
        let font_data = include_bytes!("fonts/NotoSans-Regular.ttf");
        let font = Font::new(Bytes::new(font_data), 0).ok_or("Failed to load font")?;
        let font_book = FontBook::from_fonts([&font]);
        let fonts = vec![font];

        // Create a Typst source
        let file_id = FileId::new(None, VirtualPath::new("example.typ"));
        let source = Source::new(file_id, input_str.to_string());

        // Initialize the world
        let world = SimpleWorld {
            source,
            library: Library::default().into(),
            book: font_book.into(),
            fonts,
        };

        // Compile the source to a document
        let document = typst::compile(&world)
            .output
            .map_err(|e| format!("Compilation failed: {:?}", e))?;

        // Export to PDF
        let options = PdfOptions::default();
        pdf(&document, &options).map_err(|e| format!("PDF generation failed: {:?}", e))
    })();

    match result {
        Ok(pdf_data) => {
            let len = pdf_data.len();
            let ptr = unsafe { libc::malloc(len) as *mut u8 };
            if ptr.is_null() {
                if !output_len.is_null() {
                    unsafe {
                        *output_len = 0;
                    }
                }
                return ptr::null_mut();
            }
            unsafe {
                ptr.copy_from_nonoverlapping(pdf_data.as_ptr(), len);
                *output_len = len;
            }
            ptr
        }
        Err(_) => {
            if !output_len.is_null() {
                unsafe {
                    *output_len = 0;
                }
            }
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_typst_buffer(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            libc::free(ptr as *mut libc::c_void);
        }
    }
}
