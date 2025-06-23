# Typst-FFI: Rust Static Library for Typst demo

This project converts the [`typst-example`](https://github.com/bingqiao/typst-example) project into a static library (`typst-ffi`) that compiles Typst markup into PDF files, suitable for integration with a macOS [SwiftUI app](https://github.com/bingqiao/TypstFFIApp). The library exposes a C-compatible FFI (Foreign Function Interface) and generates a static library (`libtypst_ffi.a`) and a C header file (`typst_ffi.h`) for use in macOS applications.

## Conversion from `typst-example`

The original `typst-example` was a standalone Rust project that compiled Typst markup into a PDF. The following changes were made to convert it into a static library:

1. **Modified `Cargo.toml`**:
   - Changed the crate type to `staticlib` to produce a static library:
     ```toml
     [lib]
     crate-type = ["staticlib"]
     ```
   - Updated the `[package]` section to reflect the new project name (`typst-ffi`) and edition (`2024`).
   - Added `cbindgen` as a build dependency to generate the C header file:
     ```toml
     [build-dependencies]
     cbindgen = "0.29"
     ```

2. **Updated `lib.rs`**:
   - Replaced the standalone `main` function with FFI-compatible functions: `compile_typst` and `free_typst_buffer`.
   - Implemented `compile_typst` to take a C string input, compile it to a PDF using the Typst library, and return a pointer to the PDF data along with its length.
   - Added `free_typst_buffer` to safely deallocate memory allocated by `compile_typst`.
   - Retained the `SimpleWorld` struct implementing the `World` trait for Typst compilation, embedding the `NotoSans-Regular.ttf` font via `include_bytes!`.

3. **FFI Interface**:
   - Functions are marked with `#[no_mangle]` and `extern "C"` to ensure C-compatible symbols.
   - Input is a C string (`*const c_char`), and output is a raw pointer (`*mut u8`) with a length (`size_t`).
   - Error handling returns a null pointer for failures, with the output length set to 0.

4. **Font Embedding**:
   - The `NotoSans-Regular.ttf` font is embedded in `lib.rs` to ensure consistent PDF rendering without external dependencies.

## Building the Library and Header Files

### Prerequisites
- **Rust**: Install Rust via `rustup` (https://rustup.rs/).
- **cbindgen**: Install via `cargo install cbindgen`.
- **macOS**: Ensure you’re on an Apple Silicon Mac (arm64) or adjust for Intel (x86_64) if needed.

### Build Steps
1. **Build the Static Library**:
   - For arm64:
     ```bash
     cargo build --release
     ```
     This generates `target/release/libtypst_ffi.a`.

2. **Generate the C Header File**:
   - Run `cbindgen` to create `typst_ffi.h`:
     ```bash
     cbindgen --lang c --output typst_ffi.h
     ```
     N.B. `--lang c` is critical. Without it, a C++ header will be generated which doesn't work straightaway with SwiftUI project.
     The header file defines the FFI functions:
     ```c
     unsigned char *compile_typst(const char *input, size_t *output_len);
     void free_typst_buffer(unsigned char *ptr);
     ```

5. **Copy Files for macOS App**:
   - Copy the library and header to your macOS project directory (e.g., `~/TypstFFIApp`):
     ```bash
     cp target/release/libtypst_ffi.a ~/TypstFFIApp/
     cp typst_ffi.h ~/TypstFFIApp/
     ```

## Usage
- The library is designed to be linked into a macOS app (e.g., a SwiftUI app).
- Include `typst_ffi.h` in a bridging header to call `compile_typst` and `free_typst_buffer` from Swift.
- See the macOS app’s `README.md` for integration details.

## Notes
- Ensure the `NotoSans-Regular.ttf` font file is available in the `fonts/` directory for embedding.

# Testing
A binary `main.rs` is created to call the static library for testing purpose.
```bash
cargo run --bin test-typst-ffi
```
