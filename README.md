# slpkg
Unpacker for Esri Scene Layer Package (.slpk) files

# Description
An Esri Scene Layer Package (slpk) file is a zipped archive containing an Indexed 3D Scene (I3S). The specification for both of these is found [here](https://github.com/Esri/i3s-spec). Typically, the package as a whole is not created with any compression. However, each file in the package will be individually gzipped. This allows the files in the package to be served as-is by an HTTP server, instead of being decompressed during extraction from the zip, and then recompressed for the HTTP transfer.

This tool will unpack a scene layer package, allowing users to inspect the package contents. This includes unpacking the zip archive, as well as decompressing any files whose name ends with the `.gz` extension.

# Usage

`slpkg unpack [--verbose] <slpk_file>`

At the moment, the only allowable sub-command is `unpack`. In the future this tool may be extended to allow repacking a folder into a .slpk package.

By default the program produces very little output, except in the case of errors. The `--verbose` flag can be used to have the program log a message for each file extracted from the scene layer package.

# License

This program is licenced under the terms of the BSD-2-Clause license.

# Building From Source

Building this utility from source is quite easy. The following instructions apply on all platforms (Windows/Linux/Mac).
1. Install Rust. The simplest way to get rust is with [rustup](https://rustup.rs/).
2. Clone a local copy of this repository.
3. Open a command prompt in the directory of your local respository
4. Run `cargo build --release` to build the program. You can run the program using `cargo run --release [--verbose] <slpk_file>`
5. To install the program, so that you don't need to use `cargo` to run it, use the `cargo install --path .` command. Once this is done, the program should be runnable from a command prompt in any directory, using the `slpkg` command.
