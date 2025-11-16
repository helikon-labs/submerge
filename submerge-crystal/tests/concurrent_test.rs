use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
#[ignore] // Run with: cargo test --test concurrent_test -- --ignored --nocapture
async fn test_concurrent_vs_sequential() -> anyhow::Result<()> {
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "wss://polkadot.dotters.network".to_string());

    println!("\nConnecting to {}...", rpc_url);

    let config = submerge_substrate_client::RPCConfig {
        rpc_url,
        rpc_connection_timeout_secs: 30,
        rpc_request_timeout_secs: 30,
        rpc_subscription_timeout_secs: 60,
    };

    let client = Arc::new(submerge_substrate_client::SubstrateClient::new(&config).await?);
    println!("Connected successfully");

    // Get current finalized block
    let finalized_hash = client.get_finalized_block_hash().await?;
    let finalized_header = client.get_block_header(&finalized_hash).await?;
    let finalized_number = finalized_header.get_number()?;
    println!("Current finalized block: #{}", finalized_number);

    // Test with last 100 blocks
    let block_count = 100;
    let start = finalized_number.saturating_sub(block_count - 1);
    let end = finalized_number;

    println!("\nTesting {} blocks (#{} to #{})", block_count, start, end);
    println!("============================================");

    // Sequential baseline
    println!("\n1. Sequential fetch (baseline):");
    let seq_start = Instant::now();
    let mut seq_count = 0;

    for num in start..=end {
        if client.get_block_hash(num).await?.is_some() {
            seq_count += 1;
        }
    }

    let seq_time = seq_start.elapsed();
    println!("   Fetched {} blocks", seq_count);
    println!("   Time: {:?}", seq_time);
    println!("   Rate: {:.2} blocks/sec", seq_count as f64 / seq_time.as_secs_f64());

    // Concurrent fetch with 100 parallel requests
    println!("\n2. Concurrent fetch (100 parallel):");
    let conc_start = Instant::now();

    let mut rx = submerge_crystal::worker::processor::concurrent::fetch_hashes_range(
        &client,
        start,
        end,
        100,
    )
    .await;

    let mut conc_count = 0;
    while let Some(result) = rx.recv().await {
        result?;
        conc_count += 1;
    }

    let conc_time = conc_start.elapsed();
    println!("   Fetched {} blocks", conc_count);
    println!("   Time: {:?}", conc_time);
    println!("   Rate: {:.2} blocks/sec", conc_count as f64 / conc_time.as_secs_f64());

    // Results
    let speedup = seq_time.as_secs_f64() / conc_time.as_secs_f64();
    println!("\nPerformance Summary:");
    println!("============================================");
    println!("   Sequential: {:?}", seq_time);
    println!("   Concurrent: {:?}", conc_time);
    println!("   Speedup: {:.2}x faster", speedup);

    assert!(conc_count == seq_count, "Block counts must match");
    assert!(speedup > 1.0, "Concurrent should be faster");

    println!("\nTest passed!");

    Ok(())
}
