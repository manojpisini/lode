# dlltool-stub

A minimal drop-in replacement for MSYS2's `dlltool.exe` used when building LODE
with the `stable-x86_64-pc-windows-gnu` Rust toolchain.

## Purpose

The GNU ABI Rust toolchain links against native libraries (e.g. SQLite via
`libsqlite3-sys` or PCRE2 via `libpcre2-sys`) that require Windows import
libraries (`.a` files). Normally `dlltool` from MSYS2 or LLVM generates these.
This stub:

1. Looks in the Rust toolchain directory for the real import library.
2. Copies it to the output path if found.
3. Fails loudly with a clear error message if the library is missing — never
   creates an empty archive that would fail silently at link time.

## When to use

- Building with `stable-x86_64-pc-windows-gnu` toolchain.
- MSYS2 `dlltool` is not in `PATH`.
- All required import libraries are already bundled in the rust toolchain.

## When to remove

Switch to the `stable-x86_64-pc-windows-msvc` toolchain or install MSYS2
ucrt64 with `mingw-w64-ucrt-x86_64-dlltool` to use the real `dlltool`.
