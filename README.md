This project generates pseudo manifest for installation of your rust crate with scoop.
Creates `app_name.json` manifest in `~\scoop\buckets\local`.

Generated file will contain:
```json
{
    "version": "0.0.0", // From Cargo.toml
    "url": "path/to/local/app_name.exe",
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
            "url": "path/to/local/app_name.exe",
            "hash": "" // sha256 of the app_name.exe
        }
    }
}
```
