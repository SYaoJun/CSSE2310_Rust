# Building Process

## Build Methods

The UQEntropy project provides two ways to build the application:

### 1. Using Cargo

The standard Rust way to build the project is with Cargo:

```bash
cargo build
```

This command will:
1. Download and compile dependencies
2. Compile the source code
3. Place the executable in `target/debug/uqentropy`

For release builds with optimizations:

```bash
cargo build --release
```

This places the optimized executable in `target/release/uqentropy`.

### 2. Using Make

The project includes a Makefile that simplifies the build process:

```bash
make
```

The Makefile performs these steps:
1. Runs `cargo build --debug`
2. Copies the executable from `target/debug/uqentropy` to the current directory

To clean built files:

```bash
make clean
```

This removes the copied executable from the current directory.

## Build Dependencies

When building the project, Cargo automatically handles dependencies specified in `Cargo.toml`:

- `log = "0.4"`: Logging facade
- `env_logger = "0.9"`: Logger implementation
- `chrono = { version = "0.4", features = ["serde"] }`: Date/time handling for log files

These dependencies will be downloaded from crates.io (the Rust package registry) if not already present in the local cache.

## Binary Generation

After a successful build, you'll have the `uqentropy` executable which can be run directly:

```bash
./uqentropy [OPTIONS] [FILES...]
```

## Cross-platform Compatibility

As a Rust application, UQEntropy can be compiled for different platforms supported by the Rust toolchain. The project doesn't appear to use platform-specific APIs, suggesting good cross-platform compatibility.