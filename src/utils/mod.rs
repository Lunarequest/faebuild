pub mod git;
mod private;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use private::download_with_pb;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Error, Read},
    path::PathBuf,
    process::exit,
};
use tar::Archive;
use url::Url;
use xz2::read::XzDecoder;
use zip::ZipArchive;
use zstd::stream::Decoder;

pub async fn calculate_sha56sum(path: &PathBuf) -> Result<String, Error> {
    if !path.is_file() {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path is not a file",
        ));
    }
    // Open the file in read mode
    let mut file = File::open(path)?;

    // Create a SHA256 hasher
    let mut hasher = Sha256::new();

    // Read the file in chunks and feed them to the hasher
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Obtain the resulting hash value
    let hash_result = hasher.finalize();

    // Convert the hash value to a hexadecimal string
    let checksum = hash_result
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();

    Ok(checksum)
}

pub async fn download_and_extract_with_sha(
    url: Url,
    sha256sum: String,
    src_out: &PathBuf,
    workdir: &PathBuf,
) -> Result<PathBuf, String> {
    download_with_pb(url, src_out).await?;

    let sha = calculate_sha56sum(src_out)
        .await
        .map_err(|e| e.to_string())?;
    if sha256sum != sha {
        eprintln!("expected sha256sum: {sha256sum} got {sha}");
        exit(1);
    }

    if src_out.to_owned().ends_with(".gz") {
        let tar_gz = File::open(src_out).map_err(|e| e.to_string())?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(workdir).map_err(|e| e.to_string())?;
    } else if src_out.ends_with(".xz") {
        let tar_xz = File::open(src_out).map_err(|e| e.to_string())?;
        let tar = XzDecoder::new(tar_xz);
        let mut archive = Archive::new(tar);
        archive.unpack(workdir).map_err(|e| e.to_string())?;
    } else if src_out.ends_with(".bz2") {
        let tar_bz = File::open(src_out).map_err(|e| e.to_string())?;
        let tar = BzDecoder::new(tar_bz);
        let mut archive = Archive::new(tar);
        archive.unpack(workdir).map_err(|e| e.to_string())?;
    } else if src_out.ends_with(".zstd") {
        let tar_zstd = File::open(src_out).map_err(|e| e.to_string())?;
        let tar = Decoder::new(tar_zstd).map_err(|e| e.to_string())?;
        let mut archive = Archive::new(tar);
        archive.unpack(workdir).map_err(|e| e.to_string())?;
    } else if src_out.ends_with("zip") {
        let zip = File::open(src_out).map_err(|e| e.to_string())?;
        let mut zip_archive = ZipArchive::new(zip).map_err(|e| e.to_string())?;
        zip_archive.extract(workdir).map_err(|e| e.to_string())?;
    } else {
        eprintln!("Usupported archive format");
        exit(1);
    }

    Ok(workdir.to_owned())
}
