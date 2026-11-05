use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct ScanResult{
    total_size: u64,
    file_count: u64,
    dir_count: u64,
    largest_files: Vec<(PathBuf, u64)>,
    extension_usage: HashMap<String, u64>,
}
impl ScanResult{
    pub fn total_size(&self) -> u64{
        self.total_size
    }

    pub fn file_count(&self) -> u64{
        self.file_count
    }

    pub fn dir_count(&self) -> u64{
        self.dir_count
    }

    pub fn largest_files(&self) -> &[(PathBuf, u64)] {
        &self.largest_files
    }

    pub fn extension_usage(&self) -> &HashMap<String, u64>{
        &self.extension_usage
    }
}



pub fn scan_directory(path: &Path) -> Result<ScanResult, std::io::Error>{
    let mut result = ScanResult::default();
    scan_into(path, &mut result)?;
    Ok(result)
}

fn scan_into(path: &Path, result: &mut ScanResult) -> std::io::Result<()>{
    let directory = fs::read_dir(path)?;
    for entry in directory{
        let entry = entry?;

        let path_current = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir(){
            result.dir_count +=1;
            scan_into(&path_current, result)?;
        } else if file_type.is_file(){
            let metadata = entry.metadata()?;
            let size = metadata.len();

            result.total_size += size;
            result.file_count += 1;

            result.largest_files.push((path_current.clone(), size));
            result.largest_files.sort_unstable_by(|a, b| b.1.cmp(&a.1));
            result.largest_files.truncate(10);


            let extension = path_current.extension()
                .map(|ext| ext.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_else(|| "(no extension)".to_string());

            *result.extension_usage.entry(extension).or_insert(0) += size;

        }
    }

    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_directory_scan(){
        let path = std::env::temp_dir().join("DirectoryFootprintTest");

        let result = scan_directory(&path).unwrap();

        assert_eq!(result.total_size, 42340);
        assert_eq!(result.file_count, 13);
        assert_eq!(result.dir_count, 2);
        assert_eq!(result.extension_usage.get("jpg"), Some(&1024));
        assert_eq!(result.extension_usage.get("zip"), Some(&2048));
        assert_eq!(result.extension_usage.get("mp4"), Some(&4096));
        assert_eq!(result.largest_files.len(), 10);

        let sizes: Vec<u64> = result.largest_files
            .iter()
            .map(|(_, size)| *size)
            .collect();

        assert_eq!(
            sizes,
            vec![8192, 7168, 6144, 5120, 4096,
                 3072, 2560, 2048, 1536, 1024]
        );

    }
}