#![no_std]
#![forbid(unsafe_code)]

use cfg_if::cfg_if;

#[cfg(all(feature = "byte_chunks", feature = "wide_chunks"))]
compile_error!();

cfg_if! {
    if #[cfg(feature = "byte_chunks")] {
        type Chunk = u8;
        type IChunk = i8;
        type ChunkW = u8;
    } else if #[cfg(feature = "wide_chunks")] {
        type Chunk = u128;
        type IChunk = i128;
        type ChunkW = u8;
    } else {
        type Chunk = u64;
        type IChunk = i64;
        type ChunkW = u8;
    }
}

mod common;
mod i;
mod u;

pub use i::I;
pub use u::U;
