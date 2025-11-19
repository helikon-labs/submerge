use std::sync::Arc;

#[tokio::test]
#[ignore] // Run with: cargo test --test concurrent_safety_test -- --ignored --nocapture
async fn test_block_count_verification() -> anyhow::Result<()> {
    println!("\nTesting block count verification safety...");

    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "wss://polkadot.dotters.network".to_string());

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

    // Test with last 100 blocks - all should be fetched
    let block_count = 100;
    let start = finalized_number.saturating_sub(block_count - 1);
    let end = finalized_number;

    println!("\nFetching {} blocks (#{} to #{})", block_count, start, end);

    let mut rx = submerge_crystal::worker::processor::concurrent::fetch_hashes_range(
        &client, start, end, 100,
    )
    .await;

    let mut received_count = 0;
    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(result) = rx.recv().await {
        received_count += 1;
        match result {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    println!("Received: {} blocks", received_count);
    println!("  - Success: {}", success_count);
    println!("  - Errors: {}", error_count);

    // Verify we received exactly the expected number of blocks
    assert_eq!(
        received_count, block_count,
        "Expected {} blocks but received {}",
        block_count, received_count
    );

    // All blocks should succeed (none should be missing)
    assert_eq!(
        success_count, block_count,
        "Expected all {} blocks to succeed, but only {} succeeded",
        block_count, success_count
    );

    assert_eq!(
        error_count, 0,
        "Expected no errors, but got {}",
        error_count
    );

    println!("\nBlock count verification passed!");
    println!("All {} blocks were accounted for", block_count);

    Ok(())
}
