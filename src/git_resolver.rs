use std::io::Cursor;
use reqwest::Client;
use crate::pb::GitSource;

pub async fn resolve_git_source(source: &GitSource) -> Result<Vec<u8>, String> {
    if source.repository.is_empty() {
        return Err("Repository URL is empty".to_string());
    }
    if source.file_path.is_empty() {
        return Err("File path is empty".to_string());
    }

    let git_ref = if source.git_ref.is_empty() {
        "main"
    } else {
        &source.git_ref
    };

    let url = build_raw_url(&source.repository, git_ref, &source.file_path)?;

    // Initialize client and request
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let mut req = client.get(&url);

    // Apply authentication token if provided
    if !source.git_token.is_empty() {
        if url.contains("github.com") || url.contains("githubusercontent.com") {
            req = req.header("Authorization", format!("token {}", source.git_token));
        } else if url.contains("gitlab.com") {
            req = req.header("PRIVATE-TOKEN", &source.git_token);
        } else {
            req = req.header("Authorization", format!("Bearer {}", source.git_token));
        }
    }

    let response = req.send().await
        .map_err(|e| format!("Failed to fetch file from Git: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Git server returned error status: {} for URL: {}", response.status(), url));
    }

    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?
        .to_vec();

    // Check if the downloaded file is a ZIP archive
    let is_zip = source.file_path.ends_with(".zip") || 
                 (bytes.len() >= 4 && &bytes[0..4] == b"PK\x03\x04");

    if is_zip {
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to parse ZIP archive: {}", e))?;

        let mut wasm_bytes = None;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read ZIP entry at index {}: {}", i, e))?;
            
            if file.name().ends_with(".wasm") {
                let mut buf = Vec::new();
                std::io::copy(&mut file, &mut buf)
                    .map_err(|e| format!("Failed to extract WASM from ZIP: {}", e))?;
                wasm_bytes = Some(buf);
                break;
            }
        }

        wasm_bytes.ok_or_else(|| "No .wasm file found inside the ZIP archive".to_string())
    } else {
        Ok(bytes)
    }
}

pub fn build_raw_url(repository: &str, branch: &str, file_path: &str) -> Result<String, String> {
    let repo = repository.trim().trim_end_matches(".git");
    if repo.contains("github.com") {
        let parts: Vec<&str> = repo.split("github.com/").collect();
        if parts.len() == 2 {
            let path = parts[1]; // owner/repo
            Ok(format!("https://raw.githubusercontent.com/{}/{}/{}", path, branch, file_path))
        } else {
            Err("Invalid GitHub repository URL format".to_string())
        }
    } else if repo.contains("gitlab.com") {
        let parts: Vec<&str> = repo.split("gitlab.com/").collect();
        if parts.len() == 2 {
            let path = parts[1]; // owner/repo
            Ok(format!("https://gitlab.com/{}/-/raw/{}/{}", path, branch, file_path))
        } else {
            Err("Invalid GitLab repository URL format".to_string())
        }
    } else {
        Err("Unsupported Git host. Only GitHub and GitLab are currently supported.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_raw_url_github() {
        let url = build_raw_url("https://github.com/nativebpm/wasmee", "main", "dist/app.wasm").unwrap();
        assert_eq!(url, "https://raw.githubusercontent.com/nativebpm/wasmee/main/dist/app.wasm");

        let url_git = build_raw_url("https://github.com/nativebpm/wasmee.git", "dev", "app.zip").unwrap();
        assert_eq!(url_git, "https://raw.githubusercontent.com/nativebpm/wasmee/dev/app.zip");
    }

    #[test]
    fn test_build_raw_url_gitlab() {
        let url = build_raw_url("https://gitlab.com/nativebpm/wasmee", "master", "app.wasm").unwrap();
        assert_eq!(url, "https://gitlab.com/nativebpm/wasmee/-/raw/master/app.wasm");
    }

    #[test]
    fn test_build_raw_url_invalid() {
        let res = build_raw_url("https://otherhost.com/repo", "main", "app.wasm");
        assert!(res.is_err());
    }

    #[test]
    fn test_zip_decompression() {
        // Create a mock zip archive in memory with a dummy .wasm file
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let w = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(w);
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            
            zip.start_file("test.wasm", options).unwrap();
            zip.write_all(b"\x00asm\x01\x00\x00\x00dummy_wasm_code").unwrap();
            zip.finish().unwrap();
        }

        // Run extraction logic manually
        let cursor = std::io::Cursor::new(buf);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut wasm_bytes = None;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            if file.name().ends_with(".wasm") {
                let mut buf = Vec::new();
                std::io::copy(&mut file, &mut buf).unwrap();
                wasm_bytes = Some(buf);
                break;
            }
        }

        let extracted = wasm_bytes.unwrap();
        assert_eq!(extracted, b"\x00asm\x01\x00\x00\x00dummy_wasm_code");
    }
}
