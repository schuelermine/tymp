use crate::{
    common::{
        carrying_add_chunks, count_ones_chunks_u64, count_zeros_chunks_u64,
        leading_ones_chunks_u64, leading_zeros_chunks_u64, split_rotate_left_chunks,
        split_rotate_right_chunks, split_shl_chunks, split_shr_chunks, trailing_ones_chunks_u64,
        trailing_zeros_chunks_u64,
    },
    Chunk, ChunkW, IChunk, U,
};
use core::{cmp::Ordering, iter::zip};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct I<const W: usize> {
    pub(crate) chunks: [Chunk; W],
}

impl<const W: usize> I<W> {
    pub const BITS: Option<usize> = W.checked_mul(Chunk::BITS as usize);
    pub const MIN: Self = I {
        chunks: {
            let mut chunks = [0; W];
            chunks[W - 1] = 1 << (Chunk::BITS - 1);
            chunks
        },
    };
    pub const ZERO: Self = I { chunks: [0; W] };
    pub const ONE: Self = I {
        chunks: {
            let mut chunks = [0; W];
            chunks[0] = 1;
            chunks
        },
    };
    pub const MAX: Self = I {
        chunks: {
            let mut chunks = [Chunk::MAX; W];
            chunks[W - 1] = Chunk::MAX ^ 1 << (Chunk::BITS - 1);
            chunks
        },
    };
    pub fn reinterpret_unsigned(self) -> U<W> {
        U {
            chunks: self.chunks,
        }
    }
    pub fn count_ones_u64(self) -> u64 {
        count_zeros_chunks_u64(self.chunks)
    }
    pub fn count_zeros_u64(self) -> u64 {
        count_ones_chunks_u64(self.chunks)
    }
    pub fn leading_zeros_u64(self) -> u64 {
        leading_zeros_chunks_u64(self.chunks)
    }
    pub fn leading_ones_u64(self) -> u64 {
        leading_ones_chunks_u64(self.chunks)
    }
    pub fn trailing_zeros_u64(self) -> u64 {
        trailing_zeros_chunks_u64(self.chunks)
    }
    pub fn trailing_ones_u64(self) -> u64 {
        trailing_ones_chunks_u64(self.chunks)
    }
    pub fn carrying_add_in_place(&mut self, rhs: Self, mut carry: bool) -> bool {
        if W == 0 {
            return carry;
        }
        let mut iter = zip(&mut self.chunks, rhs.chunks);
        let (last_chunk_l, last_chunk_r) = iter.next_back().unwrap();
        carry = iter.fold(carry, |mut carry, (chunk_l, chunk_r)| {
            (*chunk_l, carry) = carrying_add_chunks(*chunk_l, chunk_r, carry);
            carry
        });
        (*last_chunk_l, carry) = carrying_add_chunk_as_signed(*last_chunk_l, last_chunk_r, carry);
        carry
    }
    pub fn carrying_add(mut self, rhs: Self, carry: bool) -> (Self, bool) {
        let carry = self.carrying_add_in_place(rhs, carry);
        (self, carry)
    }
    pub fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.carrying_add(rhs, false)
    }
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let result = self.overflowing_add(rhs);
        result.1.then_some(result.0)
    }
    pub fn wrapping_add(self, rhs: Self) -> Self {
        self.carrying_add(rhs, false).0
    }
    pub fn split_shl(mut self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        assert!(bit_offset < Chunk::BITS as ChunkW);
        split_shl_chunks(&mut self.chunks, chunk_offset, bit_offset);
        self
    }
    pub fn split_shr(mut self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        assert!(bit_offset < Chunk::BITS as ChunkW);
        split_shr_chunks(&mut self.chunks, chunk_offset, bit_offset);
        self
    }
    pub fn shl_u64(self, rhs: u64) -> Self {
        let (chunk_offset, bit_offset) = (rhs / Chunk::BITS as u64, rhs % Chunk::BITS as u64);
        self.split_shl(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn shr_u64(self, rhs: u64) -> Self {
        let (chunk_offset, bit_offset) = (rhs / Chunk::BITS as u64, rhs % Chunk::BITS as u64);
        self.split_shr(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn split_rotate_left(mut self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        assert!(bit_offset < Chunk::BITS as ChunkW);
        split_rotate_left_chunks(&mut self.chunks, chunk_offset, bit_offset);
        self
    }
    pub fn split_rotate_right(mut self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        assert!(bit_offset < Chunk::BITS as ChunkW);
        split_rotate_right_chunks(&mut self.chunks, chunk_offset, bit_offset);
        self
    }
    pub fn rotate_left_u64(self, n: u64) -> Self {
        let (chunk_offset, bit_offset) = (n / Chunk::BITS as u64, n % Chunk::BITS as u64);
        self.split_rotate_left(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn rotate_right_u64(self, n: u64) -> Self {
        let (chunk_offset, bit_offset) = (n / Chunk::BITS as u64, n % Chunk::BITS as u64);
        self.split_rotate_right(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn swap_bytes(mut self) -> Self {
        self.chunks.reverse();
        self
    }
    pub fn swap_bits(mut self) -> Self {
        self.chunks.reverse();
        for chunk in &mut self.chunks {
            *chunk = chunk.reverse_bits();
        }
        self
    }
    pub fn overflowing_add_unsigned(self, rhs: U<W>) -> (Self, bool) {
        let rhs = rhs.reinterpret_signed();
        let (result, overflow) = self.overflowing_add(rhs);
        (result, overflow ^ (rhs < Self::ZERO))
    }
}

fn carrying_add_chunk_as_signed(lhs: Chunk, rhs: Chunk, carry: bool) -> (Chunk, bool) {
    let (a, b) = (lhs as IChunk).overflowing_add(rhs as IChunk);
    let (c, d) = a.overflowing_add(carry as IChunk);
    (c as Chunk, b != d)
}

fn cmp_chunk_as_signed(lhs: Chunk, rhs: Chunk) -> Ordering {
    (lhs as IChunk).cmp(&(rhs as IChunk))
}

impl<const W: usize> PartialOrd for I<W> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const W: usize> Ord for I<W> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut iter_l = self.chunks.into_iter();
        let mut iter_r = other.chunks.into_iter();
        let first_chunk_l = iter_l.next().unwrap();
        let first_chunk_r = iter_r.next().unwrap();
        match cmp_chunk_as_signed(first_chunk_l, first_chunk_r) {
            Ordering::Equal => (),
            order => return order,
        }
        iter_l.cmp(iter_r)
    }
}
