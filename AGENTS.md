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

The test suite consists of two types of tests:

1. **Pure logic tests** — Test backend logic, configuration handling, utilities, and vault operations. These run without any display server.
2. **UI tests** — Test GTK4 widget signals and state using `#[gtk::test]` macro. These require a headless Wayland display server (Weston).

### CI Checks

CI (`.github/workflows/quality.yml`) runs these commands; all three must pass with zero warnings:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

### Running Tests Locally

#### Prerequisites

Install the Weston headless compositor:

```bash
# Fedora
sudo dnf install -y weston

# Arch Linux
sudo pacman -S weston

# Debian/Ubuntu
sudo apt install weston
```

GTK4 and libadwaita development packages are also required to compile the tests.

#### Full Test Suite (with UI tests)

Start the Weston headless compositor, then run all tests:

```bash
# Start Weston headless
mkdir -p /tmp/weston-runtime
chmod 700 /tmp/weston-runtime
XDG_RUNTIME_DIR=/tmp/weston-runtime weston --backend=headless --renderer=pixman --no-config &

# Wait for the Wayland socket to appear
for i in $(seq 1 30); do
  [ -S /tmp/weston-runtime/wayland-0 ] && break
  [ -S /tmp/weston-runtime/wayland-1 ] && break
  sleep 1
done

# Run all tests
export XDG_RUNTIME_DIR=/tmp/weston-runtime
export WAYLAND_DISPLAY=wayland-0  # or wayland-1
export GDK_BACKEND=wayland
export XDG_SESSION_TYPE=wayland
cargo test

# Stop Weston when done
pkill weston
```

**Important:** The Wayland socket may be `wayland-0` or `wayland-1` depending on the system. The `test_utils::check_wayland_display()` function verifies the socket exists before each UI test and provides setup instructions if it's missing.

#### Pure Logic Tests Only (without Weston)

To run only tests that don't require a display server:

```bash
cargo test backend
cargo test configuration
cargo test util
cargo test vault
```

### UI Test Infrastructure

- UI tests use the `#[gtk::test]` macro from GTK4, which runs tests serially on the GTK main thread.
- The `src/test_utils.rs` module provides `init_gtk()` for test initialization (GTK init, type registration, and resource loading).
- UI tests are co-located with their source files in `#[cfg(test)] mod tests` blocks.
- Tests use `Backend::Mock` vaults to avoid depending on real encryption backends.
- Widget state is verified through public accessor methods (e.g., `locker_button_icon_name()`, `is_selected()`).
- Signal tests connect to custom signals, emit actions programmatically, and assert the callbacks were invoked.

### CI Workflow

The CI workflow automatically sets up Weston headless in the Fedora 43 container:
1. Installs the `weston` package
2. Creates `XDG_RUNTIME_DIR` with proper permissions
3. Starts Weston with `--backend=headless --renderer=pixman --no-config`
4. Waits for the Wayland socket and auto-detects `wayland-0` or `wayland-1`
5. Sets `WAYLAND_DISPLAY`, `GDK_BACKEND=wayland`, and `XDG_SESSION_TYPE=wayland`
6. Runs `cargo test`
7. Cleans up Weston after tests

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
