# slpkg
Unpacker for Esri Scene Layer Package (.slpk) files

# Description
An Esri Scene Layer Package (slpk) file is a zipped archive containing an Indexed 3D Scene (I3S). The specification for both of these is found [here](https://github.com/Esri/i3s-spec). Typically, the package as a whole is not created with any compression. However, each file in the package will be individually gzipped. This allows the files in the package to be served as-is by an HTTP server, instead of being decompressed during extraction from the zip, and then recompressed for the HTTP transfer.

This tool will unpack a scene layer package, allowing users to inspect the package contents. This includes unpacking the zip archive, as well as decompressing any files whose name ends with the `.gz` extension.

# Usage

`slpkg unpack [--verbose] <src_file>`

At the moment, the only allowable sub-command is `unpack`. In the future this tool may be extended to allow repacking a folder into a .slpk package.

By default the program produces no output, except in the case of errors. The `--verbose` flag can be used to have the program log a message for each file extracted from the scene layer package.

# License

This program is licenced under the terms of the BSD-2-Clause license.
