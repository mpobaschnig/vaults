# AGENTS.md

This file provides guidance for AI assistants working on the **Vaults** codebase.

## Project Overview

Vaults is a free and open-source GUI application for creating and managing encrypted file vaults on Linux. It provides a
user-friendly interface built with GTK4 and Libadwaita, supporting multiple encryption backends.

The application is distributed via Flathub but native deployment is also supported. Meson is used as its build system.

## Source Layout

```
src/
├── backend/          Backend abstraction (mod.rs) with the CryFS and gocryptfs implementations
├── ui/               GTK widget code; templates live in data/resources/ui/*.ui
│   └── pages/        Widgets for the vault list rows and their prompt/settings windows
├── legacy/           Migration of the old configuration format
├── application.rs    Adw::Application subclass, actions and window setup
├── vault.rs          Vault object
├── global_config_manager.rs Global configuration settings handling
├── user_config_manager.rs User vaults configuration handling
├── config.rs(.in)    Build-time generated runtime configuration
└── main.rs           Process entry point
data/
├── resources/ui/    GTK template (.ui) files used with #[derive(CompositeTemplate)]
└── resources.gresource.xml
build-aux/            Cargo build glue, Flatpak backend wrapper scripts, patches, pre-commit.hook
po/                   Translations
```

New UI template files must be registered in `data/resources.gresource.xml`; new source files containing
translatable strings must be added to `po/POTFILES.in`.

## Building

The supported way of building the Flatpak is via GNOME Builder (or VS Code with the Flatpak extension) using the
`io.github.mpobaschnig.Vaults.Devel.json` manifest.

### Flatpak

Building the Flatpak from the command line requires `flatpak` and `flatpak-builder`. Install the runtime, SDK and
required SDK extensions first:

```bash
flatpak install flathub org.gnome.Sdk//50 org.freedesktop.Sdk.Extension.rust-stable org.freedesktop.Sdk.Extension.llvm21
```

Then build and install the application from the repository root, using the manifest
`io.github.mpobaschnig.Vaults.Devel.json`:

```bash
flatpak-builder --force-clean --install-deps-from=flathub --user --install build io.github.mpobaschnig.Vaults.Devel.json
```

The application can be run afterwards with:

```bash
flatpak run io.github.mpobaschnig.Vaults.Devel
```

During development, simply omit `--force-clean`: flatpak-builder keeps the build directory and only rebuilds changed
modules by default.

### Native Deployment

For a native build, the following applications and libraries are required:

* cargo >= 1.92 (Rust edition 2024)
* Meson >= 1.5.0
* GLib >= 2.66
* GTK4 >= 4.16
* libadwaita >= 1.6
* `gtk4-devel` / `libadwaita-devel` (or the distribution equivalents) to compile and run tests

To run vaults at runtime, the backend binaries `cryfs` and `gocryptfs` as well as FUSE must be installed separately.
Inside the Flatpak these backends are bundled and invoked through the wrapper scripts in `build-aux/`
(`cryfs-wrapper.sh`, `gocryptfs-wrapper.sh`, `fusermount-wrapper.sh`, etc.).

Set up a build directory and compile the application. The `development` profile produces a debug build with the `Devel`
application ID, while the `default` profile produces a release build:

```bash
meson setup build -Dprofile=development
ninja -C build
```

The resulting binary is at `build/target/debug/vaults` for the development profile, or
`build/target/release/vaults` for the default profile, and can be run directly.

To install the application system-wide, use a prefix matching your distribution (e.g. `/usr`) and run:

```bash
meson setup build --prefix=/usr
ninja -C build
sudo meson install -C build
```

The install step also compiles the GSettings schemas and updates the icon and desktop caches. Afterwards the application
can be launched with `vaults` or from the applications menu.

## Testing Instructions

CI (`.github/workflows/quality.yml`) runs these commands; all three must pass with zero warnings:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

- Run unit tests with `cargo test` from the repository root. This requires the GTK4 and libadwaita development
  packages to be installed.

## Code Style Guidelines

- All code should be licensed under GPL-3.0-or-later.
- Never write PII including passwords in debug logs.
- Use `log` crate for logging; `pretty_env_logger` for development.
- `toml` is used as a configuration format.
- Always add documentation comments to public functions and methods, include a brief description of the function.
  Describe parameters and return value.
- Only add documentation and comments within functions if the intention is not immediately clear.
- Keep functions short and perform a single task.
- Avoid code duplication and strive for code readability.

## Git Conventions

Development happens on the branches with the name specifying its intention; pull requests target `main` branch.

Commit messages should follow the following guidelines.

### Structure

Commit messages use the following structure:

* A brief description that summarizes the commit
* The main description (body)
* Links or references to related issues (optional)

Example:

    [optional tag]: Short explanation of the commit

    Longer explanation explaining exactly what's changed and why, whether
    any external or private interfaces changed, what issue were fixed (with
    issue number if applicable) and so forth. Be concise but not too brief.

    Closes #1234

### Summary line

* A summary of the change itself, not the issue or effects/benefits (that goes in the description).
* Always a single sentence with no trailing period (full stop).
* Be sure to not exceed 72 characters; you can use ‘&’ instead of ‘and’ so the message gets shorter.
* Uses imperative present tense. For example use “Add Hindi translation” instead of “Added Hindi translation”.
* You can prefix the first line with one tag, to make it easier to know to which part of the module the commit applies.
  For example, a commit with “sound: Fix label alignments” in the Settings module clearly applies to the sound panel.

### Main description

* A detailed description of the change and the reasoning behind it. Consider the commit message as an email sent to the
  developers (or yourself, six months in the future) detailing why you changed something. There’s no need to specify the
  how: those who need to know can view the code changes alongside the commit message.
* Each line must not exceed 72 characters. There is no limit on the number of lines.
* Uses normal prose, punctuation, and capitalization.
* If you use git revert command to undo a commit, git generates a default commit message specifying the hash and commit
  message of the commit you’re undoing. However, this is not enough to know why you’re reverting it, so please add more
  comments to it.
