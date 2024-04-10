//! This crate creates pseudo manifest for Scoop

use std::{
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

use anyhow::{bail, Context, Error, Ok, Result};
use clap::Parser;

use colored::Colorize;
use path_clean::clean;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_string_pretty, Value};
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
    shortcuts: Value,
    license: String,
    architecture: Value,
}

/// Crate's Cargo.toml data
struct CargoToml {
    version: String,
    name: String,
    license: String,
}

/// Should manifest be updated?
struct Status {
    update: bool,
    create: bool,
}

impl Status {
    fn new() -> Self {
        Self {
            update: false,
            create: false,
        }
    }
    /// Check if existing manifest file needs updating
    /// Doesn't return anything, but updates `update` and `create` fields.
    fn check(&mut self, new_manifest: &Manifest, scoop_path: &Path) -> Result<()> {
        if !scoop_path.is_file() {
            self.create = true;
            return Ok(());
        }
        if let Result::Ok(file) = read_to_string(scoop_path) {
            let file_data: Option<Manifest> = from_str(&file).ok();
            if file_data.is_none() {
                self.create = true;
                return Ok(());
            }
            let old_manifest = file_data.unwrap();
            if old_manifest.hash == new_manifest.hash {
                println!("{}", "Already up to date.".yellow());
                return Ok(());
            } else {
                if old_manifest.version == new_manifest.version {
                    return Err(Error::msg(
                        "Unable to update manifest. \
                        Hashes don't match, yet, app's version wasn't changed. \
                        \nPlease, update your app's version."
                            .red(),
                    ));
                }
                self.update = true;
                return Ok(());
            }
        };

        self.create = true;
        Ok(())
    }
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
    /// Alias for Scoop shim or Shortcut.
    /// Alias should be meaningful when creating a shortcut
    alias: String,

    #[arg(long, short)]
    /// Look for a specific file
    file: Option<String>,

    #[arg(long)]
    /// Name of the installed exe
    bin: Option<String>,

    #[arg(long)]
    /// Is this a gui app or cli?
    /// Defaults to false i.e. cli.
    gui: bool,

    #[arg(long)]
    /// Debug manifest creation. Created manifest in cwd.
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cwd = clean(args.cwd);
    if !cwd.is_dir() {
        bail!("Could not find directory: {}", cwd.display());
    }

    let scoop_bucket = {
        if !args.debug {
            let path = home::home_dir()
                .with_context(|| "Could not find home.")?
                .join("scoop/buckets/local");
            path.canonicalize().with_context(|| {
                format!(
                    "Could not find Scoop bucket. Please make sure dir '{}' exists.",
                    path.display()
                )
            })?
        } else {
            cwd.to_owned()
        }
    };

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

    // Path to exe file
    let release_file = {
        if args.file.is_none() {
            let path = cwd
                .join("target/release/")
                .join(format!("{}.exe", cargo_meta.name));

            path.canonicalize().with_context(|| {
                format!(
                    "Could not get {}. Have you compiled release?",
                    path.display()
                )
            })?
        } else {
            let file = args.file.unwrap();
            let path = Path::new(&file);
            path.canonicalize()
                .with_context(|| format!("Could not find provided path: {}", path.display()))?
        }
    };

    let release_hash = try_digest(&release_file)?;
    let release_url = release_file.to_str().unwrap().replace(r"\\?\", ""); // Scoop can't parse URL otherwise.
    let bin_name = if args.bin.is_none() {
        release_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    } else {
        args.bin.unwrap()
    };

    // Building manifest
    let manifest = Manifest {
        version: cargo_meta.version,
        url: release_url.to_owned(),
        hash: release_hash.to_owned(),
        bin: if !args.gui {
            json!([[bin_name, args.alias]])
        } else {
            serde_json::to_value(bin_name.to_owned())?
        },
        shortcuts: if args.gui {
            json!([[bin_name, args.alias]])
        } else {
            json!([])
        },
        license: cargo_meta.license,
        architecture: json!({
            "64bit": {
                "url": release_url.to_owned(),
                "hash": release_hash.to_owned()
            }
        }),
    };

    let manifest_path = scoop_bucket.join(format!("{}.json", cargo_meta.name));
    let mut manifest_status = Status::new();
    if !args.debug {
        manifest_status.check(&manifest, &manifest_path)?;
    }

    if manifest_status.update || manifest_status.create || args.debug {
        let mut file = File::create(&manifest_path)?;
        file.write_all(to_string_pretty(&manifest)?.as_bytes())?;
        println!(
            "{} {} At {}",
            "Manifest file successfully".green(),
            if manifest_status.update {
                "updated".green()
            } else if manifest_status.create {
                "created".green()
            } else {
                "created for debugging".red()
            },
            manifest_path.display()
        );
    } else {
        println!("{}", "Could not determine what to do.".red());
    }

    Ok(())
}
