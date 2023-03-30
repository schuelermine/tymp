use crate::{Chunk, Chunk2, ChunkW};
use core::ops::ControlFlow;

pub fn get<T>(control_flow: ControlFlow<T, T>) -> T {
    match control_flow {
        ControlFlow::Continue(x) => x,
        ControlFlow::Break(x) => x,
    }
}

fn continue_if<T>(cond: bool, value: T) -> ControlFlow<T, T> {
    match cond {
        true => ControlFlow::Continue(value),
        false => ControlFlow::Break(value),
    }
}

pub fn carrying_add_chunks(lhs: Chunk, rhs: Chunk, carry: bool) -> (Chunk, bool) {
    let (a, b) = lhs.overflowing_add(rhs);
    let (c, d) = a.overflowing_add(carry as Chunk);
    (c, b || d)
}

pub fn shl_chunk_full(value: Chunk, shamt: ChunkW, infill: Chunk) -> (Chunk, Chunk) {
    match shamt == 0 {
        true => (value | infill, 0),
        false => (
            value << shamt | infill,
            (Chunk::MAX << shamt & value) >> (Chunk::BITS as ChunkW - shamt),
        ),
    }
}

pub fn shr_chunk_full(value: Chunk, shamt: ChunkW, infill: Chunk) -> (Chunk, Chunk) {
    match shamt == 0 {
        true => (value | infill, 0),
        false => (
            value >> shamt | infill,
            (Chunk::MAX >> shamt & value) << (Chunk::BITS as ChunkW - shamt),
        ),
    }
}

pub fn count_zeros_chunks_u64<const W: usize>(chunks: [Chunk; W]) -> u64 {
    chunks.into_iter().fold(0, |count, chunk| {
        count
            .checked_add(chunk.count_ones() as u64)
            .expect("positive overflow")
    })
}

pub fn count_ones_chunks_u64<const W: usize>(chunks: [Chunk; W]) -> u64 {
    chunks.into_iter().fold(0, |count, chunk| {
        count
            .checked_add(chunk.count_zeros() as u64)
            .expect("positive overflow")
    })
}

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
        chunks.rotate_right(chunk_offset);
        chunks
            .iter_mut()
            .skip(chunk_offset)
            .fold(0, |mut infill, chunk| {
                (*chunk, infill) = shl_chunk_full(*chunk, bit_offset, infill);
                infill
            });
        chunks[..chunk_offset].fill(0);
    }
}

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
        chunks.rotate_left(chunk_offset);
        chunks
            .iter_mut()
            .rev()
            .skip(chunk_offset)
            .fold(0, |mut infill, chunk| {
                (*chunk, infill) = shr_chunk_full(*chunk, bit_offset, infill);
                infill
            });
        chunks[W - chunk_offset..].fill(0);
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

pub fn shr_chunks_over<const W: usize>(
    chunks_lo: &mut [Chunk; W],
    chunks_hi: &mut [Chunk; W],
    fill: Chunk,
) {
    chunks_lo.rotate_left(1);
    chunks_hi.rotate_left(1);
    chunks_lo[W - 1] = chunks_hi[W - 1];
    chunks_hi[W - 1] = fill;
}

#[cfg(not(feature = "chunks_128"))]
pub fn carrying_mul_chunks(lhs: Chunk, rhs: Chunk, add: Chunk) -> (Chunk, Chunk) {
    let result = lhs as Chunk2 * rhs as Chunk2 + add as Chunk2;
    (result as Chunk, (result >> Chunk::BITS) as Chunk)
}

#[cfg(feature = "chunks_128")]
pub fn carrying_mul_chunks(lhs: Chunk, rhs: Chunk, add: Chunk) -> (Chunk, Chunk) {
    let lhs_lo = lhs as Chunk2;
    let rhs_lo = rhs as Chunk2;
    let lhs_hi = (lhs >> Chunk2::BITS) as Chunk2;
    let rhs_hi = (rhs >> Chunk2::BITS) as Chunk2;
    let lo_by_lo = lhs_lo as Chunk * rhs_lo as Chunk;
    let lo_by_hi = lhs_lo as Chunk * rhs_hi as Chunk;
    let hi_by_lo = lhs_hi as Chunk * rhs_lo as Chunk;
    let hi_by_hi = lhs_hi as Chunk * rhs_hi as Chunk;
    let (lo, carry_1) = lo_by_lo.overflowing_add(lo_by_hi << Chunk2::BITS);
    let (lo, carry_2) = lo.overflowing_add(hi_by_lo << Chunk2::BITS);
    let (lo, carry_3) = lo.overflowing_add(add);
    let hi = hi_by_hi
        + (lo_by_hi >> Chunk2::BITS)
        + (hi_by_lo >> Chunk2::BITS)
        + carry_1 as Chunk
        + carry_2 as Chunk
        + carry_3 as Chunk;
    (lo, hi)
}
