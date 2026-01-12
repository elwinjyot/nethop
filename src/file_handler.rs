use std::{env::current_dir, ffi::OsStr, fs, path::Path};

use walkdir::WalkDir;

pub fn read_queries_from_file(path: &str) -> Result<String, String> {
    let path = Path::new(path);
    if path.extension().unwrap_or(OsStr::new("")) != "hop" {
        return Err("Only .hop files are supported!".into());
    }

    if !path.exists() {
        return Err(format!(
            "{} no such file or directory",
            path.to_string_lossy()
        ));
    }

    let queries =
        fs::read_to_string(path).map_err(|e| format!("Could not read the passed file, {}", e))?;
    Ok(queries)
}

pub fn read_queries_from_workspace() -> Result<String, String> {
    let mut path = current_dir().map_err(|_| "Failed to get current working directory")?;
    path.push(".nethop");

    let config_path = path.join("config.hop");
    if !config_path.exists() {
        return Err("Your project does not have nethop setup. Run nethop init to get a basic structure initialized".to_string());
    }

    let mut script = String::new();
    let mut files_found = 0;

    let headers =
        fs::read_to_string(&config_path).map_err(|_| "Failed to read base config.hop file")?;
    script.push_str(&headers);
    script.push('\n');

    println!("🔍 Searching workspace files...\n");

    for entry in WalkDir::new(&path).into_iter().filter_map(|et| et.ok()) {
        let file_path = entry.path();

        if file_path.is_file()
            && file_path.extension().is_some_and(|f| f == "hop")
            && file_path != config_path
        {
            let content = fs::read_to_string(file_path)
                .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

            script.push_str(&content);
            script.push('\n');

            files_found += 1;
            println!(
                "🕸️  {} file added",
                file_path
                    .file_name()
                    .unwrap_or(OsStr::new(""))
                    .to_string_lossy()
            );
        }
    }

    if files_found == 0 {
        println!("\n⚠️  Only config.hop was found.\n");
    } else {
        println!("\n✅ {} additional hops found. \n", files_found);
    }

    Ok(script)
}
