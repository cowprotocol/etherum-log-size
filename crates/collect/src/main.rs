// TODO:
// - Consider keeping track of already handled blocks to avoid duplicate fetching.

use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Context, Result};
use ethrpc::{
    eth::{BlockNumber, GetLogs},
    http::Client,
    types::{BlockSpec, Empty, Log, LogBlocks, LogFilter, LogFilterValue, U256},
};

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_());
}

async fn main_() {
    let quit = Arc::new(AtomicBool::new(false));
    let quit_ = quit.clone();
    ctrlc::set_handler(move || {
        quit_.store(true, Ordering::SeqCst);
    })
    .unwrap();

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("out")
        .unwrap();
    let node_url = std::env::var("NODE_URL").unwrap();
    let client = Client::new(node_url.parse().unwrap());

    let max_block = current_block(&client)
        .await
        .unwrap()
        .checked_sub(64)
        .unwrap();
    println!("Using maximum block {max_block}.");
    let block_range = 0u64..=max_block;
    let mut i: u64 = 0;
    while !quit.load(Ordering::SeqCst) {
        let block = fastrand::u64(block_range.clone());
        let logs = match logs_in_block(&client, block).await {
            Ok(logs) => logs,
            Err(err) => {
                println!("error for block {block}: {err:?}");
                continue;
            }
        };

        let data: u64 = logs.iter().map(|log| log.data.len() as u64).sum();
        let topics: u64 = logs.iter().map(|log| log.topics.len() as u64).sum();
        let mut entry = [0u8; 8 + 8 + 8 + 8];
        entry[0..8].copy_from_slice(block.to_le_bytes().as_slice());
        entry[8..16].copy_from_slice(logs.len().to_le_bytes().as_slice());
        entry[16..24].copy_from_slice(data.to_le_bytes().as_slice());
        entry[24..32].copy_from_slice(topics.to_le_bytes().as_slice());
        file.write_all(&entry).unwrap();

        i += 1;
        if i % 100 == 0 {
            println!("{} {} {} {} {}", i, block, logs.len(), data, topics);
            file.flush().unwrap();
        }
    }
    file.flush().unwrap();
    file.sync_all().unwrap();
}

async fn logs_in_block(client: &Client, block: u64) -> Result<Vec<Log>> {
    let block = BlockSpec::Number(U256::new(block as u128));
    let log_filter = LogFilter {
        blocks: LogBlocks::Range {
            from: block,
            to: block,
        },
        address: LogFilterValue::Any,
        topics: Default::default(),
    };
    client
        .execute(GetLogs, (log_filter,))
        .await
        .context("execute")
}

async fn current_block(client: &Client) -> Result<u64> {
    let block = client
        .execute(BlockNumber, Empty)
        .await
        .context("execute")?;
    block.try_into().context("block doesn't fit in u64")
}
