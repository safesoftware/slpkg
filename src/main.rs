extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Settings {
    // Packing a directory into a .slpk file is not yet implemented.
    /*
    #[structopt(name = "pack")]
    Pack {
        #[structopt(parse(from_os_str))]
        src_dir: PathBuf
    },
    */
    /// Unpacks a .slpk file into a directory
    #[structopt(name = "unpack")]
    Unpack {
        /// The .slpk file which will be unpacked
        #[structopt(parse(from_os_str))]
        src_file: PathBuf,
    },
}

fn main() {
    let params = Settings::from_args();
    println!("{:?}", params);
}
