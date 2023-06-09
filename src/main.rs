use std::{
    error::Error,
    fs::{remove_dir, remove_file, File},
    io::Write,
    process::Command,
};

use clap::Parser;
use flate2::read::GzDecoder;
use tar::Archive;

const CDN_URL: &str = "https://cm.topc.at";

const PREFIX_WIN: &str = "win/";
const PREFIX_NIX: &str = "nix/";
const PREFIX_OSX: &str = "osx/";

const FILENAME_WIN: &str = "Win64.zip";
const FILENAME_NIX: &str = "Linux.tar.gz";
const FILENAME_OSX: &str = "MacOS.tar.gz";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Update channel
    #[arg(long, default_value = "stable")]
    channel: String,

    /// What chromapper version to dowload
    #[arg(long, default_value_t = -1)]
    cm_version: i32,

    /// Keep the downloaded file after unpacking it
    #[arg(long, default_value_t = false)]
    keep_download_file: bool,

    /// More log output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args = Args::parse();

    let channel = args.channel; // stable|dev
    let mut version: i32 = args.cm_version;

    let (pfx, fln) = get_os_specifics();

    let prefix = pfx.as_str();
    let file_name = fln.as_str();

    if version == -1 {
        match fetch_version(&channel).await {
            Ok(vers) => {
                version = vers;
            }
            Err(e) => {
                eprintln!("Error while fetching version: {e}");
                return Err(());
            }
        }
    }

    let download_file_name = "download.tar.gz";

    println!("Downloading archive file");
    match download_update_file(prefix, version, file_name, download_file_name).await {
        Ok(_) => {
            println!("Done");
        }
        Err(e) => {
            eprintln!("Error downloading archive file: {e}");
            return Err(());
        }
    }

    remove_dir("chromapper");

    println!("Unpacking archive");
    let result = unpack_download_file(download_file_name);

    if !args.keep_download_file {
        remove_file(download_file_name);
    }

    match result {
        Ok(_) => {
            println!("Done");
        }
        Err(_) => {
            println!("Error unpacking archive");
            return Err(());
        }
    }

    println!("Launching ChroMapper");
    launch_cm();

    Ok(())
}

fn get_os_specifics() -> (String, String) {
    if cfg!(windows) {
        return (PREFIX_WIN.to_string(), FILENAME_WIN.to_string());
    } else if cfg!(target_os = "macos") {
        return (PREFIX_OSX.to_string(), FILENAME_OSX.to_string());
    } else {
        return (PREFIX_NIX.to_string(), FILENAME_NIX.to_string());
    }
}

async fn fetch_version(channel: &str) -> Result<i32, Box<dyn Error>> {
    let url = format!("{CDN_URL}/{channel}");

    let args = Args::parse();
    if args.verbose {
        println!("Getting version from {url}");
    }

    let response = reqwest::get(url).await?;
    let body = response.error_for_status()?.text().await?;
    let version: i32 = body.parse()?;

    Ok(version)
}

async fn download_update_file(
    prefix: &str,
    version: i32,
    file_name: &str,
    out_file_name: &str,
) -> Result<(), Box<dyn Error>> {
    let url = format!("{CDN_URL}/{prefix}{version}/{file_name}");

    let args = Args::parse();
    if args.verbose {
        println!("Downloading from {url}");
    }

    download_file(&url, out_file_name).await
}

async fn download_file(url: &str, out_file_name: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let body = response.error_for_status()?.bytes().await?;

    let mut file = std::fs::File::create(out_file_name).expect("Could not create file");
    file.write_all(&body)?;

    Ok(())
}

fn unpack_download_file(file_name: &str) -> Result<(), Box<dyn Error>> {
    let in_file = File::open(file_name)?;
    let gz_decoder = GzDecoder::new(in_file);
    let mut archive = Archive::new(gz_decoder);
    archive.unpack("./")?;

    Ok(())
}

fn launch_cm() -> Result<(), Box<dyn Error>> {
    Command::new("./chromapper/ChroMapper").output()?;
    Ok(())
}
