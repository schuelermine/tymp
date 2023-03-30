use core::{cmp::Ordering, iter::zip, ops::ControlFlow};

use crate::{
    common::{
        carrying_add_chunks, count_ones_chunks_u64, count_zeros_chunks_u64,
        leading_ones_chunks_u64, leading_zeros_chunks_u64, split_rotate_left_chunks,
        split_rotate_right_chunks, split_shl_chunks, split_shr_chunks, trailing_ones_chunks_u64,
        trailing_zeros_chunks_u64,
    },
    Chunk, ChunkW, IChunk, U,
};
use itertools::izip;

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
    pub fn carrying_add(self, rhs: Self, mut carry: bool) -> (Self, bool) {
        if W == 0 {
            return (I { chunks: [0; W] }, carry);
        }
        let mut chunks = [0; W];
        let mut iter = izip!(self.chunks, rhs.chunks, &mut chunks);
        let (last_chunk_l, last_chunk_r, last_dest) = iter.next_back().unwrap();
        carry = iter.fold(carry, |mut carry, (chunk_l, chunk_r, dest)| {
            (*dest, carry) = carrying_add_chunks(chunk_l, chunk_r, carry);
            carry
        });
        (*last_dest, carry) = carrying_add_chunk_as_signed(last_chunk_l, last_chunk_r, carry);
        (I { chunks }, carry)
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
    pub fn overflowing_add_unsigned(self, rhs: U<W>) -> (Self, bool) {
        let rhs = rhs.reinterpret_signed();
        let (result, overflow) = self.overflowing_add(rhs);
        (result, overflow ^ rhs.lt(Self::ZERO))
    }
    pub fn cmp(self, rhs: Self) -> Ordering {
        let mut iter = zip(self.chunks, rhs.chunks);
        let (chunk_l, chunk_r) = iter.next_back().unwrap();
        match cmp_chunk_as_signed(chunk_l, chunk_r) {
            Ordering::Equal => (),
            order => return order,
        }
        match iter.try_rfold((), |(), (chunk_l, chunk_r)| match chunk_l.cmp(&chunk_r) {
            Ordering::Equal => ControlFlow::Continue(()),
            order => ControlFlow::Break(order),
        }) {
            ControlFlow::Continue(()) => Ordering::Equal,
            ControlFlow::Break(order) => order,
        }
    }
    pub fn lt(self, rhs: Self) -> bool {
        self.cmp(rhs).is_lt()
    }
    pub fn le(self, rhs: Self) -> bool {
        self.cmp(rhs).is_ne()
    }
    pub fn gt(self, rhs: Self) -> bool {
        self.cmp(rhs).is_gt()
    }
    pub fn ge(self, rhs: Self) -> bool {
        self.cmp(rhs).is_ge()
    }
    pub fn eq(self, rhs: Self) -> bool {
        self.cmp(rhs).is_eq()
    }
    pub fn ne(self, rhs: Self) -> bool {
        self.cmp(rhs).is_ne()
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
