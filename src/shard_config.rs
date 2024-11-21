use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct RpcResponse {
    jsonrpc: String,
    result: ShardConfig,
    id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardConfig {
    pub shard_id: u32,
    pub num_shard: u32,
}

pub async fn get_config(url: &str) -> ShardConfig {
    // 创建 HTTP 客户端
    let client = Client::new();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "zgs_getShardConfig",
        "params": [],
        "id": 1
    });

    // 发送请求并获取响应
    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .unwrap()
        .json::<RpcResponse>()
        .await
        .unwrap();

    response.result
}
