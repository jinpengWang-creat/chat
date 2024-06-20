use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::error::AppError;

use super::ChatFile;
use sha1::{Digest, Sha1};

impl ChatFile {
    pub fn new(filename: &str, data: &[u8], ws_id: u64) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.rsplit('.').next().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}/{}", self.ws_id, self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir
            .join(self.ws_id.to_string())
            .join(self.hash_to_path())
    }

    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}.{}", part1, part2, part3, self.ext)
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix("/files/")
            .ok_or(AppError::ChatFile("Invalid file path".to_string()))?;
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFile("Invalid file path".to_string()));
        }
        let ws_id = parts[0]
            .parse::<u64>()
            .map_err(|_| AppError::ChatFile(format!("Invalid workspace id: {:?}", parts[0])))?;
        let (ext, part3) = parts[3]
            .rsplit_once('.')
            .ok_or(AppError::ChatFile("Invalid file name".to_string()))?;
        let hash = format!("{}{}{}", parts[1], parts[2], part3);

        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chat_file() {
        let filename = "README.MD";
        let data = b"hello world";
        let file = ChatFile::new(filename, data, 1);
        assert_eq!(file.ext, "MD");
    }
}
