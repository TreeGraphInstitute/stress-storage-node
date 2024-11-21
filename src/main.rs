mod cli;
mod shard_config;

use ethereum_types::H256;
use reqwest::Client;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::time;

use tokio::sync::mpsc::{self, Receiver};
use tokio::sync::RwLock;

// const FILE_ROOT: H256 = H256(hex!(
//     "e67465864bb2e8e0cc8586992d9390089d621ede49e76e3fcc77382916ba7111"
// ));

async fn send_request_worker(url: String, rx: Arc<RwLock<Receiver<(H256, usize, usize)>>>) {
    let client = Client::new();

    loop {
        let mut rx = rx.write().await;

        let (hash, start, end) = if let Some(msg) = rx.recv().await {
            msg
        } else {
            break;
        };

        std::mem::drop(rx);

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "zgs_downloadSegment",
            "params": [hex::encode(hash), start, end],
            "id": 1
        });

        let response = client.post(&url).json(&payload).send().await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    let ans = res.text().await;
                    if ans.is_err() {
                        println!("text err {:?}", ans);
                    }
                } else {
                    println!("Error {:?}", &res);
                }
            }
            Err(e) => {
                println!("Error {:?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = cli::Cli::from_args();

    let file_root = H256::from_str(&args.file_root).unwrap();
    let shard_config = shard_config::get_config(&args.url).await;

    let (tx, rx) = mpsc::channel(args.concurrent_workers); // 创建一个通道

    println!("Spawn worker");

    let rx = Arc::new(RwLock::new(rx));

    // 创建多个工作线程
    let mut handles = Vec::new();
    for _ in 0..args.concurrent_workers {
        let rx = rx.clone();
        let url = args.url.clone();
        let handle = tokio::spawn(async move { send_request_worker(url, rx).await });
        handles.push(handle);
    }

    println!("Send task");

    let start_time = time::Instant::now();
    // 发送任务到通道
    for i in 0..args.total_requests {
        let group = i / 512;
        let index = i % 512;
        let task_base = (group * shard_config.num_shard as usize + shard_config.shard_id as usize)
            * 1024
            + index * 2;
        let task = (file_root, task_base, task_base + 2);
        if tx.send(task).await.is_err() {
            println!("Cannot send");
            break; // 如果接收端已关闭，则停止发送
        }
    }

    drop(tx);

    for handle in handles {
        let _ = handle.await;
    }
    let duration = time::Instant::now().duration_since(start_time);
    println!(
        "Total time for {} requests: {:?}s",
        args.total_requests,
        duration.as_secs_f32()
    );
}
