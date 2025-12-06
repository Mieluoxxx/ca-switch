use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// 配置类别路径
#[derive(Debug, Clone)]
pub struct CategoryPaths {
    pub name: String,
    pub files: HashMap<String, PathBuf>,
    pub directories: HashMap<String, PathBuf>,
}

/// 文件检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCheckResult {
    pub category: String,
    pub name: String,
    pub files: HashMap<String, FileInfo>,
    pub directories: HashMap<String, DirInfo>,
    pub total_exists: usize,
    pub total_count: usize,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub exists: bool,
    pub size: u64,
}

/// 目录信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirInfo {
    pub path: String,
    pub exists: bool,
    pub file_count: usize,
}

/// 备份数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub category: String,
    pub timestamp: String,
    pub files: HashMap<String, String>,  // 文件名 -> 内容
    pub metadata: BackupMetadata,
}

/// 备份元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub version: String,
    pub created_at: String,
    pub hostname: String,
    pub total_files: usize,
    pub total_size: u64,
}

/// 文件管理器
pub struct FileManager {
    home_dir: PathBuf,
}

impl FileManager {
    /// 创建新的文件管理器
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| crate::error::CliError::Config("无法获取用户主目录".to_string()))?;

        Ok(Self { home_dir })
    }

    /// 初始化配置路径
    pub fn init_config_paths(&self) -> HashMap<String, CategoryPaths> {
        let mut paths = HashMap::new();

        // CC-CLI 配置
        let cc_cli_files = HashMap::new();
        let mut cc_cli_dirs = HashMap::new();
        cc_cli_dirs.insert(
            ".ca-switch".to_string(),
            self.home_dir.join(".ca-switch"),
        );
        paths.insert(
            "ccCli".to_string(),
            CategoryPaths {
                name: "CA-Switch配置".to_string(),
                files: cc_cli_files,
                directories: cc_cli_dirs,
            },
        );

        // Claude Code 配置
        let mut claude_files = HashMap::new();
        claude_files.insert(
            "settings.json".to_string(),
            self.home_dir.join(".claude").join("settings.json"),
        );
        claude_files.insert(
            "CLAUDE.md".to_string(),
            self.home_dir.join(".claude").join("CLAUDE.md"),
        );

        let mut claude_dirs = HashMap::new();
        claude_dirs.insert(
            "agents".to_string(),
            self.home_dir.join(".claude").join("agents"),
        );
        claude_dirs.insert(
            "commands".to_string(),
            self.home_dir.join(".claude").join("commands"),
        );
        claude_dirs.insert(
            "skills".to_string(),
            self.home_dir.join(".claude").join("skills"),
        );

        paths.insert(
            "claudeCode".to_string(),
            CategoryPaths {
                name: "Claude Code配置".to_string(),
                files: claude_files,
                directories: claude_dirs,
            },
        );

        // Codex 配置
        let mut codex_files = HashMap::new();
        codex_files.insert(
            "config.toml".to_string(),
            self.find_codex_file("config.toml"),
        );
        codex_files.insert(
            "auth.json".to_string(),
            self.find_codex_file("auth.json"),
        );
        codex_files.insert(
            "AGENTS.md".to_string(),
            self.find_codex_file("AGENTS.md"),
        );

        paths.insert(
            "codex".to_string(),
            CategoryPaths {
                name: "Codex配置".to_string(),
                files: codex_files,
                directories: HashMap::new(),
            },
        );

        // Gemini 配置
        let mut gemini_files = HashMap::new();
        gemini_files.insert(
            ".env".to_string(),
            self.home_dir.join(".gemini").join(".env"),
        );
        gemini_files.insert(
            "settings.json".to_string(),
            self.home_dir.join(".gemini").join("settings.json"),
        );

        paths.insert(
            "gemini".to_string(),
            CategoryPaths {
                name: "Gemini配置".to_string(),
                files: gemini_files,
                directories: HashMap::new(),
            },
        );

        // OpenCode 配置
        let mut opencode_files = HashMap::new();
        opencode_files.insert(
            "opencode.json".to_string(),
            self.home_dir.join(".opencode").join("opencode.json"),
        );

        paths.insert(
            "opencode".to_string(),
            CategoryPaths {
                name: "OpenCode配置".to_string(),
                files: opencode_files,
                directories: HashMap::new(),
            },
        );

        paths
    }

    /// 查找 Codex 配置文件
    fn find_codex_file(&self, filename: &str) -> PathBuf {
        let possible_paths = vec![
            self.home_dir.join(".codex").join(filename),
            self.home_dir.join(".config").join("codex").join(filename),
            self.home_dir.join("Documents").join("codex").join(filename),
        ];

        for path in possible_paths {
            if path.exists() {
                return path;
            }
        }

        // 默认返回 ~/.codex/
        self.home_dir.join(".codex").join(filename)
    }

    /// 检查配置类别的文件存在性
    pub async fn check_category_files(&self, category: &str) -> Result<FileCheckResult> {
        let config_paths = self.init_config_paths();
        let paths = config_paths
            .get(category)
            .ok_or_else(|| crate::error::CliError::Config(format!("未知的配置类别: {category}")))?;

        let mut result = FileCheckResult {
            category: category.to_string(),
            name: paths.name.clone(),
            files: HashMap::new(),
            directories: HashMap::new(),
            total_exists: 0,
            total_count: 0,
        };

        // 检查文件
        for (name, path) in &paths.files {
            let exists = path.exists();
            let size = if exists {
                fs::metadata(path).await.ok().map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            result.files.insert(
                name.clone(),
                FileInfo {
                    path: path.display().to_string(),
                    exists,
                    size,
                },
            );

            result.total_count += 1;
            if exists {
                result.total_exists += 1;
            }
        }

        // 检查目录
        for (name, path) in &paths.directories {
            let exists = path.exists();
            let file_count = if exists {
                self.count_files_in_dir(path).await.unwrap_or(0)
            } else {
                0
            };

            result.directories.insert(
                name.clone(),
                DirInfo {
                    path: path.display().to_string(),
                    exists,
                    file_count,
                },
            );

            result.total_count += 1;
            if exists {
                result.total_exists += 1;
            }
        }

        Ok(result)
    }

    /// 统计目录中的文件数量
    async fn count_files_in_dir(&self, dir: &PathBuf) -> Result<usize> {
        let mut count = 0;
        let mut entries = fs::read_dir(dir).await?;

        while let Some(_entry) = entries.next_entry().await? {
            count += 1;
        }

        Ok(count)
    }

    /// 收集备份数据
    pub async fn collect_backup_data(&self, category: &str) -> Result<BackupData> {
        let config_paths = self.init_config_paths();
        let paths = config_paths
            .get(category)
            .ok_or_else(|| crate::error::CliError::Config(format!("未知的配置类别: {category}")))?;

        let mut files_content = HashMap::new();
        let mut total_size = 0u64;

        // 收集文件内容
        for (name, path) in &paths.files {
            if path.exists() {
                match fs::read_to_string(path).await {
                    Ok(content) => {
                        total_size += content.len() as u64;
                        files_content.insert(name.clone(), content);
                    }
                    Err(e) => {
                        eprintln!("读取文件 {} 失败: {}", path.display(), e);
                    }
                }
            }
        }

        // 收集目录内容
        for (dir_name, dir_path) in &paths.directories {
            if dir_path.exists() {
                let dir_files = self.collect_directory_files(dir_path).await?;
                for (file_name, content) in dir_files {
                    total_size += content.len() as u64;
                    files_content.insert(format!("{dir_name}/{file_name}"), content);
                }
            }
        }

        // 创建备份数据
        let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(BackupData {
            category: category.to_string(),
            timestamp: timestamp.clone(),
            files: files_content.clone(),
            metadata: BackupMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                hostname,
                total_files: files_content.len(),
                total_size,
            },
        })
    }

    /// 递归收集目录中的所有文件
    async fn collect_directory_files(&self, dir: &PathBuf) -> Result<HashMap<String, String>> {
        let mut files = HashMap::new();

        self.collect_dir_recursive(dir, dir, &mut files).await?;

        Ok(files)
    }

    /// 递归收集目录文件的辅助函数
    #[allow(clippy::only_used_in_recursion)]
    fn collect_dir_recursive<'a>(
        &'a self,
        base_dir: &'a PathBuf,
        current_dir: &'a PathBuf,
        files: &'a mut HashMap<String, String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let mut entries = fs::read_dir(current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_file() {
                    // 计算相对路径
                    let relative_path = path
                        .strip_prefix(base_dir)
                        .unwrap_or(&path)
                        .display()
                        .to_string();

                    if let Ok(content) = fs::read_to_string(&path).await {
                        files.insert(relative_path, content);
                    }
                } else if path.is_dir() {
                    // 递归处理子目录
                    self.collect_dir_recursive(base_dir, &path, files).await?;
                }
            }

            Ok(())
        })
    }

    /// 恢复备份数据
    #[allow(dead_code)]
    pub async fn restore_backup_data(&self, category: &str, backup_data: &BackupData) -> Result<()> {
        let config_paths = self.init_config_paths();
        let paths = config_paths
            .get(category)
            .ok_or_else(|| crate::error::CliError::Config(format!("未知的配置类别: {category}")))?;

        // 恢复文件
        for (file_name, content) in &backup_data.files {
            // 判断是普通文件还是目录中的文件
            if file_name.contains('/') {
                // 目录中的文件
                let parts: Vec<&str> = file_name.splitn(2, '/').collect();
                if parts.len() == 2 {
                    let dir_name = parts[0];
                    let relative_path = parts[1];

                    if let Some(base_dir) = paths.directories.get(dir_name) {
                        let file_path = base_dir.join(relative_path);

                        // 确保父目录存在
                        if let Some(parent) = file_path.parent() {
                            fs::create_dir_all(parent).await?;
                        }

                        fs::write(&file_path, content).await?;
                    }
                }
            } else {
                // 普通文件
                if let Some(file_path) = paths.files.get(file_name) {
                    // 确保父目录存在
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).await?;
                    }

                    fs::write(file_path, content).await?;
                }
            }
        }

        Ok(())
    }

    /// 获取所有配置类别
    #[allow(dead_code)]
    pub fn get_categories(&self) -> Vec<String> {
        self.init_config_paths().keys().cloned().collect()
    }

    /// 格式化文件大小
    pub fn format_file_size(&self, bytes: u64) -> String {
        if bytes == 0 {
            return "0 B".to_string();
        }

        let k = 1024f64;
        let sizes = ["B", "KB", "MB", "GB"];
        let i = (bytes as f64).log(k).floor() as usize;
        let size = bytes as f64 / k.powi(i as i32);

        format!("{:.2} {}", size, sizes[i.min(sizes.len() - 1)])
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new().expect("Failed to create FileManager")
    }
}
