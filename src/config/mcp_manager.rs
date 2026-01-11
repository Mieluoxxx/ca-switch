// MCP 配置管理器
// 负责管理 ~/.opcd/mcp/ 目录下的多个 JSON 文件，并同步到 opencode.json

use crate::config::models::{McpConfig, McpOAuthConfig, McpServer, McpServerType};
use crate::config::ConfigError;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// MCP 配置管理器
/// 采用目录模式：~/.opcd/mcp/ 下每个服务器一个 JSON 文件
pub struct McpConfigManager {
    mcp_dir: PathBuf,       // ~/.opcd/mcp/
    opencode_dir: PathBuf,  // ~/.opencode
    opencode_json: PathBuf, // ~/.opencode/opencode.json
}

#[allow(dead_code)]
impl McpConfigManager {
    /// 创建新的 MCP 配置管理器
    pub fn new(config_dir: PathBuf) -> Result<Self, ConfigError> {
        let mcp_dir = config_dir.join("mcp");

        // 确保 mcp 目录存在
        if !mcp_dir.exists() {
            fs::create_dir_all(&mcp_dir)?;
        }

        let opencode_dir = dirs::home_dir()
            .ok_or_else(|| ConfigError::NotFound {
                name: "用户主目录".to_string(),
            })?
            .join(".opencode");

        let opencode_json = opencode_dir.join("opencode.json");

        Ok(Self {
            mcp_dir,
            opencode_dir,
            opencode_json,
        })
    }

    // ========================================================================
    // 目录模式配置读写
    // ========================================================================

    /// 获取服务器配置文件路径
    fn get_server_file(&self, name: &str) -> PathBuf {
        self.mcp_dir.join(format!("{}.json", name))
    }

    /// 从目录加载所有服务器配置，装配成 McpConfig
    /// 加载过程中的错误会被静默忽略（使用 read_config_with_warnings 获取警告）
    pub fn read_config(&self) -> Result<McpConfig, String> {
        let (config, _warnings) = self.read_config_with_warnings()?;
        Ok(config)
    }

    /// 从目录加载所有服务器配置，同时返回加载过程中的警告信息
    /// 返回 (配置, 警告列表)
    pub fn read_config_with_warnings(&self) -> Result<(McpConfig, Vec<String>), String> {
        let mut config = McpConfig::new();
        let mut warnings = Vec::new();

        if !self.mcp_dir.exists() {
            return Ok((config, warnings));
        }

        // 遍历 mcp 目录下的所有 .json 文件
        let entries =
            fs::read_dir(&self.mcp_dir).map_err(|e| format!("读取 mcp 目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
            let path = entry.path();

            // 只处理 .json 文件
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.read_server_file(&path) {
                        Ok(server) => {
                            config.add_server(name.to_string(), server);
                        }
                        Err(e) => {
                            // 收集警告，继续处理其他文件
                            warnings.push(format!("加载 MCP 配置 '{}' 失败: {}", name, e));
                        }
                    }
                }
            }
        }

