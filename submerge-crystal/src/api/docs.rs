use crate::types::api::{
    dto::{
        pagination::PaginationData,
        response::{
            block::{BlockDTO, BlockList, PaginatedBlockList},
            call::{CallDTO, PaginatedCallList},
            error::{BadRequest, InternalServerError, NotFound, TooManyRequests},
            extrinsic::{ExtrinsicList, PaginatedExtrinsicList},
        },
    },
    error::APIErrorBody,
};

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Submerge Crystal API v1",
        description = "REST API for Submerge Crystal, the core indexer component of Submerge.\n\nSubmerge API endpoints are grouped under five resource sets:\n- Blocks\n- Calls\n- Events\n- Extrinsics\n- Genesis\n- Metadata\n- Traces\n\nPublic API is limited by 5 requests per second.\nRequest query parameters use `snake_case`; response fields use `camelCase`.",
        version = "1.0.0",
        contact(
            name = "Helikon Labs",
            url = "https://submerge.io",
            email = "info@helikon.io"
        ),
        license(
            name = "GPLv3",
            url = "https://www.gnu.org/licenses/gpl-3.0.html",
        )
    ),
    servers(
        (
            url = "https://coretime-polkadot.crystal.submerge.io/api/v1",
            description = "API preview deployment for Polkadot Coretime.",
        ),
        (
            url = "https://{chain}.crystal.submerge.io/api/v1",
            description = "Submerge Crystal production API per deployed chain.",
            variables(
                ("chain" = (
                    default = "coretime-polkadot",
                    description = "Chain subdomain",
                    enum_values(
                        "polkadot",
                        "asset-hub-polkadot",
                        "bridge-hub-polkadot",
                        "collectives-polkadot",
                        "coretime-polkadot",
                        "people-polkadot",
                        "kusama",
                        "asset-hub-kusama",
                        "bridge-hub-kusama",
                        "coretime-kusama",
                        "people-kusama",
                    ),
                )
            )),
        )
    ),
    tags(
        (name = "block", description = "Endpoints related to blocks."),
        (name = "call", description = "Endpoints related to calls in extrinsics."),
        (name = "event", description = "Endpoints related to events."),
        (name = "extrinsic", description = "Endpoints related to extrinsics."),
        (name = "genesis", description = "Endpoints related to genesis records."),
        (name = "metadata", description = "Endpoints related to metadata."),
        (name = "trace", description = "Endpoints related to block traces.")
    ),
    paths(
        // block
        crate::api::v1::block::get_blocks,
        crate::api::v1::block::get_blocks_by_reference,
        // call
        crate::api::v1::call::get_calls,
        crate::api::v1::call::get_calls_by_block_reference,
        crate::api::v1::call::get_calls_by_block_reference_and_extrinsic_index,
        crate::api::v1::call::get_calls_by_extrinsic_hash,
        crate::api::v1::call::get_call_by_hash,
        crate::api::v1::call::get_call_args_by_hash,
        crate::api::v1::call::get_parent_call_by_hash,
        crate::api::v1::call::get_sub_calls_by_hash,
        crate::api::v1::call::get_call_extrinsic_by_hash,
        // extrinsic
        crate::api::v1::extrinsic::get_extrinsics,
        crate::api::v1::extrinsic::get_extrinsics_by_block_reference,
        crate::api::v1::extrinsic::get_extrinsics_by_block_reference_and_index,
        crate::api::v1::extrinsic::get_extrinsic_by_hash,
        crate::api::v1::extrinsic::get_extrinsic_root_call_by_hash,
    ),
    components(
        schemas(BlockDTO, CallDTO, PaginationData, APIErrorBody),
        responses(
            BlockList, PaginatedBlockList, PaginatedCallList, ExtrinsicList, PaginatedExtrinsicList,
            BadRequest, TooManyRequests, InternalServerError, NotFound,
        ),
    ),
)]
pub struct APIDoc;
