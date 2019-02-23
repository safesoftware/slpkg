extern crate failure;
extern crate structopt;
extern crate zip;

use std::path::PathBuf;
use structopt::StructOpt;

mod unpack;

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

        #[structopt(short = "q", long = "quiet")]
        quiet: bool,
    },
}

fn main() {
    let params = Settings::from_args();
    match params {
        Settings::Unpack { src_file, quiet } => {
            if let Err(e) = unpack::unpack(src_file, quiet) {
                eprintln!("{}", e);
            }
        }
    }
}
