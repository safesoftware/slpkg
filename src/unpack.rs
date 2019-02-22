use failure::Error;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;
use zip::ZipArchive;

fn open_slpk_archive(slpk_file_path: PathBuf) -> Result<ZipArchive<impl Read + Seek>, Error> {
    let file = File::open(slpk_file_path)?;
    let buf_reader = BufReader::new(file);
    return Ok(ZipArchive::new(buf_reader)?);
}

fn get_target_directory(mut slpk_file_path: PathBuf) -> Result<PathBuf, Error> {
    // Essentially this just trims the extension from the file path. There
    // has got to be a way to do this without a memory allocation.
    // TODO: Remove this unwrap, map the Option to an Result
    let file_stem = slpk_file_path.file_stem().unwrap().to_os_string();
    slpk_file_path.set_file_name(file_stem);

    // TODO: Handle the case where the slpk file has no extension, and
    // thus the file stem is the same as the filename. In this case we
    // could either error out, or append a suffix (_unpacked maybe?) to
    // the filename.

    // TODO: Probably the behaviour with respect to existing directories
    // should be configurable.

    if slpk_file_path.exists() {
        if slpk_file_path.is_dir() {
            std::fs::remove_dir_all(slpk_file_path.clone())?;
        } else if slpk_file_path.is_file() {
            std::fs::remove_file(slpk_file_path.clone())?;
        }
    }

    std::fs::create_dir(slpk_file_path.clone())?;
    Ok(slpk_file_path)
}

pub fn unpack(slpk_file_path: PathBuf) -> Result<(), Error> {
    let mut slpk_archive = open_slpk_archive(slpk_file_path.clone())?;

    let target_directory = get_target_directory(slpk_file_path)?;
    if target_directory.exists() {}

    for i in 0..slpk_archive.len() {
        let file = slpk_archive.by_index(i)?;
        let mut target_file = target_directory.clone();
        target_file.push(file.sanitized_name());
        println!("{} -> {:?}", file.name(), target_file);
    }

    Ok(())
}
