This project generates pseudo manifest for installation of your rust crate with scoop.
Creates `app_name.json` manifest in `~\scoop\buckets\local`.

# Usage 
`--cwd` Full path to crate dir

`--alias` Alias for shim created by Scoop

Generated file will contain:
```json
{
    "version": "0.0.0", // From Cargo.toml
    "url": "app_name/target/release/app_name.exe",
    "hash": "", // sha256 of the app_name.exe
    "bin": [
        [
            "app_name.exe",
            "alias to call from terminal"
        ]
    ],
    "license": "Unknown", // License from Cargo.toml or Unknown
    "architecture": {
        "64bit": {
            "url": "app_name/target/release/app_name.exe",
            "hash": "" // sha256 of the app_name.exe
        }
    }
}
```

## Installation can be done with

`task scoop-install` or `scoop install "$HOME\scoop\buckets\local\pseudo_manifest"`
