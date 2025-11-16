use std::sync::Arc;

use futures::stream::{FuturesUnordered, StreamExt};
use submerge_substrate_client::SubstrateClient;
use tokio::sync::{Semaphore, mpsc};

/// Block hash for concurrent pre-fetching
#[derive(Debug, Clone)]
pub struct BlockHash {
    pub number: u64,
    pub hash_hex: String,
}

/// Fetch block hashes concurrently
pub async fn fetch_hashes_range(
    substrate_client: &Arc<SubstrateClient>,
    start: u64,
    end: u64,
    max_concurrent: usize,
) -> mpsc::Receiver<anyhow::Result<BlockHash>> {
    let (tx, rx) = mpsc::channel(max_concurrent * 2);
    let client = Arc::clone(substrate_client);
    let sem = Arc::new(Semaphore::new(max_concurrent));

    tokio::spawn(async move {
        let mut futures = FuturesUnordered::new();

        for number in start..=end {
            let client = Arc::clone(&client);
            let sem = Arc::clone(&sem);
            // Clone sender for each task - cheap Arc increment, allows concurrent sends
            let tx = tx.clone();

            futures.push(async move {
                let _permit = sem.acquire().await
                    .map_err(|e| anyhow::anyhow!("Semaphore error for block {}: {}", number, e))?;

                match client.get_block_hash(number).await {
                    Ok(Some(hash_hex)) => {
                        tx.send(Ok(BlockHash { number, hash_hex })).await
                            .map_err(|_| anyhow::anyhow!("Channel closed"))?;
                    }
                    Ok(None) => {
                        let err = anyhow::anyhow!("Block #{} not found (returned None)", number);
                        tx.send(Err(err)).await
                            .map_err(|_| anyhow::anyhow!("Channel closed"))?;
                    }
                    Err(e) => {
                        let err = anyhow::anyhow!("Failed to fetch block #{}: {}", number, e);
                        tx.send(Err(err)).await
                            .map_err(|_| anyhow::anyhow!("Channel closed"))?;
                    }
                }

                anyhow::Ok(())
            });
        }

        while let Some(result) = futures.next().await {
            if let Err(e) = result {
                tracing::error!("Concurrent fetch task error: {}", e);
            }
        }
    });

    rx
}

