use std::{env, fs::{read_dir, ReadDir, File}, io::Read};

pub const UPLOAD_OP: u8 = 0x01;
pub const LISTDIRS_OP: u8 = 0x02;
pub const LISTFILES_OP: u8 = 0x03;
pub const DOWNLOAD_OP: u8 = 0x04;

pub const UPLOAD: &str = "upload";
pub const LISTDIRS: &str = "listdirs";
pub const LISTFILES: &str = "listfiles";
pub const DOWNLOAD: &str = "download";
pub const GENERATE: &str = "generate";

pub struct FileItem {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

pub struct UploadOp {
    pub files_count: u32,
    pub dir_path: String,
    read_dir: ReadDir,
}

impl UploadOp {
    pub fn new(files_count: u32, dir_path: String) -> Self {
        let current_dir = read_dir(&dir_path).unwrap();
        Self { files_count, dir_path , read_dir: current_dir}
    }
}

impl TryInto<UploadOp> for &Vec<String> {
    type Error = &'static str;

    fn try_into(self) -> Result<UploadOp, Self::Error> {
        if self[1] != UPLOAD {
            return Err("Invalid operation");
        }

        let current_dir = env::current_dir().unwrap();
        let dir = current_dir.join(&self[2]);
        
        if !dir.exists() {
            return Err("Directory does not exist");
        }

        let n_files = read_dir(&dir)
            .unwrap()
            .filter(|f| {
                let f = f.as_ref().unwrap();
                f.metadata().unwrap().is_file()
            })
            .count() as u32;

        Ok(UploadOp::new(n_files, dir.to_str().unwrap().to_string()))   
    }
}

impl Iterator for UploadOp {
    type Item = FileItem;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.read_dir.next() {
          Some(f) => {
            match f {
                Ok(f) => {
                    if f.metadata().unwrap().is_dir() {
                        self.next()
                    } else {
                        let file_name = f.file_name().into_string().unwrap();
                        let mut file = File::open(f.path()).unwrap();
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap();
                        Some(FileItem { file_name, bytes: buffer })
                    }
                },
                Err(_) => None,
            }
          },
          None => None,
        }
    }
}