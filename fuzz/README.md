# LODE Fuzz Testing

This directory contains fuzz targets for LODE.

## Setup

```bash
cargo install cargo-fuzz
```

## Running

```bash
# Fuzz ValidatedRoot path resolution
cargo fuzz run validated_root

# Fuzz Process validation
cargo fuzz run process_validation
```

## Adding targets

Each target is defined in `fuzz/fuzz_targets/` as a separate `.rs` file.
