# Setup Environment

## Prerequisites

To work with the UQEntropy project, you'll need the following tools installed:

1. **Rust Toolchain**: The project is written in Rust, so you'll need to install the Rust compiler and Cargo package manager.
2. **Make**: For running the provided Makefile.
3. **Standard Unix Tools**: Such as bash, cat, etc.

## Installing Rust

The easiest way to install Rust is through rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to complete the installation. After installation, you should have access to both `rustc` (the Rust compiler) and `cargo` (the package manager and build tool).

Verify the installation:

```bash
rustc --version
cargo --version
```

## Project Structure Setup

Clone or download the project to your local machine. The project structure should look like this:

```
.
├── src/bin/uqentropy.rs
├── Cargo.toml
├── Makefile
├── README.md
├── testa1_rust.sh
└── testfiles/
```

## Dependencies

The project depends on the following crates (libraries):

1. `log` - Logging facade
2. `env_logger` - Logger implementation
3. `chrono` - Date and time handling

These dependencies are automatically managed by Cargo and specified in the `Cargo.toml` file. They will be downloaded and compiled when you build the project.

## Building the Project

You can build the project in two ways:

1. Using Cargo directly:
   ```bash
   cargo build
   ```

2. Using the provided Makefile:
   ```bash
   make
   ```

Both methods will compile the binary to `target/debug/uqentropy` and copy it to the current directory.