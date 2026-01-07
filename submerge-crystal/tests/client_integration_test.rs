mod common;
use common::get_api_client;
use serial_test::serial;

use submerge_crystal::api::v1::client::{
    BlocksByReferenceRequest, BlocksByReferenceRequestPath, BlocksByReferenceResponse,
    BlocksRequest, BlocksRequestQuery, BlocksResponse, EventsRequest, EventsRequestQuery,
    EventsResponse, ExtrinsicsRequest, ExtrinsicsRequestQuery, ExtrinsicsResponse,
    MetadataListRequest, MetadataListRequestQuery, MetadataListResponse,
};

pub async fn retry_request<F, Fut, T, E>(f: F) -> Result<T, E>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T, E>> + Send,
{
    const MAX_ATTEMPTS: u64 = 10;

    for attempt in 1..=MAX_ATTEMPTS {
        let fut = f();
        match fut.await {
            Ok(val) => return Ok(val),
            Err(err) => {
                if attempt < MAX_ATTEMPTS {
                    let backoff_secs = 4_u64.pow(attempt as u32); // 2, 4, 8, 16 seconds
                    tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
                } else {
                    return Err(err);
                }
            }
        }
    }

    unreachable!()
}

#[tokio::test]
#[serial]
async fn get_blocks_test() -> anyhow::Result<()> {
    let client = get_api_client();

    let request = BlocksRequest {
        query: BlocksRequestQuery {
            page: Some(1),
            page_size: Some(1),
            ..Default::default()
        },
    };

    let response = retry_request(|| {
        let client = client.clone();
        let request = request.clone();
        async move { client.blocks(request).await }
    })
    .await?;

    if let BlocksResponse::Ok(data) = response {
        assert!(!data.data.is_empty(), "No blocks returned");
    } else {
        panic!("Unexpected response: {:?}", response);
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_blocks_by_reference_test() -> anyhow::Result<()> {
    let client = get_api_client();

    //  the latest block to have a valid block number for the next request
    let latest_block_request = BlocksRequest {
        query: BlocksRequestQuery {
            page: Some(1),
            page_size: Some(1),
            ..Default::default()
        },
    };

    let latest_block_response = retry_request(|| {
        let client = client.clone();
        let request = latest_block_request.clone();
        async move { client.blocks(request).await }
    })
    .await?;

    let block_number = match latest_block_response {
        BlocksResponse::Ok(data) => {
            assert!(!data.data.is_empty(), "No blocks returned");
            data.data[0].number
        }
        _ => {
            panic!("Unexpected response: {:?}", latest_block_response);
        }
    };

    let request = BlocksByReferenceRequest {
        path: BlocksByReferenceRequestPath {
            block_ref: block_number.to_string(),
        },
    };

    let response = retry_request(|| {
        let client = client.clone();
        let request = request.clone();
        async move { client.blocks_by_reference(request).await }
    })
    .await?;

    if let BlocksByReferenceResponse::Ok(data) = response {
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
#[serial]
async fn get_events_test() -> anyhow::Result<()> {
    let client = get_api_client();

    let request = EventsRequest {
        query: EventsRequestQuery {
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    };

    let response = retry_request(|| {
        let client = client.clone();
        let request = request.clone();
        async move { client.events(request).await }
    })
    .await?;

    match response {
        EventsResponse::Ok(data) => {
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
#[serial]
async fn get_extrinsics_test() -> anyhow::Result<()> {
    let client = get_api_client();

    let request = ExtrinsicsRequest {
        query: ExtrinsicsRequestQuery {
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    };

    let response = retry_request(|| {
        let client = client.clone();
        let request = request.clone();
        async move { client.extrinsics(request).await }
    })
    .await?;

    match response {
        ExtrinsicsResponse::Ok(data) => {
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
#[serial]
async fn get_events_bad_page_size_fails() -> anyhow::Result<()> {
    let client = get_api_client();

    // page_size of 0 should be invalid per typical API behaviour
    let request = EventsRequest {
        query: EventsRequestQuery {
            page: Some(1),
            page_size: Some(0),
            ..Default::default()
        },
    };

    let err = client
        .events(request)
        .await
        .expect_err("expected validation error");

    assert_eq!(err.to_string(), "parameter validation");

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_events_pagination_does_not_overlap() -> anyhow::Result<()> {
    let client = get_api_client();

    let first_page_req = EventsRequest {
        query: EventsRequestQuery {
            page: Some(1),
            page_size: Some(5),
            ..Default::default()
        },
    };
    let second_page_req = EventsRequest {
        query: EventsRequestQuery {
            page: Some(2),
            page_size: Some(5),
            ..Default::default()
        },
    };

    let first_page = match retry_request(|| {
        let client = client.clone();
        let request = first_page_req.clone();
        async move { client.events(request).await }
    })
    .await?
    {
        EventsResponse::Ok(d) => d.data,
        other => panic!("unexpected: {:?}", other),
    };

    let second_page = match retry_request(|| {
        let client: submerge_crystal::api::v1::client::SubmergeCrystalApiV1Client = client.clone();
        let request = second_page_req.clone();
        async move { client.events(request).await }
    })
    .await?
    {
        EventsResponse::Ok(d) => d.data,
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
#[serial]
async fn get_events_rejects_page_size_too_large() {
    let client = get_api_client();

    let err = client
        .events(EventsRequest {
            query: EventsRequestQuery {
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
#[serial]
async fn get_events_empty_page_is_ok() -> anyhow::Result<()> {
    let client = get_api_client();

    let response = retry_request(|| {
        let client = client.clone();
        async move {
            client
                .events(EventsRequest {
                    query: EventsRequestQuery {
                        page: Some(99999999), // likely empty
                        page_size: Some(10),
                        ..Default::default()
                    },
                })
                .await
        }
    })
    .await?;

    let data = match response {
        EventsResponse::Ok(data) => data,
        other => panic!("unexpected response: {:?}", other),
    };

    assert!(data.data.is_empty());

    Ok(())
}

#[tokio::test]
#[serial]
async fn concurrent_requests() -> anyhow::Result<()> {
    use tokio::task::JoinSet;

    let client = get_api_client();
    let mut tasks = JoinSet::new();

    for _ in 0..5 {
        let client = client.clone();
        tasks.spawn(async move {
            let response = retry_request(|| {
                let client = client.clone();
                async move {
                    client
                        .metadata_list(MetadataListRequest {
                            query: MetadataListRequestQuery {
                                page: Some(1),
                                page_size: Some(5),
                                ..Default::default()
                            },
                        })
                        .await
                }
            })
            .await;

            // Match on the response enum
            let data = match response {
                Ok(MetadataListResponse::Ok(data)) => data,
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
