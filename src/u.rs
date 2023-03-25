use crate::{
    common::{
        count_ones_chunks_u64, count_zeros_chunks_u64, leading_ones_chunks_u64,
        leading_zeros_chunks_u64, split_rotate_left_chunks, split_rotate_right_chunks,
        split_shl_chunks, split_shr_chunks, trailing_ones_chunks_u64, trailing_zeros_chunks_u64,
    },
    Chunk, ChunkW, I,
};
use itertools::izip;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct U<const W: usize> {
    pub(crate) chunks: [Chunk; W],
}

impl<const W: usize> U<W> {
    pub const BITS: Option<usize> = W.checked_mul(Chunk::BITS as usize);
    pub const MIN: Self = U { chunks: [0; W] };
    pub const ZERO: Self = Self::MIN;
    pub const MAX: Self = U {
        chunks: [Chunk::MAX; W],
    };
    pub fn reinterpret_signed(self) -> I<W> {
        I {
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
    pub fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
        if W == 0 {
            return (U { chunks: [0; W] }, carry);
        }
        let mut chunks = [0; W];
        let carry = izip!(self.chunks, rhs.chunks, &mut chunks).fold(
            carry,
            |mut carry, (chunk_l, chunk_r, dest)| {
                (*dest, carry) = chunk_l.carrying_add(chunk_r, carry);
                carry
            },
        );
        (U { chunks }, carry)
    }
    pub fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        self.carrying_add(rhs, false)
    }
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let result = self.carrying_add(rhs, false);
        match result.1 {
            true => None,
            false => Some(result.0),
        }
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
}