        Ok((config, warnings))
    }

    /// 读取单个服务器配置文件
    /// 支持两种格式：
    /// 1. 用户原始格式：{ "command": "uvx", "args": [...] }
    /// 2. McpServer 结构体格式（向后兼容）
    fn read_server_file(&self, path: &PathBuf) -> Result<McpServer, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;

        // 首先尝试解析为 McpServer 结构体格式
        if let Ok(server) = serde_json::from_str::<McpServer>(&content) {
            return Ok(server);
        }

        // 尝试解析用户原始 JSON 格式，使用统一的 McpServer::from_json
        let json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

        McpServer::from_json(&json)
    }

    /// 写入单个服务器配置文件
    fn write_server_file(&self, name: &str, server: &McpServer) -> Result<(), String> {
        let path = self.get_server_file(name);
        let content =
            serde_json::to_string_pretty(server).map_err(|e| format!("序列化失败: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))
    }

    /// 删除服务器配置文件
    fn delete_server_file(&self, name: &str) -> Result<(), String> {
        let path = self.get_server_file(name);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
        }
        Ok(())
    }

    // ========================================================================
    // 服务器管理
    // ========================================================================

    /// 获取服务器
    pub fn get_server(&self, name: &str) -> Result<Option<McpServer>, String> {
        let path = self.get_server_file(name);
        if !path.exists() {
            return Ok(None);
        }
        self.read_server_file(&path).map(Some)
    }

    /// 获取原始 JSON 内容
    pub fn get_raw_json(&self, name: &str) -> Result<Option<String>, String> {
        let path = self.get_server_file(name);
        if !path.exists() {
            return Ok(None);
        }
        fs::read_to_string(&path)
            .map(Some)
            .map_err(|e| format!("读取文件失败: {}", e))
    }

    /// 获取排序后的服务器名称列表
    pub fn get_sorted_server_names(&self) -> Result<Vec<String>, String> {
        let config = self.read_config()?;
        Ok(config.get_sorted_server_names())
    }

    /// 添加本地服务器
    pub fn add_local_server(
        &mut self,
        name: String,
        command: Vec<String>,
        environment: HashMap<String, String>,
        timeout: Option<u32>,
    ) -> Result<(), String> {
        let path = self.get_server_file(&name);
        if path.exists() {
            return Err(format!("MCP 服务器 '{}' 已存在", name));
        }

        let mut server = McpServer::new_local(command, environment);
        server.timeout = timeout;

        self.write_server_file(&name, &server)
    }

    /// 添加远程服务器
    pub fn add_remote_server(
        &mut self,
        name: String,
        url: String,
        headers: HashMap<String, String>,
        oauth: Option<McpOAuthConfig>,
        timeout: Option<u32>,
    ) -> Result<(), String> {
        let path = self.get_server_file(&name);
        if path.exists() {
            return Err(format!("MCP 服务器 '{}' 已存在", name));
        }

        let mut server = McpServer::new_remote(url, headers, oauth);
        server.timeout = timeout;

        self.write_server_file(&name, &server)
    }

    /// 保存服务器（添加或更新）
    pub fn save_server(&mut self, name: &str, mut server: McpServer) -> Result<(), String> {
        server.update_timestamp();
        self.write_server_file(name, &server)
    }

    /// 直接保存原始 JSON 配置（自动格式化）
    pub fn save_raw_json(&mut self, name: &str, json_content: &str) -> Result<(), String> {
        let path = self.get_server_file(name);

        // 解析并重新格式化 JSON
        let formatted = match serde_json::from_str::<serde_json::Value>(json_content) {
            Ok(value) => serde_json::to_string_pretty(&value)
                .map_err(|e| format!("格式化 JSON 失败: {}", e))?,
            Err(_) => {
                // 如果无法解析，直接保存原内容
                json_content.to_string()
            }
        };

        fs::write(&path, formatted).map_err(|e| format!("写入文件失败: {}", e))
    }

    /// 删除服务器
    pub fn delete_server(&mut self, name: &str) -> Result<(), String> {
        let path = self.get_server_file(name);
        if !path.exists() {
            return Err(format!("MCP 服务器 '{}' 不存在", name));
        }
        self.delete_server_file(name)
    }

    /// 重命名服务器
    pub fn rename_server(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        let old_path = self.get_server_file(old_name);
        let new_path = self.get_server_file(new_name);

        if !old_path.exists() {
            return Err(format!("MCP 服务器 '{}' 不存在", old_name));
        }

        if new_path.exists() {
            return Err(format!("MCP 服务器 '{}' 已存在", new_name));
        }

        fs::rename(&old_path, &new_path).map_err(|e| format!("重命名失败: {}", e))
    }

    /// 切换服务器启用状态
    pub fn toggle_server_enabled(&mut self, name: &str) -> Result<bool, String> {
        let mut server = self
            .get_server(name)?
            .ok_or_else(|| format!("MCP 服务器 '{}' 不存在", name))?;

        server.enabled = !server.enabled;
        server.update_timestamp();
        let new_state = server.enabled;

        self.write_server_file(name, &server)?;
        Ok(new_state)
    }

    // ========================================================================
    // 同步到 opencode.json
    // ========================================================================

    /// 同步 MCP 配置到 OpenCode 配置文件（全局）
    pub fn sync_to_opencode(&self, server_names: Option<&[String]>) -> Result<(), String> {
        if !self.opencode_dir.exists() {
            fs::create_dir_all(&self.opencode_dir)
                .map_err(|e| format!("创建 .opencode 目录失败: {}", e))?;
        }

        let mcp_config = self.read_config()?;

        // 读取现有的 opencode.json（如果存在）
        let mut opencode_data: serde_json::Value = if self.opencode_json.exists() {
            let content = fs::read_to_string(&self.opencode_json)
                .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;
            serde_json::from_str(&content).map_err(|e| format!("解析 opencode.json 失败: {}", e))?
        } else {
            serde_json::json!({
                "$schema": "https://opencode.ai/config.json",
                "theme": "tokyonight",
                "autoupdate": false,
                "provider": {},
                "tools": {},
                "agent": {},
                "mcp": {}
            })
        };

        // 构建 MCP 配置（OpenCode 格式）
        let mcp_map = self.build_opencode_mcp_map(&mcp_config, server_names)?;

        // 更新 mcp 字段
        opencode_data["mcp"] = serde_json::Value::Object(mcp_map);

        // 写回 opencode.json
        let content = serde_json::to_string_pretty(&opencode_data)
            .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;

        fs::write(&self.opencode_json, content)
            .map_err(|e| format!("写入 opencode.json 失败: {}", e))
    }

    /// 同步 MCP 配置到项目级 opencode.json
    pub fn sync_to_project(&self, server_names: Option<&[String]>) -> Result<(), String> {
        let current_dir =
            std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?;

        let project_opencode_dir = current_dir.join(".opencode");
        let project_opencode_json = project_opencode_dir.join("opencode.json");

        if !project_opencode_dir.exists() {
            fs::create_dir_all(&project_opencode_dir)
                .map_err(|e| format!("创建项目 .opencode 目录失败: {}", e))?;
        }

        let mcp_config = self.read_config()?;

        // 读取现有的项目 opencode.json
        let mut opencode_data: serde_json::Value = if project_opencode_json.exists() {
            let content = fs::read_to_string(&project_opencode_json)
                .map_err(|e| format!("读取项目 opencode.json 失败: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("解析项目 opencode.json 失败: {}", e))?
        } else {
            serde_json::json!({
                "$schema": "https://opencode.ai/config.json",
                "mcp": {}
            })
        };

        // 构建 MCP 配置
        let mcp_map = self.build_opencode_mcp_map(&mcp_config, server_names)?;

        opencode_data["mcp"] = serde_json::Value::Object(mcp_map);

        let content = serde_json::to_string_pretty(&opencode_data)
            .map_err(|e| format!("序列化项目 opencode.json 失败: {}", e))?;

        fs::write(&project_opencode_json, content)
            .map_err(|e| format!("写入项目 opencode.json 失败: {}", e))
    }

    /// 构建 OpenCode 格式的 MCP 配置
    fn build_opencode_mcp_map(
        &self,
        mcp_config: &McpConfig,
        server_names: Option<&[String]>,
    ) -> Result<serde_json::Map<String, serde_json::Value>, String> {
        let mut mcp_map = serde_json::Map::new();

        let servers_to_sync: Vec<(String, McpServer)> = match server_names {
            Some(names) => names
                .iter()
                .filter_map(|n| mcp_config.get_server(n).map(|s| (n.clone(), s.clone())))
                .collect(),
            None => mcp_config
                .servers
                .iter()
                .filter(|(_, s)| s.enabled)
                .map(|(n, s)| (n.clone(), s.clone()))
                .collect(),
        };

        for (name, server) in servers_to_sync {
            let server_value = self.server_to_opencode_format(&server)?;
            mcp_map.insert(name, server_value);
        }

        Ok(mcp_map)
    }

    /// 将 McpServer 转换为 OpenCode 格式
    fn server_to_opencode_format(&self, server: &McpServer) -> Result<serde_json::Value, String> {
        let mut obj = serde_json::Map::new();

        obj.insert(
            "type".to_string(),
            serde_json::json!(server.server_type.to_string()),
        );
        obj.insert("enabled".to_string(), serde_json::json!(server.enabled));

        if let Some(timeout) = server.timeout {
            obj.insert("timeout".to_string(), serde_json::json!(timeout));
        }

        match server.server_type {
            McpServerType::Local => {
                if let Some(ref cmd) = server.command {
                    obj.insert("command".to_string(), serde_json::json!(cmd));
                }
                if !server.environment.is_empty() {
                    obj.insert(
                        "environment".to_string(),
                        serde_json::json!(server.environment),
                    );
                }
            }
            McpServerType::Remote => {
                if let Some(ref url) = server.url {
                    obj.insert("url".to_string(), serde_json::json!(url));
                }
                if !server.headers.is_empty() {
                    obj.insert("headers".to_string(), serde_json::json!(server.headers));
                }
                if let Some(ref oauth) = server.oauth {
                    if !oauth.is_empty() {
                        let mut oauth_obj = serde_json::Map::new();
                        if let Some(ref client_id) = oauth.client_id {
                            oauth_obj.insert("clientId".to_string(), serde_json::json!(client_id));
                        }
                        if let Some(ref client_secret) = oauth.client_secret {
                            oauth_obj.insert(
                                "clientSecret".to_string(),
                                serde_json::json!(client_secret),
                            );
                        }
                        if let Some(ref scope) = oauth.scope {
                            oauth_obj.insert("scope".to_string(), serde_json::json!(scope));
                        }
                        obj.insert("oauth".to_string(), serde_json::Value::Object(oauth_obj));
                    }
                }
            }
        }

        Ok(serde_json::Value::Object(obj))
    }

    /// 获取 MCP 目录路径
    pub fn get_mcp_dir(&self) -> &PathBuf {
        &self.mcp_dir
    }
}
