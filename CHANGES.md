- Replaced 1.4 ModInfo 'version' and 'tmodloader_version' with 'versions' array
- New struct: 
  ```rust 
    struct ModVersion {
        mod_version: String,
        tmodloader_version: String,
    }
  ```