use failure::Error;

use flate2::read::GzDecoder;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;
use zip::ZipArchive;
use zip::read::ZipFile;

fn open_slpk_archive(slpk_file_path: PathBuf) -> Result<ZipArchive<impl Read + Seek>, Error> {
    let file = File::open(slpk_file_path)?;
    let buf_reader = BufReader::new(file);
    return Ok(ZipArchive::new(buf_reader)?);
}

fn get_unpack_folder(mut slpk_file_path: PathBuf) -> Result<PathBuf, Error> {
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
            println!("Deleting folder: {}", slpk_file_path.to_string_lossy());
            std::fs::remove_dir_all(slpk_file_path.clone())?;
        } else if slpk_file_path.is_file() {
            println!("Deleting file: {}", slpk_file_path.to_string_lossy());
            std::fs::remove_file(slpk_file_path.clone())?;
        }
    }

    std::fs::create_dir(slpk_file_path.clone())?;
    Ok(slpk_file_path)
}

fn create_folder_for_entry(
    mut target_directory: PathBuf,
    zip_entry: &PathBuf,
) -> Result<PathBuf, Error> {
    // TODO: if the zip_entry is an absolute path, this will end up creating an absolute path on
    // the target machine. This shouldn't happen, as all files should be extracted into the
    // target folder.
    if let Some(parent_path) = zip_entry.parent() {
        target_directory.push(parent_path);
        std::fs::create_dir_all(target_directory.clone())?;
    }
    Ok(target_directory)
}

fn unpack_entry(mut archive_entry: ZipFile, unpack_folder: PathBuf) -> Result<(), Error> {
    let archive_entry_path = archive_entry.sanitized_name();
    let target_folder = create_folder_for_entry(unpack_folder, &archive_entry_path)?;

    if let Some("gz") = archive_entry_path.extension().and_then(|x| x.to_str()) {
        let mut target_file_path = target_folder;
        target_file_path.push(archive_entry_path.file_stem().unwrap());

        println!(
            "Decompress: {} -> {}",
            archive_entry.name(),
            target_file_path.to_str().unwrap()
        );
        let mut gz_reader = GzDecoder::new(archive_entry);

        let mut target_file = File::create(target_file_path)?;
        std::io::copy(&mut gz_reader, &mut target_file)?;
    } else {
        let mut target_file_path = target_folder;
        target_file_path.push(archive_entry_path.file_name().unwrap());

        println!(
            "Copy: {} -> {}",
            archive_entry.name(),
            target_file_path.to_str().unwrap()
        );
        let mut target_file = File::create(target_file_path)?;
        std::io::copy(&mut archive_entry, &mut target_file)?;
    }

    Ok(())
}

pub fn unpack(slpk_file_path: PathBuf) -> Result<(), Error> {
    let mut slpk_archive = open_slpk_archive(slpk_file_path.clone())?;
    let unpack_folder = get_unpack_folder(slpk_file_path)?;

    for i in 0..slpk_archive.len() {
        let archive_entry = slpk_archive.by_index(i)?;
        unpack_entry(archive_entry, unpack_folder.clone())?;
    }

    Ok(())
}
