use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::{
    cmp::min,
    fs::File,
    io::{Seek, Write},
    path::PathBuf,
};
use url::Url;

pub async fn download_with_pb(url: Url, out: &PathBuf) -> Result<(), String> {
    let client = Client::builder().build().unwrap();
    let res = client
        .get(url.clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
.template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.white/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").map_err(|e| e.to_string())?
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
        file = File::create(&out).or(Err(format!("Failed to create file '{}'", &out.display())))?;
    }

    println!("Commencing transfer");
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    let finishmsg = format!("Downloaded {} to {}", url, &out.display());
    pb.finish_with_message(finishmsg);
    Ok(())
}
