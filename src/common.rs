use core::{
    cmp::Ordering,
    ops::{BitOrAssign, ControlFlow},
};

use discard_while::discard_while;

pub trait ChunkType: Sized + Copy + BitOrAssign + Eq + Ord {
    const BITS: Self::BitCounter;
    const MAX: Self;
    const ONE: Self;
    const ZERO: Self;
    const LEADING_ONE: Self;
    const LEADING_ZERO: Self;
    type BitCounter: ChunkBitCounter<Self>;
    fn count_ones(self) -> Self::BitCounter;
    fn count_zeros(self) -> Self::BitCounter;
    fn leading_ones(self) -> Self::BitCounter;
    fn leading_zeros(self) -> Self::BitCounter;
    fn trailing_ones(self) -> Self::BitCounter;
    fn trailing_zeros(self) -> Self::BitCounter;
    fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool);
    fn carrying_add_as_signed(self, rhs: Self, carry: bool) -> (Self, bool);
    fn shl_chunk_full(self, shamt: Self::BitCounter, infill: Self) -> (Self, Self);
    fn shr_chunk_full(self, shamt: Self::BitCounter, infill: Self) -> (Self, Self);
    fn cmp_as_signed(self, other: Self) -> Ordering;
    fn reverse_bits(self) -> Self;
}

pub trait ChunkBitCounter<Chunk: ChunkType>: Copy + PartialEq {
    const ZERO: Self;
    fn is_valid(self) -> bool;
}

pub trait TotalBitCounter<Chunk: ChunkType>: Sized {
    const ZERO: Self;
    fn from_chunk_count(count: usize) -> Option<Self>;
    fn checked_add(self, rhs: Chunk::BitCounter) -> Option<Self>;
    fn split(self) -> (usize, Chunk::BitCounter);
}

pub fn get<T>(cf: ControlFlow<T, T>) -> T {
    match cf {
        ControlFlow::Continue(x) => x,
        ControlFlow::Break(x) => x,
    }
}

pub fn break_if<T>(cond: bool, value: T) -> ControlFlow<T, T> {
    match cond {
        true => ControlFlow::Break(value),
        false => ControlFlow::Continue(value),
    }
}

pub fn count_zeros_chunks<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>>(
    chunks: [Chunk; W],
) -> Option<Total> {
    chunks.into_iter().try_fold(Total::ZERO, |count, chunk| {
        count.checked_add(chunk.count_zeros())
    })
}

pub fn count_ones_chunks<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>>(
    chunks: [Chunk; W],
) -> Option<Total> {
    chunks.into_iter().try_fold(Total::ZERO, |count, chunk| {
        count.checked_add(chunk.count_ones())
    })
}

pub fn leading_zeros_chunks<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>>(
    chunks: [Chunk; W],
) -> Option<Total> {
    let (chunk, count) = discard_while(chunks.into_iter().rev(), |&chunk| chunk == Chunk::ZERO);
    Total::from_chunk_count(count)?.checked_add(if let Some(chunk) = chunk {
        chunk.leading_zeros()
    } else {
        Chunk::BitCounter::ZERO
    })
}

pub fn leading_ones_chunks<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>>(
    chunks: [Chunk; W],
) -> Option<Total> {
    let (chunk, count) = discard_while(chunks.into_iter().rev(), |&chunk| chunk == Chunk::MAX);
    Total::from_chunk_count(count)?.checked_add(if let Some(chunk) = chunk {
        chunk.leading_ones()
    } else {
        Chunk::BitCounter::ZERO
    })
}

pub fn trailing_zeros_chunks<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>>(
    chunks: [Chunk; W],
) -> Option<Total> {
    let (chunk, count) = discard_while(chunks, |&chunk| chunk == Chunk::ZERO);
    Total::from_chunk_count(count)?.checked_add(if let Some(chunk) = chunk {
        chunk.trailing_zeros()
    } else {
        Chunk::BitCounter::ZERO
    })
}

pub fn trailing_ones_chunks<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>>(
    chunks: [Chunk; W],
) -> Option<Total> {
    let (chunk, count) = discard_while(chunks, |&chunk| chunk == Chunk::MAX);
    Total::from_chunk_count(count)?.checked_add(if let Some(chunk) = chunk {
        chunk.trailing_ones()
    } else {
        Chunk::BitCounter::ZERO
    })
}

pub fn split_shl_chunks<const W: usize, Chunk: ChunkType>(
    chunks: &mut [Chunk; W],
    chunk_offset: usize,
    bit_offset: Chunk::BitCounter,
) {
    if chunk_offset == 0 {
        chunks.iter_mut().fold(Chunk::ZERO, |mut infill, chunk| {
            (*chunk, infill) = chunk.shl_chunk_full(bit_offset, infill);
            infill
        });
    } else {
        chunks.rotate_right(chunk_offset);
        chunks
            .iter_mut()
            .skip(chunk_offset)
            .fold(Chunk::ZERO, |mut infill, chunk| {
                (*chunk, infill) = chunk.shl_chunk_full(bit_offset, infill);
                infill
            });
        chunks[..chunk_offset].fill(Chunk::ZERO);
    }
}

pub fn split_shr_chunks<const W: usize, Chunk: ChunkType>(
    chunks: &mut [Chunk; W],
    chunk_offset: usize,
    bit_offset: Chunk::BitCounter,
) {
    if chunk_offset == 0 {
        chunks.iter_mut().rfold(Chunk::ZERO, |mut infill, chunk| {
            (*chunk, infill) = (*chunk).shr_chunk_full(bit_offset, infill);
            infill
        });
    } else {
        chunks.rotate_left(chunk_offset);
        chunks
            .iter_mut()
            .rev()
            .skip(chunk_offset)
            .fold(Chunk::ZERO, |mut infill, chunk| {
                (*chunk, infill) = chunk.shr_chunk_full(bit_offset, infill);
                infill
            });
        chunks[W - chunk_offset..].fill(Chunk::ZERO);
    }
}

pub fn split_rotate_left_chunks<const W: usize, Chunk: ChunkType>(
    chunks: &mut [Chunk; W],
    chunk_offset: usize,
    bit_offset: Chunk::BitCounter,
) {
    chunks.rotate_right(chunk_offset);
    let infill = chunks.iter_mut().fold(Chunk::ZERO, |mut infill, chunk| {
        (*chunk, infill) = chunk.shl_chunk_full(bit_offset, infill);
        infill
    });
    chunks[0] |= infill;
}

pub fn split_rotate_right_chunks<const W: usize, Chunk: ChunkType>(
    chunks: &mut [Chunk; W],
    chunk_offset: usize,
    bit_offset: Chunk::BitCounter,
) {
    chunks.rotate_left(chunk_offset);
    let infill = chunks.iter_mut().rfold(Chunk::ZERO, |mut infill, chunk| {
        (*chunk, infill) = chunk.shr_chunk_full(bit_offset, infill);
        infill
    });
    chunks[W - 1] |= infill;
}
