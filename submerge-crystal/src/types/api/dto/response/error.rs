use serde::Serialize;
use utoipa::ToResponse;

use crate::types::api::error::APIErrorBody;

/// Invalid path or query parameter(s).
#[derive(Serialize, ToResponse)]
#[response(
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct BadRequest(pub APIErrorBody);

/// Rate limit exceeded.
#[derive(Serialize, ToResponse)]
#[response(
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
        ("X-Retry-After" = u32),
        ("Retry-After" = u32),
    ),
)]
pub struct TooManyRequests(pub APIErrorBody);

/// Internal server error.
#[derive(Serialize, ToResponse)]
#[response(
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct InternalServerError(pub APIErrorBody);

/// Item not found.
#[derive(Serialize, ToResponse)]
#[response(
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
)]
pub struct NotFound(pub APIErrorBody);
