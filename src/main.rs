mod lib;
use lib::{compile_typst, free_typst_buffer};

use std::ffi::CString;
use std::fs::File;
use std::io::Write;

fn main() {
    // Sample Typst input
    let input = r#"
    #set page(width: 200pt, height: 200pt)
    Hello, *Typst* world from main.rs!
    "#;

    // Convert the input to a C-compatible string
    let c_input = match CString::new(input) {
        Ok(c_str) => c_str,
        Err(e) => {
            eprintln!("Failed to create CString: {}", e);
            return;
        }
    };

    // Call the compile_typst function
    let mut output_len: usize = 0;
    let pdf_ptr = unsafe { compile_typst(c_input.as_ptr(), &mut output_len) };

    if pdf_ptr.is_null() {
        eprintln!("Compilation failed: returned null pointer");
        return;
    }

    // Convert the output to a Rust slice
    let pdf_data = unsafe { std::slice::from_raw_parts(pdf_ptr, output_len) };

    // Save the PDF to a file
    let mut file = match File::create("output.pdf") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create output file: {}", e);
            free_typst_buffer(pdf_ptr);
            return;
        }
    };

    if let Err(e) = file.write_all(pdf_data) {
        eprintln!("Failed to write PDF data: {}", e);
    } else {
        println!(
            "PDF generated successfully: output.pdf ({} bytes)",
            output_len
        );
    }

    // Free the allocated buffer
    free_typst_buffer(pdf_ptr);
}
