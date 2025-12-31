use crate::error::{CliError, Result};
use crate::ui::{confirm, show_error, show_info, show_success, show_warning};
use chrono::TimeZone;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Password};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// WebDAV 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDAVConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

/// WebDAV 文件信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WebDAVFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub category: String,
    pub timestamp: Option<chrono::DateTime<chrono::Local>>,
}

/// 健康状态信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HealthStatus {
    pub connected: bool,
    pub latency_ms: Option<u64>,
    pub server_type: String,
    pub error_message: Option<String>,
}

/// 存储信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StorageInfo {
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub categories: std::collections::HashMap<String, usize>,
}

impl WebDAVFile {
    /// 从文件名中提取分类和时间戳
    fn parse_filename(name: &str) -> (String, Option<chrono::DateTime<chrono::Local>>) {
        // 文件名格式: {category}_{timestamp}.json
        // 例如: opencode_20250101_120000.json

        let parts: Vec<&str> = name.trim_end_matches(".json").split('_').collect();

        if parts.len() >= 3 {
            let category = parts[0].to_string();
            let datetime_str = format!("{}_{}", parts[1], parts[2]);

            // 尝试解析时间戳
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y%m%d_%H%M%S") {
                let timestamp = chrono::Local.from_local_datetime(&dt).single();
                return (category, timestamp);
            }
        }

        // 如果解析失败，返回默认值
        ("unknown".to_string(), None)
    }
}

/// WebDAV 客户端管理器
pub struct WebDAVClient {
    config_path: PathBuf,
    client: Option<Client>,
    config: Option<WebDAVConfig>,
}

