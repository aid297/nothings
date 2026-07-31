use crate::compressions::Compressor;
use std::fs;
use std::path::Path;

pub struct File {
    pub err: Option<String>,
    pub filename: String,
    pub base_path: String,
    pub full_path: String,
    pub size: u64,
    pub info: Option<fs::FileType>,
    pub mode: Option<fs::Permissions>,
    pub exist: bool,
    pub ext: String,
}

impl File {
    pub fn new(full_path: &str) -> Self {
        let full_path = full_path.to_string();
        let path = Path::new(full_path.as_str());

        // 检查文件是否存在
        let exist = path.exists();

        // 获取文件名
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // 获取扩展名
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        // 获取上级目录
        let base_path = path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        // 获取元数据（如果文件存在）
        let (size, info, mode) = if exist {
            match fs::metadata(path) {
                Ok(metadata) => (
                    metadata.len(),
                    Some(metadata.file_type()),
                    Some(metadata.permissions()),
                ),
                Err(_) => (0, None, None),
            }
        } else {
            (0, None, None)
        };

        File {
            err: None,
            filename,
            base_path,
            full_path,
            size,
            info,
            mode,
            exist,
            ext,
        }
    }

    /// 检查是否是普通文件
    pub fn is_file(&self) -> bool {
        self.info.as_ref().map(|t| t.is_file()).unwrap_or(false)
    }

    /// 检查是否是目录
    pub fn is_dir(&self) -> bool {
        self.info.as_ref().map(|t| t.is_dir()).unwrap_or(false)
    }

    /// 将数据写入文件（覆盖已有内容）
    pub fn write(&mut self, data: &[u8]) -> Result<bool, String> {
        match fs::write(&self.full_path, data) {
            Ok(_) => {
                self.size = data.len() as u64;
                self.exist = true;
                // 重新获取元数据
                if let Ok(metadata) = fs::metadata(&self.full_path) {
                    self.info = Some(metadata.file_type());
                    self.mode = Some(metadata.permissions());
                }
                Ok(true)
            }
            Err(e) => Err(format!("写入文件失败: {}", e)),
        }
    }

    /// 读取文件内容
    pub fn read(&self) -> Result<Vec<u8>, String> {
        if !self.exist {
            return Err("文件不存在".to_string());
        }
        match fs::read(&self.full_path) {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("读取文件失败: {}", e)),
        }
    }

    /// 删除文件
    pub fn remove(self) -> Result<bool, String> {
        let path = Path::new(&self.full_path);
        if !path.exists() {
            return Err("文件不存在".to_string());
        }
        if !path.is_file() {
            return Err("路径不是普通文件".to_string());
        }
        match fs::remove_file(path) {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("删除文件失败: {}", e)),
        }
    }

    /// 移动/重命名文件
    pub fn mv(&mut self, dst: &str) -> &Self {
        let dst_path = Path::new(dst);
        if dst_path.exists() {
            self.err = Some("目标文件已存在".to_string());
            return self;
        }
        match fs::rename(&self.full_path, dst) {
            Ok(_) => {
                // 更新路径信息
                self.full_path = dst.to_string();
                self.filename = dst_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                self.base_path = dst_path
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_string();
                self.ext = dst_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string();
                self
            }
            Err(e) => {
                self.err = Some(format!("移动文件失败: {}", e));
                self
            }
        }
    }

    /// 将文件压缩为另一个文件
    pub fn zip(&self, filename: &str, compressor: &dyn Compressor) -> Result<File, String> {
        if !self.exist {
            return Err("文件不存在".to_string());
        }
        // 读取原文件
        let data = match fs::read(&self.full_path) {
            Ok(d) => d,
            Err(e) => return Err(format!("读取文件失败: {}", e)),
        };
        // 压缩
        let compressed = match compressor.compress(&data) {
            Ok(d) => d,
            Err(e) => return Err(format!("压缩失败: {}", e)),
        };
        // 写入目标文件
        match fs::write(filename, &compressed) {
            Ok(_) => Ok(File::new(filename)),
            Err(e) => Err(format!("写入文件失败: {}", e)),
        }
    }

    /// 从压缩文件解压并创建新文件
    pub fn unzip(
        filename: &str,
        output_file: &str,
        compressor: &dyn Compressor,
    ) -> Result<File, String> {
        // 读取压缩文件
        let compressed = match fs::read(filename) {
            Ok(data) => data,
            Err(e) => return Err(format!("读取压缩文件失败: {}", e)),
        };
        // 解压
        let data = match compressor.decompress(&compressed) {
            Ok(d) => d,
            Err(e) => return Err(format!("解压失败: {}", e)),
        };
        // 写入输出文件
        match fs::write(output_file, &data) {
            Ok(_) => Ok(File::new(output_file)),
            Err(e) => Err(format!("写入文件失败: {}", e)),
        }
    }
}
