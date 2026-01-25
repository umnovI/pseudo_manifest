# Pseudo Manifest

A Rust-based CLI utility designed to automate the creation of **local Scoop manifests** for your Rust projects. This tool streamlines the process of installing your own compiled binaries via the Scoop package manager by generating the necessary JSON manifest automatically from your `Cargo.toml`.

---

## 🚀 Features

* **Automatic Metadata Extraction**: Pulls version, name, and license directly from your `Cargo.toml`.
* **Hash Generation**: Automatically calculates the SHA256 hash of your release executable.
* **Smart Updates**: Compares existing manifests with new builds; it only updates if the hash has changed and ensures the version is incremented.
* **GUI & CLI Support**: Configure shims for CLI tools or shortcuts for GUI applications.
* **Architecture Aware**: Specifically generates manifests for 64-bit Windows environments.

---

## 🛠 Installation

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (cargo)
* [Scoop](https://scoop.sh/)
* [Task](https://taskfile.dev/) (optional, for automation)

### Build from source

```bash
git clone https://github.com/umnovI/pseudo_manifest
cd pseudo_manifest
cargo build --release

```

---

## 📖 Usage

The tool is intended to be run from within your Rust project's root directory.

### Command Line Arguments

| Argument | Long Flag | Description |
| --- | --- | --- |
| `cwd` | `--cwd` | **Required**. Path to your project's root directory. |
| `alias` | `--alias` | **Required**. The command name (shim) or shortcut name for Scoop. |
| `file` | `--file` | Path to a specific executable (optional). |
| `bin` | `--bin` | The name of the installed `.exe` (optional). |
| `gui` | `--gui` | Switch to GUI mode (creates a shortcut instead of a shim). |
| `debug` | `--debug` | Creates the manifest in the `cwd` instead of the Scoop bucket. |

### Example

```bash
# Generate a manifest for a CLI tool named "my-tool"
pseudo_manifest --cwd . --alias my-tool

```

---

## 🤖 Automation with Taskfile

This project includes a `Taskfile.yml` to simplify your development workflow:

* **Build & Manifest**: Run `task rwm` to build your project in release mode and immediately update/create its Scoop manifest.
* **Installation**: Run `task scoop-install` to install the generated manifest via Scoop from your local bucket.

---

## 📂 How it Works

1. **Reads `Cargo.toml`**: Gathers project identity.
2. **Locates Binary**: Looks into `target/release/` for the `.exe`.
3. **Calculates SHA256**: Ensures Scoop can verify the binary integrity.
4. **Validates Versioning**: If a manifest already exists in your local Scoop bucket (`~/scoop/buckets/local`), the tool checks if the hash is different. If the hash changed but the version didn't, it will prompt you to update your version.
5. **Writes JSON**: Produces a Scoop-compatible JSON file.

---

## 📄 License

This project is licensed under the same terms as the code it processes (detected via `Cargo.toml`), or defaults to the license specified in the repository.
