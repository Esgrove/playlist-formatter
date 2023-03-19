# Playlist Tool

Python program for auto-formatting DJ playlists exported from DJ software.
Originally created for my own and fellow Bassoradio DJs use back when I was doing a radio show at Bassoradio.
Has both a PyQt6 GUI and CLI version.

Used to process a raw playlist file exported from DJ softwares:
text is formatted and title cased properly and play times are calculated from timestamps,
and then exported back to a file.
The formatted playlist will work well for example with Mixcloud.

Currently supports:

- csv playlists exported from Serato DJ Pro
- txt playlists exported from Rekordbox

~~Also has the option to autofill the imported playlist to Bassoradio's database,
which saves me a lot of time and manual work when I don't have to input every song manually through the not so great web interface.
Implemented in a somewhat hacky way with _Selenium_, as I could not get it working directly with HTTP posts using the _requests_ package.~~

## Dependencies

- Python 3.11+ (due to use of `Self` type hinting :sweat_smile:)
- [requirements.txt](./requirements.txt)

## Looks like

![alt text](https://github.com/Esgrove/playlistTool/blob/master/playlistformatter.png)

## Rust version

Under development. CLI to begin with.

### Build

```shell
# debug
cargo build
cargo run
# release
cargo build --release
cargo run --release
```

Cargo will output the executable to either

```shell
rust/target/debug/vault
rust/target/release/vault
```

depending on which build profile is used.

### Install

You can install a release binary locally using [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html).
Note that you need to specify the path to the directory containing [Cargo.toml](/Cargo.toml):

```shell
cargo install --path .
```

Cargo will put the binary under `$HOME/.cargo/bin` by default,
which should be added to PATH so the binaries installed through Cargo will be found.

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
