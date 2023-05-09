use std::{error::Error, io::Write};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Update channel
    #[arg(long, default_value = "stable")]
    channel: String,

    /// What chromapper version to dowload
    #[arg(long, default_value_t = -1)]
    cm_version: i32,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args = Args::parse();

    let channel = args.channel; // stable|dev
    let mut version: i32 = args.cm_version;

    // TODO: Dynamic
    let prefix = "nix/";
    let file_name = "Linux.tar.gz";

    if version == -1 {
        match fetch_version(&channel).await {
            Ok(vers) => {
                version = vers;
            }
            Err(e) => {
                eprintln!("Error while fetching version: {e}");
            }
        }
    }

    println!("Downloading zip file");
    match download_update_zip(prefix, version, file_name, "download.zip").await {
        Ok(_) => {
            println!("Done")
        }
        Err(e) => {
            eprintln!("Error downloading zip file: {e}")
        }
    }

    Ok(())
}

async fn fetch_version(channel: &str) -> Result<i32, Box<dyn Error>> {
    let cdn_url = "https://cm.topc.at";
    let url = format!("{cdn_url}/{channel}");

    let response = reqwest::get(url).await?;
    let body = response.error_for_status()?.text().await?;
    let version: i32 = body.parse()?;

    Ok(version)
}

async fn download_update_zip(
    prefix: &str,
    version: i32,
    file_name: &str,
    out_file_name: &str,
) -> Result<(), Box<dyn Error>> {
    let cdn_url = "https://cm.topc.at";
    let url = format!("{cdn_url}/{prefix}{version}/{file_name}");

    let args = Args::parse();
    if args.verbose {
        println!("Downloading from {url}");
    }

    return download_file(&url, out_file_name).await;
}

async fn download_file(url: &str, out_file_name: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let body = response.error_for_status()?.bytes().await?;

    let mut file = std::fs::File::create(out_file_name).expect("Could not create file");
    file.write_all(&body)?;

    Ok(())
}