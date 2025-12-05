// OpenCode 站点和模型检测器
// 用于检测站点可用性、获取模型列表、测试模型性能

use crate::config::models::{ModelDetectionResult, SiteDetectionResult};
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
    pub async fn detect_site(
        &self,
        base_url: &str,
        api_key: &str,
    ) -> SiteDetectionResult {
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

    // ========== 模型检测 ==========

    /// 完整的模型检测
    pub async fn detect_model(
        &self,
        base_url: &str,
        api_key: &str,
        model_id: &str,
        test_stream: bool,
    ) -> ModelDetectionResult {
        let mut result = ModelDetectionResult {
            detected_at: chrono::Utc::now().to_rfc3339(),
            model_id: model_id.to_string(),
            is_available: false,
            first_token_time_ms: None,
            tokens_per_second: None,
            total_response_time_ms: None,
            stream_available: None,
            error_message: None,
        };

        // 1. 测试非流式请求
        match self.test_model_completion(base_url, api_key, model_id).await {
            Ok(perf) => {
                result.is_available = true;
                result.first_token_time_ms = Some(perf.first_token_ms);
                result.total_response_time_ms = Some(perf.total_ms);
                result.tokens_per_second = perf.tokens_per_sec;
            }
            Err(e) => {
                result.error_message = Some(e);
                return result; // 非流式失败就不测试流式了
            }
        }

        // 2. 可选: 测试流式请求
        if test_stream {
            match self.test_model_streaming(base_url, api_key, model_id).await {
                Ok(_) => result.stream_available = Some(true),
                Err(_) => result.stream_available = Some(false),
            }
        }

        result
    }

    /// 测试模型的非流式完成请求
    async fn test_model_completion(
        &self,
        base_url: &str,
        api_key: &str,
        model_id: &str,
    ) -> Result<ModelPerformance, String> {
        let url = build_api_url(base_url, "/chat/completions");

        let body = serde_json::json!({
            "model": model_id,
            "messages": [
                {
                    "role": "user",
                    "content": "请用一句话介绍你自己"
                }
            ],
            "max_tokens": 50,
            "stream": false,
        });

        let start = Instant::now();

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        let first_token_ms = start.elapsed().as_millis() as f64;

        if !response.status().is_success() {
            return Err(format!("HTTP {}: 模型返回错误", response.status()));
        }

        // 解析响应
        #[derive(Deserialize, Debug)]
        struct CompletionResponse {
            usage: Option<Usage>,
            #[allow(dead_code)]
            choices: Option<serde_json::Value>,
        }

        #[derive(Deserialize, Debug)]
        struct Usage {
            completion_tokens: Option<u32>,
            #[allow(dead_code)]
            total_tokens: Option<u32>,
            #[allow(dead_code)]
            prompt_tokens: Option<u32>,
        }

        let completion_resp: CompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        let total_ms = start.elapsed().as_millis() as f64;

        // 计算token速度
        let tokens_per_sec = if let Some(usage) = &completion_resp.usage {
            if let Some(tokens) = usage.completion_tokens {
                if tokens > 0 {
                    Some((tokens as f64) / (total_ms / 1000.0))
                } else {
                    // completion_tokens 为 0，可能API没统计或模型没生成内容
                    None
                }
            } else {
                // 没有 completion_tokens 字段
                None
            }
        } else {
            // 没有 usage 字段
            None
        };

        Ok(ModelPerformance {
            first_token_ms,
            total_ms,
            tokens_per_sec,
        })
    }

    /// 测试模型的流式输出
    async fn test_model_streaming(
        &self,
        base_url: &str,
        api_key: &str,
        model_id: &str,
    ) -> Result<(), String> {
        let url = build_api_url(base_url, "/chat/completions");

        let body = serde_json::json!({
            "model": model_id,
            "messages": [
                {
                    "role": "user",
                    "content": "Say hello"
                }
            ],
            "max_tokens": 10,
            "stream": true,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}: 流式请求失败", response.status()));
        }

        // 简单验证: 只要能收到响应就认为流式可用
        Ok(())
    }
}

/// 模型性能数据
#[derive(Debug)]
struct ModelPerformance {
    first_token_ms: f64,
    total_ms: f64,
    tokens_per_sec: Option<f64>,
}
