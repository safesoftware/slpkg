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

#[derive(Debug, Fail)]
enum UnpackError {
    #[fail(display = "Unable to create a folder for unpacking the package")]
    NoFolderForPackage,
}

fn open_slpk_archive(slpk_file_path: PathBuf) -> Result<ZipArchive<impl Read + Seek>, Error> {
    let file = File::open(slpk_file_path)?;
    let buf_reader = BufReader::new(file);
    return Ok(ZipArchive::new(buf_reader)?);
}

fn get_unpack_folder(mut slpk_file_path: PathBuf, verbose: bool) -> Result<PathBuf, Error> {
    // Try to extract the file stem. This name will be used as the folder name which
    // the package will be unpacked into. If the package has no file_stem, then
    // it cannot be unpacked. We could come up with some other name to use, but
    // this is a fairly unlikely scenario, so just quitting is ok.
    match slpk_file_path.extension() {
        Some(_) => {
            if let Some(file_stem) = slpk_file_path.file_stem() {
                slpk_file_path.set_file_name(file_stem.to_os_string());
            } else {
                // This probably shouldn't happen. Tough to have a file with an
                // extension but no file stem.
                return Err(Error::from(UnpackError::NoFolderForPackage));
            }
        }
        None => {
            // The file has no extension. This means we cannot use the file basename
            // as the unpacking folder.
            return Err(Error::from(UnpackError::NoFolderForPackage));
        }
    }

    // TODO: Probably the behaviour with respect to existing directories
    // should be configurable.

    if slpk_file_path.exists() {
        if slpk_file_path.is_dir() {
            if verbose {
                println!("Deleting folder: {}", slpk_file_path.to_string_lossy());
            }
            std::fs::remove_dir_all(slpk_file_path.clone())?;
        } else if slpk_file_path.is_file() {
            if verbose {
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
    verbose: bool,
) -> Result<(), Error> {
    let archive_entry_path = archive_entry.sanitized_name();
    let target_folder = create_folder_for_entry(unpack_folder, &archive_entry_path)?;

    if let Some("gz") = archive_entry_path.extension().and_then(|x| x.to_str()) {
        if let Some(non_gzip_name) = archive_entry_path.file_stem() {
            let mut target_file_path = target_folder;
            target_file_path.push(non_gzip_name);

            if verbose {
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

            if verbose {
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

fn calculate_entries_for_thread(
    thread_id: usize,
    num_threads: usize,
    num_entries: usize,
) -> Option<(usize, usize)> {
    // In theory, we're going to divide the work between threads, so that each thread gets
    // an equal number of entries. In practice however, the number of entries is not going
    // to be divisible by the number of threads, so some threads have to do extra work.
    let entries_per_thread = ((num_entries as f64) / (num_threads as f64)).ceil() as usize;

    // Note that if the number of entries is small and the number of threads is fairly large,
    // the calculations below could result in entry indices which are greater than num_entries.
    let start_entry = entries_per_thread * thread_id;
    let end_entry = entries_per_thread * (thread_id + 1);

    if start_entry < num_entries {
        return Some((start_entry, std::cmp::min(end_entry, num_entries)));
    } else {
        return None;
    }
}

pub fn unpack(slpk_file_path: PathBuf, verbose: bool) -> Result<(), Error> {
    let unpack_folder = get_unpack_folder(slpk_file_path.clone(), verbose)?;

    // We'll create one thread per CPU core.
    let num_threads = num_cpus::get();

    let mut threads = Vec::with_capacity(num_threads);
    for i in 0..num_threads {
        let slpk_file_path = slpk_file_path.clone();
        let unpack_folder = unpack_folder.clone();
        threads.push(thread::spawn(move || -> Result<usize, Error> {
            let mut slpk_archive = open_slpk_archive(slpk_file_path.clone())?;
            let num_entries = slpk_archive.len();

            let mut entries_unpacked = 0;
            if let Some((start_entry, end_entry)) =
                calculate_entries_for_thread(i, num_threads, num_entries)
            {
                for entry_idx in start_entry..end_entry {
                    let archive_entry = slpk_archive.by_index(entry_idx)?;
                    unpack_entry(archive_entry, unpack_folder.clone(), verbose)?;
                    entries_unpacked += 1;
                }
            }

            Ok(entries_unpacked)
        }));
    }

    let mut total_entries_unpacked = 0;
    for t in threads {
        let thread_result = t.join();
        match thread_result {
            Ok(Ok(n)) => {
                total_entries_unpacked += n;
            }
            Ok(Err(e)) => {
                eprintln!("{}", e);
                // TODO: Should this return, or wait for other threads to finish?
                return Err(e);
            }
            Err(e) => {
                eprintln!("{:?}", e);
                panic!("Thread panicked!")
            }
        }
    }

    if verbose {
        println!("{} files unpacked", total_entries_unpacked);
    }

    Ok(())
}
