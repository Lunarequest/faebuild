pub mod git;
use anyhow::{anyhow, Context, Result};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
pub use patch::patch;
use reqwest::{redirect::Policy, Client};
use sha2::{Digest, Sha256};
use std::{
    cmp::min,
    io::{Seek, Write},
};
use std::{
    ffi::OsStr,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::exit,
};
use tar::Archive;
use url::Url;
use xz2::read::XzDecoder;
use zip::ZipArchive;
use zstd::stream::Decoder;
mod patch;

pub fn get_filename_from_url(url: &Url) -> Option<String> {
    if let Some(path) = url.path_segments() {
        if let Some(last_segment) = path.last() {
            if let Some(filename) = Path::new(last_segment).file_name() {
                return Some(filename.to_string_lossy().to_string());
            }
        }
    }
    None
}

pub async fn download_with_pb(url: Url, out: &PathBuf) -> Result<()> {
    let client = Client::builder()
        .redirect(Policy::limited(10))
        .build()
        .unwrap();
    let res = client.get(url.clone()).fetch_mode_no_cors().send().await?;

    if !res.status().is_success() {
        return Err(anyhow!("The {url} return status code {}", res.status()));
    }

    let total_size = res
        .content_length()
        .context(format!("Failed to get content length from '{}'", &url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
.template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
.progress_chars("█  "));

    let mut file;
    let mut downloaded = 0;
    let mut stream = res.bytes_stream();

    println!("Seeking in file.");
    if std::path::Path::new(&out).exists() {
        println!("File exists. Resuming.");
        file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(&out)
            .unwrap();

        let file_size = std::fs::metadata(&out).unwrap().len();
        file.seek(std::io::SeekFrom::Start(file_size)).unwrap();
        downloaded = file_size;
    } else {
        println!("Fresh file..");
        file = File::create(&out).context(format!("Failed to create file '{}'", &out.display()))?;
    }

    println!("Commencing transfer");
    while let Some(item) = stream.next().await {
        let chunk = item.context("Error while downloading file")?;
        file.write(&chunk).context("Error while writing to file")?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    let finishmsg = format!("Downloaded {} to {}", url, &out.display());
    pb.finish_with_message(finishmsg);
    Ok(())
}

pub async fn calculate_sha56sum(path: &PathBuf) -> Result<String> {
    if !path.is_file() {
        return Err(anyhow!("Path is not a file",));
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

pub async fn extract_with_sha(
    sha256sum: String,
    src_out: &PathBuf,
    workdir: &PathBuf,
) -> Result<PathBuf> {
    let sha = calculate_sha56sum(src_out).await?;
    if sha256sum != sha {
        eprintln!("expected sha256sum: {sha256sum} got {sha}");
        exit(1);
    }

    if src_out.extension().and_then(OsStr::to_str) == Some("gz") {
        let tar_gz = File::open(src_out)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        archive.unpack(workdir)?;
    } else if src_out.extension().and_then(OsStr::to_str) == Some("xz") {
        let tar_xz = File::open(src_out)?;
        let tar = XzDecoder::new(tar_xz);
        let mut archive = Archive::new(tar);
        archive.unpack(workdir)?;
    } else if src_out.extension().and_then(OsStr::to_str) == Some("bz2") {
        let tar_bz = File::open(src_out)?;
        let tar = BzDecoder::new(tar_bz);
        let mut archive = Archive::new(tar);
        archive.unpack(workdir)?;
    } else if src_out.extension().and_then(OsStr::to_str) == Some("zstd") {
        let tar_zstd = File::open(src_out)?;
        let tar = Decoder::new(tar_zstd)?;
        let mut archive = Archive::new(tar);
        archive.unpack(workdir)?;
    } else if src_out.extension().and_then(OsStr::to_str) == Some("zip") {
        let zip = File::open(src_out)?;
        let mut zip_archive = ZipArchive::new(zip)?;
        zip_archive.extract(workdir)?;
    } else {
        eprintln!("Usupported archive format");
        exit(1);
    }

    Ok(workdir.to_owned())
}
