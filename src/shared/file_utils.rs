use anyhow::Context;
use async_recursion::async_recursion;
use std::path::PathBuf;
use tokio::fs;

pub async fn ensure_directory_exists_and_empty(dir: &PathBuf) -> anyhow::Result<()> {
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .await
            .with_context(|| "Failed to create directory".to_string())?;
    } else {
        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(path).await?;
            }
        }
    }
    Ok(())
}

pub async fn write_index_to_file(file: &PathBuf, index: &str) -> anyhow::Result<()> {
    fs::write(file, index)
        .await
        .with_context(|| format!("Failed to write file: {}", file.to_str().unwrap()))?;
    Ok(())
}

#[async_recursion]
pub async fn list_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            list_files(&path, files).await?;
        } else {
            continue; // skip links, sockets etc
        }
    }
    Ok(())
}

pub async fn read_file(file: &PathBuf) -> anyhow::Result<String> {
    let content = tokio::fs::read_to_string(file)
        .await
        .with_context(|| format!("Failed to read file: {}", file.to_str().unwrap()))?;
    Ok(content)
}
