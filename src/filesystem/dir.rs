use crate::any_slices::app::AnySlice;
use crate::compressions::Compressor;
use crate::filesystem::file::File;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

pub struct Dir {
    pub err: Option<String>,
    pub dirname: String,
    pub base_path: String,
    pub full_path: String,
    pub info: Option<fs::FileType>,
    pub mode: Option<fs::Permissions>,
    pub exist: bool,
    pub sub_files: AnySlice<File>,
    pub sub_dirs: AnySlice<Dir>,
}

impl Dir {
    pub fn new(full_path: &str) -> Self {
        let full_path = full_path.to_string();
        let path = Path::new(full_path.as_str());

        // 检查目录是否存在
        let exist = path.exists();

        // 获取目录名
        let dirname = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // 获取上级目录
        let base_path = path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        // 获取元数据（如果目录存在）
        let (info, mode) = if exist {
            match fs::metadata(path) {
                Ok(metadata) => (Some(metadata.file_type()), Some(metadata.permissions())),
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };

        let mut dir = Dir {
            err: None,
            dirname,
            base_path,
            full_path,
            info,
            mode,
            exist,
            sub_files: AnySlice::new(vec![]),
            sub_dirs: AnySlice::new(vec![]),
        };
        if !dir.exist {
            dir.err = Some("目录不存在".to_string());
        }
        dir.list();
        dir
    }

    pub fn new_or_create(full_path: &str) -> Result<Self, String> {
        let path = Path::new(full_path);
        if !path.exists() {
            return match fs::create_dir_all(path) {
                Ok(_) => Ok(Dir::new(full_path)),
                Err(e) => Err(format!("目录不存在且无法创建：{}", e)),
            };
        }

        Ok(Dir::new(full_path))
    }

    /// 检查是否是目录
    pub fn is_dir(&self) -> bool {
        self.info.as_ref().map(|t| t.is_dir()).unwrap_or(false)
    }

    /// 扫描目录，将文件和子目录分别保存到 sub_files 和 sub_dirs 中
    /// 出错时将错误信息保存到 err 字段并停止操作
    pub fn list(&mut self) -> &Self {
        let path = Path::new(&self.full_path);
        if !path.exists() {
            self.err = Some("目录不存在".to_string());
            return self;
        }
        if !path.is_dir() {
            self.err = Some("路径不是目录".to_string());
            return self;
        }

        self.sub_files.clean();
        self.sub_dirs.clean();
        self.err = None;

        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                self.err = Some(format!("读取目录失败: {}", e));
                return self;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    self.err = Some(format!("读取条目失败: {}", e));
                    return self;
                }
            };
            let entry_path = entry.path();
            let full_path_str = entry_path.to_str().unwrap_or("").to_string();

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    self.err = Some(format!("获取文件类型失败: {}", e));
                    return self;
                }
            };

            if file_type.is_file() {
                self.sub_files.push(File::new(&full_path_str));
            } else if file_type.is_dir() {
                self.sub_dirs.push(Dir::new(&full_path_str));
            }
        }

        self
    }

    /// 获取已扫描的子文件列表（需先调用 list）
    pub fn list_files(&self) -> &AnySlice<File> {
        &self.sub_files
    }

    /// 获取已扫描的子目录列表（需先调用 list）
    pub fn list_dirs(&self) -> &AnySlice<Dir> {
        &self.sub_dirs
    }

    /// 获取目录下的条目数量（需先调用 list）
    pub fn count(&self) -> usize {
        self.sub_files.len() + self.sub_dirs.len()
    }

    /// 检查目录是否为空（需先调用 list）
    pub fn is_empty(&self) -> bool {
        self.sub_files.empty() && self.sub_dirs.empty()
    }

    /// 返回上级目录
    pub fn up(mut self) -> Self {
        if self.base_path == "" {
            self.err = Some("无法返回上级目录".to_string());
            return self;
        }

        Dir::new(self.base_path.as_str())
    }

    /// 拼接路径
    pub fn join(mut self, paths: &Vec<&str>) -> Self {
        let path = PathBuf::from(self.full_path.as_str());

        for i in 0..paths.len() {
            let _ = path.join(paths[i]);
        }

        if path.to_str().unwrap_or("").is_empty() {
            self.err = Some("路径无效".to_string());
            return self;
        }

        Dir::new(path.to_str().unwrap())
    }

    /// 删除目录及其所有内容
    pub fn remove(self) -> Result<bool, String> {
        let path = Path::new(&self.full_path);

        if !path.exists() {
            return Err("目录不存在".to_string());
        }

        if !path.is_dir() {
            return Err("路径不是目录".to_string());
        }

        match fs::remove_dir_all(path) {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("删除目录失败: {}", e)),
        }
    }

    /// 创建目录（可指定 Unix 权限模式，如 0o755）
    pub fn create(&mut self, mode: u32) -> &Self {
        let path = Path::new(&self.full_path);

        if path.exists() {
            self.err = Some("目录已存在".to_string());
            return self;
        }

        #[cfg(unix)]
        {
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true).mode(mode);
            match builder.create(path) {
                Ok(_) => self.list(),
                Err(e) => {
                    self.err = Some(format!("创建目录失败: {}", e));
                    return self;
                }
            }
        }

        #[cfg(not(unix))]
        {
            let _ = mode;
            match fs::create_dir_all(path) {
                Ok(_) => self.list(),
                Err(e) => {
                    self.err = Some(format!("创建目录失败：{}", e));
                    return self;
                }
            }
        }
    }

    /// 设置目录权限（Unix mode，如 0o755）
    pub fn set_mode(&self, mode: u32) -> Result<bool, String> {
        let path = Path::new(&self.full_path);

        if !path.exists() {
            return Err("目录不存在".to_string());
        }

        #[cfg(unix)]
        {
            let permissions = fs::Permissions::from_mode(mode);
            match fs::set_permissions(path, permissions) {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("设置权限失败: {}", e)),
            }
        }

        #[cfg(not(unix))]
        {
            let _ = mode;
            Err("当前平台不支持 mode 设置".to_string())
        }
    }

    pub fn mv(&mut self, dst: &str) -> &Self {
        let dst_path = Path::new(dst);
        if dst_path.exists() {
            self.err = Some("目标目录已存在".to_string());
            return self;
        }
        
        match fs::rename(self.full_path.as_str(), dst) {
            Ok(_) => {
                self.full_path = dst.to_string();
                self.list()
            }
            Err(e) => {
                self.err = Some(format!("移动目录失败: {}", e));
                self
            }
        }
    }

    /// 将整个目录打包并压缩为一个文件
    ///
    /// # 参数
    /// - `filename`: 压缩后的文件名
    /// - `compressor`: 压缩算法实现，通过 `CompressorFactory` 创建
    ///
    /// # 示例
    /// ```
    /// use aid::compressions::CompressorFactory;
    /// use aid::filesystem::dir::Dir;
    ///
    /// let dir = Dir::new("/path/to/dir");
    /// let compressor = CompressorFactory::with_lz4();
    /// dir.zip("output.lz4", compressor.as_ref()).unwrap();
    /// ```
    pub fn zip(&self, filename: &str, compressor: &dyn Compressor) -> Result<File, String> {
        let path = Path::new(&self.full_path);
        if !path.exists() || !path.is_dir() {
            return Err("目录不存在或不是有效目录".to_string());
        }

        // 打包目录为归档数据
        let mut archive = Vec::new();
        match self.pack_dir(path, &mut archive) {
            Ok(_) => {}
            Err(e) => return Err(format!("打包目录失败: {}", e)),
        }

        // 压缩归档数据
        let compressed = match compressor.compress(&archive) {
            Ok(data) => data,
            Err(e) => return Err(format!("压缩失败: {}", e)),
        };

        // 写入文件
        match fs::write(filename, &compressed) {
            Ok(_) => Ok(File::new(filename)),
            Err(e) => Err(format!("写入文件失败: {}", e)),
        }
    }

    /// 递归收集目录下所有文件，打包为 TLV 格式归档
    fn pack_dir(&self, base: &Path, output: &mut Vec<u8>) -> Result<(), std::io::Error> {
        let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
        Self::collect_files(base, base, &mut entries)?;

        // 写入文件数量 (4 bytes)
        let file_count = entries.len() as u32;
        output.extend_from_slice(&file_count.to_le_bytes());

        // 写入每个文件: path_len + path + data_len + data
        for (relative_path, data) in &entries {
            let path_bytes = relative_path.as_bytes();
            let path_len = path_bytes.len() as u32;
            let data_len = data.len() as u64;

            output.extend_from_slice(&path_len.to_le_bytes());
            output.extend_from_slice(path_bytes);
            output.extend_from_slice(&data_len.to_le_bytes());
            output.extend_from_slice(data);
        }

        Ok(())
    }

    /// 递归收集目录下所有文件及其相对路径
    fn collect_files(
        base: &Path,
        current: &Path,
        entries: &mut Vec<(String, Vec<u8>)>,
    ) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let relative = path
                    .strip_prefix(base)
                    .unwrap()
                    .to_str()
                    .unwrap_or("")
                    .to_string();
                let data = fs::read(&path)?;
                entries.push((relative, data));
            } else if path.is_dir() {
                Self::collect_files(base, &path, entries)?;
            }
        }
        Ok(())
    }

    /// 从压缩文件解压并还原目录
    ///
    /// # 参数
    /// - `filename`: 压缩文件路径
    /// - `output_dir`: 解压输出目录路径
    /// - `compressor`: 压缩算法实现，需与压缩时使用的算法一致
    ///
    /// # 示例
    /// ```
    /// use aid::compressions;
    /// use aid::filesystem::dir::Dir;
    ///
    /// let compressor = compressions::with_lz4();
    /// let dir = Dir::unzip("output.lz4", "/path/to/restore", compressor.as_ref()).unwrap();
    /// ```
    pub fn unzip(filename: &str, output_dir: &str, compressor: &dyn Compressor) -> Result<Dir, String> {
        // 读取压缩文件
        let compressed = match fs::read(filename) {
            Ok(data) => data,
            Err(e) => return Err(format!("读取压缩文件失败: {}", e)),
        };

        // 解压
        let archive = match compressor.decompress(&compressed) {
            Ok(data) => data,
            Err(e) => return Err(format!("解压失败: {}", e)),
        };

        // 创建输出目录
        let output = Path::new(output_dir);
        if !output.exists() {
            if let Err(e) = fs::create_dir_all(output) {
                return Err(format!("创建输出目录失败: {}", e));
            }
        }

        // 解析归档并还原文件
        match Self::unpack_dir(output, &archive) {
            Ok(_) => Ok(Dir::new(output_dir)),
            Err(e) => Err(format!("解包失败: {}", e)),
        }
    }

    /// 解析 TLV 格式归档并还原目录结构
    fn unpack_dir(output: &Path, archive: &[u8]) -> Result<(), String> {
        if archive.len() < 4 {
            return Err("归档数据无效: 数据过短".to_string());
        }

        let mut offset = 0;

        // 读取文件数量
        let file_count = u32::from_le_bytes(
            archive[offset..offset + 4].try_into().unwrap()
        ) as usize;
        offset += 4;

        for _ in 0..file_count {
            // 读取路径长度
            if offset + 4 > archive.len() {
                return Err("归档数据截断: 路径长度缺失".to_string());
            }
            let path_len = u32::from_le_bytes(
                archive[offset..offset + 4].try_into().unwrap()
            ) as usize;
            offset += 4;

            // 读取路径
            if offset + path_len > archive.len() {
                return Err("归档数据截断: 路径数据缺失".to_string());
            }
            let relative_path = std::str::from_utf8(&archive[offset..offset + path_len])
                .map_err(|_| "归档数据无效: 文件名编码错误".to_string())?;
            offset += path_len;

            // 读取数据长度
            if offset + 8 > archive.len() {
                return Err("归档数据截断: 数据长度缺失".to_string());
            }
            let data_len = u64::from_le_bytes(
                archive[offset..offset + 8].try_into().unwrap()
            ) as usize;
            offset += 8;

            // 读取数据
            if offset + data_len > archive.len() {
                return Err("归档数据截断: 文件数据缺失".to_string());
            }
            let data = &archive[offset..offset + data_len];
            offset += data_len;

            // 写入文件（自动创建父目录）
            let file_path = output.join(relative_path);
            if let Some(parent) = file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return Err(format!("创建目录失败: {}", e));
                }
            }
            if let Err(e) = fs::write(&file_path, data) {
                return Err(format!("写入文件失败: {}", e));
            }
        }

        Ok(())
    }
}
