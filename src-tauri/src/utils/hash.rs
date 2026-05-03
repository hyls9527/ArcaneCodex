use sha2::{Sha256, Digest};
use std::path::Path;

pub fn calculate_sha256(file_path: &Path) -> std::io::Result<String> {
    let content = std::fs::read(file_path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    
    #[test]
    fn test_calculate_sha256() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_sha256.txt");
        {
            let mut file = std::fs::File::create(&test_file).unwrap();
            file.write_all(b"Hello, World!").unwrap();
        }
        
        let hash = calculate_sha256(&test_file).unwrap();
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
        
        std::fs::remove_file(&test_file).ok();
    }
}
