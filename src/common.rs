use crate::{Chunk, ChunkW, IChunk};
use core::ops::ControlFlow;

#[inline]
pub fn get<T>(control_flow: ControlFlow<T, T>) -> T {
    match control_flow {
        ControlFlow::Continue(x) => x,
        ControlFlow::Break(x) => x,
    }
}

#[inline]
fn continue_if<T>(cond: bool, value: T) -> ControlFlow<T, T> {
    match cond {
        true => ControlFlow::Continue(value),
        false => ControlFlow::Break(value),
    }
}

#[inline]
pub fn carrying_add_chunk_as_signed(lhs: Chunk, rhs: Chunk, carry: bool) -> (Chunk, bool) {
    let (result, carry) = (lhs as IChunk).carrying_add(rhs as IChunk, carry);
    (result as Chunk, carry)
}

#[inline]
pub fn shl_chunk_full(value: Chunk, shamt: ChunkW, infill: Chunk) -> (Chunk, Chunk) {
    match shamt == 0 {
        true => (value | infill, 0),
        false => (
            value << shamt | infill,
            (chunk_mask(0, shamt) & value) >> (Chunk::BITS as ChunkW - shamt),
        ),
    }
}

#[inline]
pub fn shr_chunk_full(value: Chunk, shamt: ChunkW, infill: Chunk) -> (Chunk, Chunk) {
    if shamt == 0 {
        (value | infill, 0)
    } else {
        (
            value >> shamt | infill,
            (chunk_mask(shamt, 0) & value) << (Chunk::BITS as ChunkW - shamt),
        )
    }
}

#[inline]
fn chunk_mask(zeros_l: ChunkW, zeros_r: ChunkW) -> Chunk {
    Chunk::MAX << zeros_r & Chunk::MAX >> zeros_l
}

#[inline]
pub fn count_zeros_chunks_u64<const W: usize>(chunks: [Chunk; W]) -> u64 {
    chunks.into_iter().fold(0, |count, chunk| {
        count
            .checked_add(chunk.count_ones() as u64)
            .expect("positive overflow")
    })
}

#[inline]
pub fn count_ones_chunks_u64<const W: usize>(chunks: [Chunk; W]) -> u64 {
    chunks.into_iter().fold(0, |count, chunk| {
        count
            .checked_add(chunk.count_zeros() as u64)
            .expect("positive overflow")
    })
}

#[inline]
pub fn leading_zeros_chunks_u64<const W: usize>(chunks: [Chunk; W]) -> u64 {
    get(chunks.into_iter().try_rfold(0, |count: u64, chunk| {
        let all_zeros = chunk == 0;
        let chunk_leading_zeros = match all_zeros {
            true => Chunk::BITS,
            false => chunk.leading_zeros(),
        } as u64;
        let count = count
            .checked_add(chunk_leading_zeros)
            .expect("positive overflow");
        continue_if(all_zeros, count)
    }))
}

#[inline]
pub fn leading_ones_chunks_u64<const W: usize>(chunks: [Chunk; W]) -> u64 {
    get(chunks.into_iter().try_rfold(0, |count: u64, chunk| {
        let all_ones = chunk == Chunk::MAX;
        let chunk_leading_zeros = match all_ones {
            true => Chunk::BITS,
            false => chunk.leading_ones(),
        } as u64;
        let count = count
            .checked_add(chunk_leading_zeros)
            .expect("positive overflow");
        continue_if(all_ones, count)
    }))
}

#[inline]
pub fn trailing_zeros_chunks_u64<const W: usize>(chunks: [Chunk; W]) -> u64 {
    get(chunks.into_iter().try_fold(0, |count: u64, chunk| {
        let all_zeros = chunk == 0;
        let chunk_leading_zeros = match all_zeros {
            true => Chunk::BITS,
            false => chunk.trailing_zeros(),
        } as u64;
        let count = count
            .checked_add(chunk_leading_zeros)
            .expect("positive overflow");
        continue_if(all_zeros, count)
    }))
}

#[inline]
pub fn trailing_ones_chunks_u64<const W: usize>(chunks: [Chunk; W]) -> u64 {
    get(chunks.into_iter().try_fold(0, |count: u64, chunk| {
        let all_ones = chunk == Chunk::MAX;
        let chunk_leading_zeros = match all_ones {
            true => Chunk::BITS,
            false => chunk.trailing_ones(),
        } as u64;
        let count = count
            .checked_add(chunk_leading_zeros)
            .expect("positive overflow");
        continue_if(all_ones, count)
    }))
}

#[inline]
pub fn split_shl_chunks<const W: usize>(
    chunks: &mut [Chunk; W],
    chunk_offset: usize,
    bit_offset: ChunkW,
) {
    if chunk_offset == 0 {
        chunks.iter_mut().fold(0, |mut infill, chunk| {
            (*chunk, infill) = shl_chunk_full(*chunk, bit_offset, infill);
            infill
        });
    } else {
        let mut infill = 0;
        for i in 0..W - chunk_offset {
            let [src, dest] = chunks.get_many_mut([i, i + chunk_offset]).ok().unwrap();
            (*dest, infill) = shl_chunk_full(*src, bit_offset, infill);
        }
        chunks[..chunk_offset].fill(0);
    }
}

#[inline]
pub fn split_shr_chunks<const W: usize>(
    chunks: &mut [Chunk; W],
    chunk_offset: usize,
    bit_offset: ChunkW,
) {
    if chunk_offset == 0 {
        chunks.iter_mut().rfold(0, |mut infill, chunk| {
            (*chunk, infill) = shr_chunk_full(*chunk, bit_offset, infill);
            infill
        });
    } else {
        let mut infill = 0;
        for i in 0..W - chunk_offset {
            let [src, dest] = chunks.get_many_mut([i + chunk_offset, i]).ok().unwrap();
            (*dest, infill) = shr_chunk_full(*src, bit_offset, infill);
        }
        chunks[W - 1 - chunk_offset..].fill(0);
    }
}

pub fn split_rotate_left_chunks<const W: usize>(
    chunks: &mut [Chunk; W],
    chunk_offset: usize,
    bit_offset: ChunkW,
) {
    chunks.rotate_right(chunk_offset);
    let infill = chunks.iter_mut().fold(0, |mut infill, chunk| {
        (*chunk, infill) = shl_chunk_full(*chunk, bit_offset, infill);
        infill
    });
    chunks[0] |= infill;
}

pub fn split_rotate_right_chunks<const W: usize>(
    chunks: &mut [Chunk; W],
    chunk_offset: usize,
    bit_offset: ChunkW,
) {
    chunks.rotate_left(chunk_offset);
    let infill = chunks.iter_mut().rfold(0, |mut infill, chunk| {
        (*chunk, infill) = shr_chunk_full(*chunk, bit_offset, infill);
        infill
    });
    chunks[W - 1] |= infill;
}