impl WebDAVClient {
    /// 创建新的 WebDAV 客户端
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| CliError::Config("无法获取用户主目录".to_string()))?;

        let config_path = home_dir.join(".ca-switch").join("webdav-config.json");

        Ok(Self {
            config_path,
            client: None,
            config: None,
        })
    }

    /// 初始化 WebDAV 客户端
    pub async fn initialize(&mut self) -> Result<()> {
        // 尝试加载已保存的配置
        if self.load_saved_config().await? {
            self.test_connection().await?;
            show_success("✅ WebDAV 客户端初始化成功");
            return Ok(());
        }

        // 如果没有配置，提示用户配置
        self.setup_webdav().await?;
        show_success("✅ WebDAV 客户端初始化成功");

        Ok(())
    }

    /// 加载已保存的配置
    async fn load_saved_config(&mut self) -> Result<bool> {
        if !self.config_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&self.config_path).await?;
        self.config = serde_json::from_str(&content).ok();

        if let Some(ref config) = self.config {
            self.client = Some(self.create_client(config)?);
            show_success("✅ 已加载保存的 WebDAV 配置");
            Ok(true)
        } else {
            show_warning("⚠️ 加载 WebDAV 配置失败，需要重新设置");
            Ok(false)
        }
    }

    /// 设置 WebDAV 连接
    async fn setup_webdav(&mut self) -> Result<()> {
        println!("\n{}", style("🔧 WebDAV 配置向导").cyan().bold());
        println!();

        println!("{}", style("支持的 WebDAV 服务：").white());
        println!("{}", style("• 坚果云 (https://dav.jianguoyun.com/dav/) 优选 其他未测试").dim());
        println!("{}", style("• 其他支持 WebDAV 的云存储服务").dim());
        println!("{}", style("━".repeat(60)).dim());
        println!();

        loop {
            let url: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("WebDAV 服务器地址")
                .default("https://dav.jianguoyun.com/dav/".to_string())
                .validate_with(|input: &String| {
                    if input.trim().is_empty() {
                        Err("WebDAV 地址不能为空")
                    } else if !input.starts_with("http://") && !input.starts_with("https://") {
                        Err("请输入有效的 HTTP/HTTPS 地址")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?;

            let username: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("用户名")
                .validate_with(|input: &String| {
                    if input.trim().is_empty() {
                        Err("用户名不能为空")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?;

            let password: String = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("密码 (或应用专用密码)")
                .validate_with(|input: &String| {
                    if input.trim().is_empty() {
                        Err("密码不能为空")
                    } else {
                        Ok(())
                    }
                })
                .interact()?;

            // 测试连接
            println!();
            show_info("🔍 测试 WebDAV 连接...");

            let config = WebDAVConfig {
                url: url.clone(),
                username: username.clone(),
                password: password.clone(),
            };

            match self.test_config(&config).await {
                Ok(_) => {
                    show_success("✅ WebDAV 连接测试成功");

                    // 保存配置
                    self.config = Some(config.clone());
                    self.client = Some(self.create_client(&config)?);
                    self.save_config().await?;

                    // 确保备份目录存在
                    self.ensure_backup_directory().await?;

                    break;
                }
                Err(e) => {
                    show_error(&format!("❌ WebDAV 连接测试失败: {e}"));

                    println!();
                    println!("{}", style("💡 常见问题解决：").yellow());
                    println!("{}", style("• 检查 WebDAV 地址是否正确").dim());
                    println!("{}", style("• 确认用户名和密码是否正确").dim());
                    println!("{}", style("• 某些服务需要应用专用密码（如坚果云）").dim());
                    println!("{}", style("• 检查网络连接是否正常").dim());
                    println!();

                    if !confirm("是否重新配置？", true)? {
                        return Err(CliError::Config("WebDAV 配置失败".to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    /// 创建 HTTP 客户端
    fn create_client(&self, config: &WebDAVConfig) -> Result<Client> {
        let auth_value = format!("{}:{}", config.username, config.password);
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            auth_value.as_bytes(),
        );

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Basic {encoded}"))
                .map_err(|e| CliError::Config(format!("创建认证头失败: {e}")))?,
        );

        Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| CliError::Config(format!("创建 HTTP 客户端失败: {e}")))
    }

    /// 测试配置连接
    async fn test_config(&self, config: &WebDAVConfig) -> Result<()> {
        let client = self.create_client(config)?;

        let method = reqwest::Method::from_bytes(b"PROPFIND")
            .map_err(|e| CliError::Config(format!("创建 PROPFIND 方法失败: {e}")))?;

        let response = client
            .request(method, &config.url)
            .header("Depth", "0")
            .send()
            .await
            .map_err(|e| CliError::WebDav(format!("连接失败: {e}")))?;

        if response.status().is_success() || response.status().as_u16() == 207 {
            Ok(())
        } else {
            Err(CliError::WebDav(format!(
                "连接测试失败: HTTP {}",
                response.status()
            )))
        }
    }

    /// 保存配置到本地
    async fn save_config(&self) -> Result<()> {
        if let Some(ref config) = self.config {
            // 确保配置目录存在
            if let Some(parent) = self.config_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let content = serde_json::to_string_pretty(config)?;
            fs::write(&self.config_path, content).await?;

            show_success("✅ WebDAV 配置已保存");
        }

        Ok(())
    }

    /// 确保备份目录存在
    async fn ensure_backup_directory(&self) -> Result<()> {
        let backup_dir = "/ca-switch-backups";

        if let Some(ref client) = self.client {
            if let Some(ref config) = self.config {
                let url = format!("{}{}", config.url.trim_end_matches('/'), backup_dir);

                // 尝试创建目录（如果已存在会返回 405 Method Not Allowed，这是正常的）
                let method = reqwest::Method::from_bytes(b"MKCOL")
                    .map_err(|e| CliError::Config(format!("创建 MKCOL 方法失败: {e}")))?;

                let response = client
                    .request(method, &url)
                    .send()
                    .await
                    .map_err(|e| CliError::WebDav(format!("创建备份目录失败: {e}")))?;

                if response.status().is_success() {
                    show_success(&format!("✅ 创建备份目录: {backup_dir}"));
                } else if response.status().as_u16() == 405 {
                    show_success(&format!("✅ 备份目录已存在: {backup_dir}"));
                } else {
                    return Err(CliError::WebDav(format!(
                        "创建备份目录失败: HTTP {}",
                        response.status()
                    )));
                }
            }
        }

        Ok(())
    }

    /// 测试连接状态
    pub async fn test_connection(&self) -> Result<()> {
        if let Some(ref client) = self.client {
            if let Some(ref config) = self.config {
                let method = reqwest::Method::from_bytes(b"PROPFIND")
                    .map_err(|e| CliError::Config(format!("创建 PROPFIND 方法失败: {e}")))?;

                let response = client
                    .request(method, &config.url)
                    .header("Depth", "0")
                    .send()
                    .await
                    .map_err(|e| CliError::WebDav(format!("连接测试失败: {e}")))?;

                if response.status().is_success() || response.status().as_u16() == 207 {
                    Ok(())
                } else {
                    Err(CliError::WebDav(format!(
                        "连接测试失败: HTTP {}",
                        response.status()
                    )))
                }
            } else {
                Err(CliError::Config("WebDAV 未配置".to_string()))
            }
        } else {
            Err(CliError::Config("WebDAV 客户端未初始化".to_string()))
        }
    }

    /// 上传备份文件
    pub async fn upload_backup(
        &mut self,
        file_name: &str,
        data: &serde_json::Value,
    ) -> Result<String> {
        if self.client.is_none() {
            self.initialize().await?;
        }

        let content = serde_json::to_string_pretty(data)?;
        let remote_path = format!("/ca-switch-backups/{file_name}");

        if let Some(ref client) = self.client {
            if let Some(ref config) = self.config {
                let url = format!("{}{}", config.url.trim_end_matches('/'), remote_path);

                println!();
                show_info(&format!("📤 上传备份文件: {file_name}"));

                let response = client
                    .put(&url)
                    .header("Content-Type", "application/json")
                    .body(content)
                    .send()
                    .await
                    .map_err(|e| CliError::WebDav(format!("上传失败: {e}")))?;

                if response.status().is_success() || response.status().as_u16() == 201 {
                    show_success(&format!("✅ 上传成功: {file_name}"));
                    Ok(remote_path)
                } else {
                    Err(CliError::WebDav(format!(
                        "上传失败: HTTP {}",
                        response.status()
                    )))
                }
            } else {
                Err(CliError::Config("WebDAV 未配置".to_string()))
            }
        } else {
            Err(CliError::Config("WebDAV 客户端未初始化".to_string()))
        }
    }

    /// 列出所有备份文件
    pub async fn list_backups(&mut self) -> Result<Vec<WebDAVFile>> {
        if self.client.is_none() {
            self.initialize().await?;
        }

        show_info("📋 获取备份文件列表...");

        if let Some(ref client) = self.client {
            if let Some(ref config) = self.config {
                let url = format!("{}{}", config.url.trim_end_matches('/'), "/ca-switch-backups");

                let method = reqwest::Method::from_bytes(b"PROPFIND")
                    .map_err(|e| CliError::Config(format!("创建 PROPFIND 方法失败: {e}")))?;

                let response = client
                    .request(method, &url)
                    .header("Depth", "1")
                    .send()
                    .await
                    .map_err(|e| CliError::WebDav(format!("获取备份列表失败: {e}")))?;

                if !response.status().is_success() && response.status().as_u16() != 207 {
                    return Err(CliError::WebDav(format!(
                        "获取备份列表失败: HTTP {}",
                        response.status()
                    )));
                }

                let body = response.text().await?;

                // 解析 WebDAV XML 响应
                let backups = self.parse_webdav_response(&body)?;

                show_success(&format!("✅ 找到 {} 个备份文件", backups.len()));

                Ok(backups)
            } else {
                Err(CliError::Config("WebDAV 未配置".to_string()))
            }
        } else {
            Err(CliError::Config("WebDAV 客户端未初始化".to_string()))
        }
    }

    /// 解析 WebDAV XML 响应
    fn parse_webdav_response(&self, xml: &str) -> Result<Vec<WebDAVFile>> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut backups = Vec::new();
        let mut current_path = String::new();
        let mut current_size: u64 = 0;
        let mut current_modified = String::new();
        let mut in_response = false;
        let mut in_href = false;
        let mut in_getcontentlength = false;
        let mut in_getlastmodified = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"D:response" | b"d:response" => in_response = true,
                        b"D:href" | b"d:href" if in_response => in_href = true,
                        b"D:getcontentlength" | b"d:getcontentlength" if in_response => {
                            in_getcontentlength = true
                        }
                        b"D:getlastmodified" | b"d:getlastmodified" if in_response => {
                            in_getlastmodified = true
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();

                    if in_href {
                        current_path = text.trim().to_string();
                    } else if in_getcontentlength {
                        current_size = text.trim().parse().unwrap_or(0);
                    } else if in_getlastmodified {
                        current_modified = text.trim().to_string();
                    }
                }
                Ok(Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"D:href" | b"d:href" => in_href = false,
                        b"D:getcontentlength" | b"d:getcontentlength" => {
                            in_getcontentlength = false
                        }
                        b"D:getlastmodified" | b"d:getlastmodified" => in_getlastmodified = false,
                        b"D:response" | b"d:response" => {
                            if in_response && !current_path.is_empty() {
                                // 提取文件名
                                if let Some(name) = current_path.split('/').last() {
                                    // 过滤掉目录本身，只保留 .json 文件
                                    if name.ends_with(".json") {
                                        let (category, timestamp) =
                                            WebDAVFile::parse_filename(name);

                                        // 解析修改时间
                                        let last_modified = chrono::DateTime::parse_from_rfc2822(
                                            &current_modified,
                                        )
                                        .or_else(|_| {
                                            // 尝试其他格式
                                            chrono::DateTime::parse_from_rfc3339(&current_modified)
                                        })
                                        .unwrap_or_else(|_| {
                                            chrono::DateTime::from(chrono::Utc::now())
                                        })
                                        .with_timezone(&chrono::Utc);

                                        backups.push(WebDAVFile {
                                            name: name.to_string(),
                                            path: current_path.clone(),
                                            size: current_size,
                                            last_modified,
                                            category,
                                            timestamp,
                                        });
                                    }
                                }
                            }

                            // 重置状态
                            in_response = false;
                            current_path.clear();
                            current_size = 0;
                            current_modified.clear();
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(CliError::WebDav(format!("XML 解析错误: {e}")));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(backups)
    }

    /// 按分类筛选备份文件
    #[allow(dead_code)]
    pub fn filter_by_category(backups: Vec<WebDAVFile>, category: &str) -> Vec<WebDAVFile> {
        backups
            .into_iter()
            .filter(|f| f.category == category)
            .collect()
    }

    /// 按时间范围筛选备份文件
    #[allow(dead_code)]
    pub fn filter_by_date_range(
        backups: Vec<WebDAVFile>,
        start: Option<chrono::DateTime<chrono::Local>>,
        end: Option<chrono::DateTime<chrono::Local>>,
    ) -> Vec<WebDAVFile> {
        backups
            .into_iter()
            .filter(|f| {
                if let Some(ts) = f.timestamp {
                    let after_start = start.map_or(true, |s| ts >= s);
                    let before_end = end.map_or(true, |e| ts <= e);
                    after_start && before_end
                } else {
                    false
                }
            })
            .collect()
    }

    /// 按名称模糊搜索备份文件
    #[allow(dead_code)]
    pub fn search_by_name(backups: Vec<WebDAVFile>, keyword: &str) -> Vec<WebDAVFile> {
        backups
            .into_iter()
            .filter(|f| f.name.contains(keyword))
            .collect()
    }

    /// 按修改时间排序（从新到旧）
    #[allow(dead_code)]
    pub fn sort_by_time_desc(mut backups: Vec<WebDAVFile>) -> Vec<WebDAVFile> {
        backups.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        backups
    }

    /// 按修改时间排序（从旧到新）
    #[allow(dead_code)]
    pub fn sort_by_time_asc(mut backups: Vec<WebDAVFile>) -> Vec<WebDAVFile> {
        backups.sort_by(|a, b| a.last_modified.cmp(&b.last_modified));
        backups
    }

    /// 按大小排序（从大到小）
    #[allow(dead_code)]
    pub fn sort_by_size_desc(mut backups: Vec<WebDAVFile>) -> Vec<WebDAVFile> {
        backups.sort_by(|a, b| b.size.cmp(&a.size));
        backups
    }

    /// 按分类和时间排序
    #[allow(dead_code)]
    pub fn sort_by_category_and_time(mut backups: Vec<WebDAVFile>) -> Vec<WebDAVFile> {
        backups.sort_by(|a, b| {
            match a.category.cmp(&b.category) {
                std::cmp::Ordering::Equal => b.last_modified.cmp(&a.last_modified),
                other => other,
            }
        });
        backups
    }

    /// 获取最新的 N 个备份
    #[allow(dead_code)]
    pub fn get_latest_n(backups: Vec<WebDAVFile>, n: usize) -> Vec<WebDAVFile> {
        let mut sorted = Self::sort_by_time_desc(backups);
        sorted.truncate(n);
        sorted
    }

    /// 获取每个分类的最新备份
    #[allow(dead_code)]
    pub fn get_latest_per_category(backups: Vec<WebDAVFile>) -> Vec<WebDAVFile> {
        use std::collections::HashMap;

        let mut category_map: HashMap<String, WebDAVFile> = HashMap::new();

        for file in backups {
            category_map
                .entry(file.category.clone())
                .and_modify(|existing| {
                    if file.last_modified > existing.last_modified {
                        *existing = file.clone();
                    }
                })
                .or_insert(file);
        }

        category_map.into_values().collect()
    }

    /// 下载备份文件
    #[allow(dead_code)]
    pub async fn download_backup(&mut self, remote_path: &str) -> Result<serde_json::Value> {
        if self.client.is_none() {
            self.initialize().await?;
        }

        show_info(&format!("📥 下载备份文件: {remote_path}"));

        if let Some(ref client) = self.client {
            if let Some(ref config) = self.config {
                let url = format!("{}{}", config.url.trim_end_matches('/'), remote_path);

                let response = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| CliError::WebDav(format!("下载失败: {e}")))?;

                if !response.status().is_success() {
                    return Err(CliError::WebDav(format!(
                        "下载失败: HTTP {}",
                        response.status()
                    )));
                }

                let content = response.text().await?;
                let data: serde_json::Value = serde_json::from_str(&content)?;

                show_success("✅ 备份文件下载成功");

                Ok(data)
            } else {
                Err(CliError::Config("WebDAV 未配置".to_string()))
            }
        } else {
            Err(CliError::Config("WebDAV 客户端未初始化".to_string()))
        }
    }

    /// 删除备份文件
    #[allow(dead_code)]
    pub async fn delete_backup(&mut self, remote_path: &str) -> Result<()> {
        if self.client.is_none() {
            self.initialize().await?;
        }

        show_info(&format!("🗑️ 删除备份文件: {remote_path}"));

        if let Some(ref client) = self.client {
            if let Some(ref config) = self.config {
                let url = format!("{}{}", config.url.trim_end_matches('/'), remote_path);

                let response = client
                    .delete(&url)
                    .send()
                    .await
                    .map_err(|e| CliError::WebDav(format!("删除失败: {e}")))?;

                if response.status().is_success() || response.status().as_u16() == 204 {
                    show_success("✅ 备份文件删除成功");
                    Ok(())
                } else {
                    Err(CliError::WebDav(format!(
                        "删除失败: HTTP {}",
                        response.status()
                    )))
                }
            } else {
                Err(CliError::Config("WebDAV 未配置".to_string()))
            }
        } else {
            Err(CliError::Config("WebDAV 客户端未初始化".to_string()))
        }
    }

    /// 批量删除备份文件
    #[allow(dead_code)]
    pub async fn delete_backups_batch(&mut self, files: Vec<WebDAVFile>) -> Result<()> {
        if files.is_empty() {
            show_warning("⚠️ 没有要删除的文件");
            return Ok(());
        }

        show_info(&format!("🗑️ 准备删除 {} 个备份文件...", files.len()));

        let mut success_count = 0;
        let mut fail_count = 0;
        let total = files.len();

        for (index, file) in files.iter().enumerate() {
            println!();
            show_info(&format!("[{}/{}] 删除: {}", index + 1, total, file.name));

            match self.delete_backup(&file.path).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    show_error(&format!("❌ 删除失败: {e}"));
                    fail_count += 1;
                }
            }
        }

        println!();
        show_success(&format!(
            "✅ 批量删除完成: 成功 {}, 失败 {}, 总计 {}",
            success_count, fail_count, total
        ));

        Ok(())
    }

    /// 批量下载备份文件
    #[allow(dead_code)]
    pub async fn download_backups_batch(
        &mut self,
        files: Vec<WebDAVFile>,
    ) -> Result<Vec<(String, serde_json::Value)>> {
        if files.is_empty() {
            show_warning("⚠️ 没有要下载的文件");
            return Ok(Vec::new());
        }

        show_info(&format!("📥 准备下载 {} 个备份文件...", files.len()));

        let mut results = Vec::new();
        let mut success_count = 0;
        let mut fail_count = 0;
        let total = files.len();

        for (index, file) in files.iter().enumerate() {
            println!();
            show_info(&format!("[{}/{}] 下载: {}", index + 1, total, file.name));

            match self.download_backup(&file.path).await {
                Ok(data) => {
                    results.push((file.name.clone(), data));
                    success_count += 1;
                }
                Err(e) => {
                    show_error(&format!("❌ 下载失败: {e}"));
                    fail_count += 1;
                }
            }
        }

        println!();
        show_success(&format!(
            "✅ 批量下载完成: 成功 {}, 失败 {}, 总计 {}",
            success_count, fail_count, total
        ));

        Ok(results)
    }

    /// 清理旧备份（保留每个分类最新的 N 个）
    #[allow(dead_code)]
    pub async fn cleanup_old_backups(&mut self, keep_per_category: usize) -> Result<()> {
        show_info(&format!(
            "🧹 开始清理旧备份，每个分类保留最新 {} 个...",
            keep_per_category
        ));

        let all_backups = self.list_backups().await?;

        if all_backups.is_empty() {
            show_info("📭 没有发现备份文件");
            return Ok(());
        }

        // 按分类分组
        use std::collections::HashMap;
        let mut category_map: HashMap<String, Vec<WebDAVFile>> = HashMap::new();

        for file in all_backups {
            category_map
                .entry(file.category.clone())
                .or_insert_with(Vec::new)
                .push(file);
        }

        let mut to_delete = Vec::new();

        // 对每个分类，按时间排序并标记要删除的文件
        for (category, mut files) in category_map {
            files.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

            if files.len() > keep_per_category {
                let old_files = files.split_off(keep_per_category);
                show_info(&format!(
                    "📦 分类 [{}]: 找到 {} 个旧备份",
                    category,
                    old_files.len()
                ));
                to_delete.extend(old_files);
            }
        }

        if to_delete.is_empty() {
            show_success("✅ 没有需要清理的旧备份");
            return Ok(());
        }

        show_warning(&format!("⚠️ 将删除 {} 个旧备份文件", to_delete.len()));

        if confirm("确认删除这些旧备份吗？", false)? {
            self.delete_backups_batch(to_delete).await?;
        } else {
            show_info("❌ 已取消清理操作");
        }

        Ok(())
    }

    /// 获取 WebDAV 服务信息
    pub fn get_server_info(&self) -> Option<(String, String, String)> {
        self.config.as_ref().map(|config| {
            let server_type = self.detect_server_type(&config.url);
            (config.url.clone(), config.username.clone(), server_type)
        })
    }

    /// 检测服务器类型
    fn detect_server_type(&self, url: &str) -> String {
        if url.contains("jianguoyun.com") {
            "坚果云".to_string()
        } else if url.contains("nextcloud") {
            "Nextcloud".to_string()
        } else if url.contains("owncloud") {
            "ownCloud".to_string()
        } else {
            "通用WebDAV".to_string()
        }
    }

    /// 检查连接健康状态
    #[allow(dead_code)]
    pub async fn health_check(&self) -> Result<HealthStatus> {
        if let Some(ref client) = self.client {
            if let Some(ref config) = self.config {
                // 测试基本连接
                let method = reqwest::Method::from_bytes(b"PROPFIND")
                    .map_err(|e| CliError::Config(format!("创建 PROPFIND 方法失败: {e}")))?;

                let start_time = std::time::Instant::now();
                let response = client
                    .request(method, &config.url)
                    .header("Depth", "0")
                    .send()
                    .await;

                let latency = start_time.elapsed().as_millis() as u64;

                match response {
                    Ok(resp) => {
                        let status_code = resp.status().as_u16();
                        if resp.status().is_success() || status_code == 207 {
                            Ok(HealthStatus {
                                connected: true,
                                latency_ms: Some(latency),
                                server_type: self.detect_server_type(&config.url),
                                error_message: None,
                            })
                        } else {
                            Ok(HealthStatus {
                                connected: false,
                                latency_ms: Some(latency),
                                server_type: self.detect_server_type(&config.url),
                                error_message: Some(format!("HTTP 状态码: {status_code}")),
                            })
                        }
                    }
                    Err(e) => Ok(HealthStatus {
                        connected: false,
                        latency_ms: None,
                        server_type: self.detect_server_type(&config.url),
                        error_message: Some(format!("连接失败: {e}")),
                    }),
                }
            } else {
                Err(CliError::Config("WebDAV 未配置".to_string()))
            }
        } else {
            Err(CliError::Config("WebDAV 客户端未初始化".to_string()))
        }
    }

    /// 获取存储使用情况（如果服务器支持）
    #[allow(dead_code)]
    pub async fn get_storage_info(&self) -> Result<StorageInfo> {
        if let Some(ref client) = self.client {
            if let Some(ref config) = self.config {
                let url = format!("{}{}", config.url.trim_end_matches('/'), "/ca-switch-backups");

                let method = reqwest::Method::from_bytes(b"PROPFIND")
                    .map_err(|e| CliError::Config(format!("创建 PROPFIND 方法失败: {e}")))?;

                let response = client
                    .request(method, &url)
                    .header("Depth", "1")
                    .send()
                    .await
                    .map_err(|e| CliError::WebDav(format!("获取存储信息失败: {e}")))?;

                if !response.status().is_success() && response.status().as_u16() != 207 {
                    return Ok(StorageInfo {
                        total_files: 0,
                        total_size_bytes: 0,
                        categories: std::collections::HashMap::new(),
                    });
                }

                let body = response.text().await?;
                let backups = self.parse_webdav_response(&body)?;

                let total_files = backups.len();
                let total_size_bytes: u64 = backups.iter().map(|f| f.size).sum();

                // 按分类统计
                use std::collections::HashMap;
                let mut categories: HashMap<String, usize> = HashMap::new();

                for file in backups {
                    *categories.entry(file.category).or_insert(0) += 1;
                }

                Ok(StorageInfo {
                    total_files,
                    total_size_bytes,
                    categories,
                })
            } else {
                Err(CliError::Config("WebDAV 未配置".to_string()))
            }
        } else {
            Err(CliError::Config("WebDAV 客户端未初始化".to_string()))
        }
    }

    /// 格式化文件大小
    #[allow(dead_code)]
    pub fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// 清除保存的配置
    pub async fn clear_config(&mut self) -> Result<()> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path).await?;
            show_success("✅ 已清除 WebDAV 配置");
        }

        self.client = None;
        self.config = None;

        Ok(())
    }
}

impl Default for WebDAVClient {
    fn default() -> Self {
        Self::new().expect("Failed to create WebDAVClient")
    }
}
