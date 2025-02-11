use std::path::PathBuf;
use std::pin::Pin;

pub struct Directory {
    pub path: PathBuf,
    pub subdirectories: Vec<Directory>,
    pub files: Vec<PathBuf>,
    pub depth: usize,
}

impl Directory {
    pub fn new(path: PathBuf, depth: Option<usize>) -> Self {
        Self {
            path,
            subdirectories: Vec::new(),
            files: Vec::new(),
            depth: depth.unwrap_or(0),
        }
    }

    async fn scrape(&mut self) -> Result<(), ()> {
        // Scrape the directory for subdirectories and files and add them to the struct

        let mut entries = tokio::fs::read_dir(&self.path).await.unwrap();

        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();

            if path.is_dir() {
                let mut directory = Directory::new(path, Some(self.depth + 1));

                Pin::new(&mut Box::pin(directory.scrape())).await?;

                self.subdirectories.push(directory);
            } else {
                self.files.push(path);
            }
        }

        Ok(())
    }
    
    pub fn max_depth(&self) -> usize {
        let mut max = self.depth;

        for directory in self.subdirectories.iter() {
            let depth = directory.max_depth();

            if depth > max {
                max = depth;
            }
        }

        max
    }

    pub fn get_files_flattened(&self) -> Result<Vec<PathBuf>, ()> {
        let mut files = Vec::new();

        files.append(&mut self.files.clone());

        for directory in self.subdirectories.iter() {
            files.append(&mut directory.get_files_flattened()?);
        }

        Ok(files)
    }
    
    pub fn get_files(&self) -> Vec<Vec<PathBuf>> {
        let mut files = Vec::new();

        files.push(self.files.clone());
        
        for directory in self.subdirectories.iter() {
            files.append(&mut directory.get_files());
        }
        
        files
    }
}

// TODO: Add docs, new prelude, results etc.
