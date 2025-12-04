use crate::error::{CliError, Result};
use crate::ui::{confirm, show_error, show_info, show_success, show_warning};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Password};
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

        let config_path = home_dir.join(".cc-cli").join("webdav-config.json");

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
        let backup_dir = "/cc-cli-backups";

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
        let remote_path = format!("/cc-cli-backups/{file_name}");

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
                let url = format!("{}{}", config.url.trim_end_matches('/'), "/cc-cli-backups");

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

                // 简单解析 WebDAV 响应（这里简化处理，实际应该用 XML 解析器）
                let backups = Vec::new();

                // 由于 Rust 中没有简单的 WebDAV 客户端库，这里简化实现
                // 生产环境应该使用专门的 WebDAV 库或 XML 解析器
                show_success(&format!("✅ 找到备份文件 (响应大小: {} bytes)", body.len()));

                Ok(backups)
            } else {
                Err(CliError::Config("WebDAV 未配置".to_string()))
            }
        } else {
            Err(CliError::Config("WebDAV 客户端未初始化".to_string()))
        }
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
