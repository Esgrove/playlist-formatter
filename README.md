# Playlist Tool

Helper tool for formatting DJ playlists exported from different DJ software.

Originally created for my own and fellow Bassoradio DJs use back when I was doing a radio show at Bassoradio.
The original version is written in Python and has both a PyQt6 GUI and CLI version.

Since then, I have added a Rust implementation, which is my preferred version currently.
See below for the details.

## Python version

Python version supports:

- csv playlists exported from Serato DJ Pro
- txt playlists exported from Rekordbox

### Python dependencies

- Python 3.11+ required (primarily due to use of `Self` type hinting)
- Poetry

```shell
poetry install
```

### Looks like

![gui](playlist_gui.png)

![cli](playlist_cli.png)

## Rust version

Rust CLI version supports:

- csv and txt playlists exported from Serato DJ Pro
- txt playlists exported from Rekordbox

> **Note**: Expects Finnish time and date formatting and might not work fully in case timestamps are in a different format

### Build

Using helper script, which will move the release executable to the repo root:

```shell
./build.sh
```

### Install

Install a release binary locally using [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html).

```shell
./install.sh
```

After this you should have `playfmt` available globally.

**Note:** Cargo will put the binary under `$HOME/.cargo/bin` by default,
which needs to be added to PATH so the binaries installed through Cargo will be found.

### Format Rust code

Using [rustfmt](https://github.com/rust-lang/rustfmt)

```shell
cargo fmt
```

### Lint Rust code

Using [Clippy](https://github.com/rust-lang/rust-clippy)

```shell
cargo clippy
cargo clippy --fix
```

### Update Rust dependencies

```shell
cargo update
```

## TODO

- Add unit tests
- Fix playtime calculations
