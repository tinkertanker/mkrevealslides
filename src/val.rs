use std::path::PathBuf;

/// Useful validators for clap arguments

pub type ValError = String;

/// Checks whether the given path exists and is a file
pub fn validate_file_path(s: &str) -> Result<PathBuf, ValError> {
    let pb = PathBuf::from(s);
    if !pb.exists() {
        return Err(format!("File {} does not exist", pb.display()));
    }
    if pb.is_file() {
         Ok(pb)
    } else {
        Err(format!("{} is not a file", s))
    }
}

/// Checks whether the given path is a directory and if it exists
pub fn validate_dir_path(s: &str) -> Result<PathBuf, ValError> {
    let pb = PathBuf::from(s);
    if !pb.exists() {
        return Err(format!("Directory {} does not exist", pb.display()));
    }
    if pb.is_dir() {
         Ok(pb)
    } else {
        Err(format!("{} is not a directory", s))
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;
    use super::*;
    use tempfile::{tempdir};

    #[test]
    fn test_validate_file_path() {
        let tmp_dir = tempdir().unwrap();
        let tmp_file = tmp_dir.path().join("tmp.txt");
        let tmp_dir_1 = tmp_dir.path().join("tmp_dir");

        assert!(validate_file_path(tmp_file.to_str().unwrap()).is_err());

        File::create(&tmp_file).unwrap();
        assert!(validate_file_path(tmp_file.to_str().unwrap()).is_ok());

        fs::create_dir(&tmp_dir_1).unwrap();
        assert!(validate_file_path(tmp_dir_1.to_str().unwrap()).is_err());
    }

    #[test]
    fn test_validate_dir_path() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("tmp.txt");
        let dir_1 = dir.path().join("tmp_dir");

        assert!(validate_dir_path(dir.path().to_str().unwrap()).is_ok());
        assert!(validate_dir_path(file.to_str().unwrap()).is_err());
        assert!(validate_dir_path(dir_1.to_str().unwrap()).is_err());

        File::create(&file).unwrap();
        assert!(validate_dir_path(file.to_str().unwrap()).is_err());

        fs::create_dir(&dir_1).unwrap();
        assert!(validate_dir_path(dir_1.to_str().unwrap()).is_ok());
    }
}