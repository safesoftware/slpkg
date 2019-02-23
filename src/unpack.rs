use failure::Error;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;
use std::thread;
use zip::read::ZipFile;
use zip::ZipArchive;

#[derive(Debug,Fail)]
enum UnpackError {
    #[fail(display="Unable to create a folder for unpacking the package")]
    NoFolderForPackage
}

fn open_slpk_archive(slpk_file_path: PathBuf) -> Result<ZipArchive<impl Read + Seek>, Error> {
    let file = File::open(slpk_file_path)?;
    let buf_reader = BufReader::new(file);
    return Ok(ZipArchive::new(buf_reader)?);
}

fn get_unpack_folder(mut slpk_file_path: PathBuf, quiet: bool) -> Result<PathBuf, Error> {

    // Try to extract the file stem. This name will be used as the folder name which
    // the package will be unpacked into. If the package has no file_stem, then
    // it cannot be unpacked. We could come up with some other name to use, but
    // this is a fairly unlikely scenario, so just quitting is ok.
    let file_stem = slpk_file_path.file_stem();
    match file_stem {
        Some(name) => {
            slpk_file_path.set_file_name(name.to_os_string());
        },
        None => {
            return Err(Error::from(UnpackError::NoFolderForPackage));
        }
    }

    // TODO: Probably the behaviour with respect to existing directories
    // should be configurable.

    if slpk_file_path.exists() {
        if slpk_file_path.is_dir() {
            if !quiet {
                println!("Deleting folder: {}", slpk_file_path.to_string_lossy());
            }
            std::fs::remove_dir_all(slpk_file_path.clone())?;
        } else if slpk_file_path.is_file() {
            if !quiet {
                println!("Deleting file: {}", slpk_file_path.to_string_lossy());
            }
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

fn unpack_entry(
    mut archive_entry: ZipFile,
    unpack_folder: PathBuf,
    quiet: bool,
) -> Result<(), Error> {
    let archive_entry_path = archive_entry.sanitized_name();
    let target_folder = create_folder_for_entry(unpack_folder, &archive_entry_path)?;

    if let Some("gz") = archive_entry_path.extension().and_then(|x| x.to_str()) {
        if let Some(non_gzip_name) = archive_entry_path.file_stem() {
            let mut target_file_path = target_folder;
            target_file_path.push(non_gzip_name);

            if !quiet {
                println!(
                    "Decompress: {} -> {}",
                    archive_entry.name(),
                    target_file_path.to_string_lossy()
                );
            }

            let mut gz_reader = GzDecoder::new(archive_entry);
            let mut target_file = File::create(target_file_path)?;
            std::io::copy(&mut gz_reader, &mut target_file)?;
        }
    } else {
        if let Some(name) = archive_entry_path.file_name() {
            let mut target_file_path = target_folder;
            target_file_path.push(name);

            if !quiet {
                println!(
                    "Copy: {} -> {}",
                    archive_entry.name(),
                    target_file_path.to_string_lossy()
                );
            }

            let mut target_file = File::create(target_file_path)?;
            std::io::copy(&mut archive_entry, &mut target_file)?;
        }
    }

    Ok(())
}

pub fn unpack(slpk_file_path: PathBuf, quiet: bool) -> Result<(), Error> {
    let unpack_folder = get_unpack_folder(slpk_file_path.clone(), quiet)?;

    // We'll create one thread per CPU core.
    let num_threads = num_cpus::get();

    let mut threads = Vec::with_capacity(num_threads);
    for i in 0..num_threads {
        let slpk_file_path = slpk_file_path.clone();
        let unpack_folder = unpack_folder.clone();
        threads.push(thread::spawn(move || -> Result<(), Error> {
            let mut slpk_archive = open_slpk_archive(slpk_file_path.clone())?;

            let entries_per_thread = slpk_archive.len() / num_threads;
            let start_entry = entries_per_thread * i;
            let end_entry = std::cmp::min(entries_per_thread * (i + 1), slpk_archive.len());

            for entry_idx in start_entry..end_entry {
                let archive_entry = slpk_archive.by_index(entry_idx)?;
                unpack_entry(archive_entry, unpack_folder.clone(), quiet)?;
            }

            Ok(())
        }));
    }

    for t in threads {
        if let Ok(Err(e)) = t.join() {
            eprintln!("{}", e);

            // TODO: Should this return, or wait for other threads to finish?
            return Err(e);
        }
    }

    Ok(())
}
