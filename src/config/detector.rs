// OpenCode 站点和模型检测器
// 用于检测站点可用性、获取模型列表、测试模型性能

use crate::config::models::SiteDetectionResult;
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};

/// 智能拼接URL路径，避免重复 /v1
fn build_api_url(base_url: &str, path: &str) -> String {
    let base = base_url.trim_end_matches('/');

    // 如果 base_url 已经以 /v1 结尾，直接拼接路径
    if base.ends_with("/v1") {
        format!("{}{}", base, path)
    } else {
        // 否则添加 /v1 前缀
        format!("{}/v1{}", base, path)
    }
}

/// 站点和模型检测器
pub struct Detector {
    client: Client,
}

impl Detector {
    /// 创建新的检测器
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    // ========== 站点检测 ==========

    /// 完整的站点检测
    pub async fn detect_site(&self, base_url: &str, api_key: &str) -> SiteDetectionResult {
        let start = Instant::now();
        let mut result = SiteDetectionResult {
            detected_at: chrono::Utc::now().to_rfc3339(),
            is_available: false,
            api_key_valid: false,
            available_models: vec![],
            response_time_ms: None,
            error_message: None,
        };

        // 尝试获取模型列表
        match self.fetch_models_list(base_url, api_key).await {
            Ok(models) => {
                result.is_available = true;
                result.api_key_valid = true;
                result.available_models = models;
                result.response_time_ms = Some(start.elapsed().as_millis() as f64);
            }
            Err(e) => {
                result.is_available = false;
                result.error_message = Some(e);
            }
        }

        result
    }

    /// 获取模型列表 (调用 /v1/models API)
    async fn fetch_models_list(
        &self,
        base_url: &str,
        api_key: &str,
    ) -> Result<Vec<String>, String> {
        let url = build_api_url(base_url, "/models");

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}: API返回错误", response.status()));
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            id: String,
        }

        let models_resp: ModelsResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        Ok(models_resp.data.into_iter().map(|m| m.id).collect())
    }
}
