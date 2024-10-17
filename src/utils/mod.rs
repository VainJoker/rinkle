use std::path::Path;

use tokio::fs;

use crate::prelude::*;

pub async fn walk_dir(dir: &Path) -> Result<Vec<String>> {
	let mut entries = fs::read_dir(dir).await?;
	let mut content = Vec::new();

	while let Some(entry) = entries.next_entry().await? {
		if let Ok(file_name) = entry.file_name().into_string() {
			content.push(file_name);
		}
	}
	Ok(content)
}
