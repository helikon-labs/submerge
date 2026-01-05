mod common;
use common::get_api_client;

use submerge_crystal::api::v1::client::{
    GetBlocksByReferenceRequest, GetBlocksByReferenceRequestPath, GetBlocksByReferenceResponse,
    GetBlocksRequest, GetBlocksRequestQuery, GetBlocksResponse, GetEventsRequest,
    GetEventsRequestQuery, GetEventsResponse, GetExtrinsicsRequest, GetExtrinsicsRequestQuery,
    GetExtrinsicsResponse, GetMetadataListRequest, GetMetadataListRequestQuery,
    GetMetadataListResponse,
};

#[tokio::test]
async fn get_blocks_test() -> anyhow::Result<()> {
    let client = get_api_client();

    let request = GetBlocksRequest {
        query: GetBlocksRequestQuery {
            page: Some(1),
            page_size: Some(1),
            ..Default::default()
        },
    };

    let response = client.get_blocks(request).await?;

    if let GetBlocksResponse::Ok(data) = response {
        assert!(!data.data.is_empty(), "No blocks returned");
    } else {
        panic!("Unexpected response: {:?}", response);
    }

    Ok(())
}

#[tokio::test]
async fn get_blocks_by_reference_test() -> anyhow::Result<()> {
    let client = get_api_client();

    // Get the latest block to have a valid block number for the next request
    let latest_block_request = GetBlocksRequest {
        query: GetBlocksRequestQuery {
            page: Some(1),
            page_size: Some(1),
            ..Default::default()
        },
    };

    let latest_block_response = client.get_blocks(latest_block_request).await?;
    let block_number = match latest_block_response {
        GetBlocksResponse::Ok(data) => {
            assert!(!data.data.is_empty(), "No blocks returned");
            data.data[0].number
        }
        _ => {
            panic!("Unexpected response: {:?}", latest_block_response);
        }
    };

    let request = GetBlocksByReferenceRequest {
        path: GetBlocksByReferenceRequestPath {
            block_ref: block_number.to_string(),
        },
    };

    let response = client.get_blocks_by_reference(request).await?;

    if let GetBlocksByReferenceResponse::Ok(data) = response {
        assert!(
            !data.is_empty(),
            "No block returned for number {}",
            block_number
        );
        assert_eq!(data[0].number, block_number);
    } else {
        panic!("Unexpected response: {:?}", response);
    }

    Ok(())
}

#[tokio::test]
async fn get_events_test() -> anyhow::Result<()> {
    let client = get_api_client();

    let request = GetEventsRequest {
        query: GetEventsRequestQuery {
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    };

    let response = client.get_events(request).await?;

    match response {
        GetEventsResponse::Ok(data) => {
            assert!(!data.data.is_empty(), "No events returned");
            assert!(data.data.len() <= 10);
        }
        _ => {
            panic!("Unexpected response: {:?}", response);
        }
    }

    Ok(())
}

#[tokio::test]
async fn get_extrinsics_test() -> anyhow::Result<()> {
    let client = get_api_client();

    let request = GetExtrinsicsRequest {
        query: GetExtrinsicsRequestQuery {
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    };

    let response = client.get_extrinsics(request).await?;

    match response {
        GetExtrinsicsResponse::Ok(data) => {
            assert!(!data.data.is_empty(), "No extrinsics returned");
            assert!(data.data.len() <= 10)
        }
        _ => {
            panic!("Unexpected response: {:?}", response);
        }
    }

    Ok(())
}

#[tokio::test]
async fn get_events_bad_page_size_fails() -> anyhow::Result<()> {
    let client = get_api_client();

    // page_size of 0 should be invalid per typical API behaviour
    let request = GetEventsRequest {
        query: GetEventsRequestQuery {
            page: Some(1),
            page_size: Some(0),
            ..Default::default()
        },
    };

    let err = client
        .get_events(request)
        .await
        .expect_err("expected validation error");

    assert_eq!(err.to_string(), "parameter validation");

    Ok(())
}

#[tokio::test]
async fn get_events_pagination_does_not_overlap() -> anyhow::Result<()> {
    let client = get_api_client();

    let first_page_req = GetEventsRequest {
        query: GetEventsRequestQuery {
            page: Some(1),
            page_size: Some(5),
            ..Default::default()
        },
    };
    let second_page_req = GetEventsRequest {
        query: GetEventsRequestQuery {
            page: Some(2),
            page_size: Some(5),
            ..Default::default()
        },
    };

    let first_page = match client.get_events(first_page_req).await? {
        GetEventsResponse::Ok(d) => d.data,
        other => panic!("unexpected: {:?}", other),
    };

    let second_page = match client.get_events(second_page_req).await? {
        GetEventsResponse::Ok(d) => d.data,
        other => panic!("unexpected: {:?}", other),
    };

    for item in &second_page {
        assert!(
            !first_page
                .iter()
                .any(|x| format!("{}-{}", x.block_number, x.index)
                    == format!("{}-{}", item.block_number, item.index)),
            "pagination overlap for id={:?}",
            format!("{}-{}", item.block_number, item.index)
        );
    }

    Ok(())
}

#[tokio::test]
async fn get_events_rejects_page_size_too_large() {
    let client = get_api_client();

    let err = client
        .get_events(GetEventsRequest {
            query: GetEventsRequestQuery {
                page: Some(1),
                page_size: Some(10_000),
                ..Default::default()
            },
        })
        .await
        .expect_err("expected validation error");

    let msg = err.to_string();

    assert_eq!(msg, "parameter validation");
}

#[tokio::test]
async fn get_events_empty_page_is_ok() -> anyhow::Result<()> {
    let client = get_api_client();

    let response = client
        .get_events(GetEventsRequest {
            query: GetEventsRequestQuery {
                page: Some(99999999), // likely empty
                page_size: Some(10),
                ..Default::default()
            },
        })
        .await?;

    let data = match response {
        GetEventsResponse::Ok(data) => data,
        other => panic!("unexpected response: {:?}", other),
    };

    assert!(data.data.is_empty());

    Ok(())
}

#[tokio::test]
async fn concurrent_requests() -> anyhow::Result<()> {
    use tokio::task::JoinSet;

    let client = get_api_client();
    let mut tasks = JoinSet::new();

    for _ in 0..5 {
        let client = client.clone();
        tasks.spawn(async move {
            let response = client
                .get_metadata_list(GetMetadataListRequest {
                    query: GetMetadataListRequestQuery {
                        page: Some(1),
                        page_size: Some(5),
                        ..Default::default()
                    },
                })
                .await;

            // Match on the response enum
            let data = match response {
                Ok(GetMetadataListResponse::Ok(data)) => data,
                other => panic!("unexpected response variant: {:?}", other),
            };

            // Assert each response has exactly 5 items
            assert_eq!(data.data.len(), 5, "expected 5 items per page");
        });
    }

    // Await all tasks and propagate panic if any failed
    while let Some(res) = tasks.join_next().await {
        res.unwrap()
    }

    Ok(())
}
