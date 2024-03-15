//! This crate creates pseudo manifest for Scoop

use std::{
    fs::{read_to_string, File},
    io::Write,
};

use anyhow::{bail, Context, Ok, Result};
use clap::Parser;

use colored::Colorize;
use path_clean::clean;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string_pretty, Value};
use sha256::try_digest;
use toml::Table;

/// Pseudo manifest for Scoop
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Serialize, Deserialize)]
struct Manifest {
    version: String,
    url: String,
    hash: String,
    bin: Value,
    license: String,
    architecture: Value,
}

/// Crate's Cargo.toml data
struct CargoToml {
    version: String,
    name: String,
    license: String,
}

/// This program creates pseudo manifest of your crate's release .exe for Scoop
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Current working directory
    #[arg(long)]
    cwd: String,

    #[arg(long)]
    /// Alias for Scoop shim
    alias: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let scoop_bucket = {
        let path = home::home_dir()
            .with_context(|| "Could not find home.")?
            .join("scoop/buckets/local");
        path.canonicalize().with_context(|| {
            format!(
                "Could not find Scoop bucket. Please make sure dir '{}' exists.",
                path.display()
            )
        })?
    };

    let cwd = clean(args.cwd);
    if !cwd.is_dir() {
        bail!("Could not find directory: {}", cwd.display());
    }
    let cargo_toml = read_to_string(
        cwd.join("Cargo.toml")
            .canonicalize()
            .with_context(|| "Could not find Cargo.toml")?,
    )?
    .parse::<Table>()
    .unwrap();

    let cargo_meta = CargoToml {
        name: cargo_toml["package"]["name"]
            .as_str()
            .with_context(|| {
                "Could not parse project name. Please check if Cargo.toml is correct."
            })?
            .to_owned(),
        version: cargo_toml["package"]["version"]
            .as_str()
            .with_context(|| {
                "Could not parse project version. Please check if Cargo.toml is correct."
            })?
            .to_owned(),
        license: {
            let package = cargo_toml["package"].as_table().unwrap();
            if package.contains_key("license") {
                package["license"]
                    .as_str()
                    .with_context(|| {
                        "Could not parse project license. Please check if Cargo.toml is correct."
                    })?
                    .to_owned()
            } else {
                println!(
                    "{}",
                    "Could not find license information. Using Unknown.".yellow()
                );
                println!("If you want to use license in your manifest please add license key to package section.");
                "Unknown".to_string()
            }
        },
    };

    let release_exe = {
        let path = cwd
            .join("target/release/")
            .join(format!("{}.exe", cargo_meta.name));

        path.canonicalize().with_context(|| {
            format!(
                "Could not get {}. Have you compiled release?",
                path.display()
            )
        })?
    };
    let release_hash = try_digest(release_exe.clone())?;

    let manifest = Manifest {
        version: cargo_meta.version,
        url: release_exe.to_str().unwrap().to_owned(),
        hash: release_hash.to_owned(),
        bin: json!([[cargo_meta.name, args.alias]]),
        license: cargo_meta.license,
        architecture: json!({
            "64bit": {
                "url": release_exe.to_str().unwrap().to_owned(),
                "hash": release_hash.to_owned()
        }}),
    };

    let manifest_path = scoop_bucket.join(format!("{}.json", cargo_meta.name));
    let mut file = File::create(&manifest_path)?;
    file.write_all(to_string_pretty(&manifest)?.as_bytes())?;
    println!(
        "{} At {}",
        "Manifest file successfully created.".green(),
        &manifest_path.display()
    );

    Ok(())
}
