#![no_std]
#![forbid(unsafe_code)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "chunks_8")] {
        type Chunk = u8;
        type IChunk = i8;
        type Chunk2 = u16;
        type ChunkW = u8;
    } else if #[cfg(feature = "chunks_64")] {
        type Chunk = u64;
        type IChunk = i64;
        type Chunk2 = u128;
        type ChunkW = u8;
    } else if #[cfg(feature = "chunks_128")] {
        type Chunk = u128;
        type IChunk = i128;
        type Chunk2 = u64;
        type ChunkW = u8;
    } else {
        compile_error!("Enable one of the features chunks_8, chunks_64, or chunks_128 to select the desired chunk width");
    }
}

mod common;
mod i;
mod u;

pub use i::I;
pub use u::U;
