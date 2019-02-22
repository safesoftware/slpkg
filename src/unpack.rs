use failure::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;
use zip::ZipArchive;

fn open_slpk_archive(slpk_file_path: PathBuf) -> Result<ZipArchive<impl Read + Seek>,Error> {
    let file = File::open(slpk_file_path)?;
    let buf_reader = BufReader::new(file);
    return Ok(ZipArchive::new(buf_reader)?);
}

pub fn unpack(slpk_file_path: PathBuf) -> Result<(), Error> {
    let mut slpk_archive = open_slpk_archive(slpk_file_path)?;

    for i in 0..slpk_archive.len() {
        let file = slpk_archive.by_index(i)?;
        println!("{}", file.name());
    }

    Ok(())
}
