const DEFAULT_PAGE: u32 = 1;
const DEFAULT_PAGE_SIZE: u32 = 25;
const MAX_PAGE_SIZE: u32 = 100;

pub(crate) mod block;
pub(crate) mod call;
pub(crate) mod event;
pub(crate) mod extrinsic;
pub(crate) mod genesis;
pub(crate) mod metadata;
pub(crate) mod system;
pub(crate) mod trace;
