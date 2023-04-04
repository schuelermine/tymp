use crate::{
    common::{
        carrying_add_chunks, count_ones_chunks_u64, count_zeros_chunks_u64,
        leading_ones_chunks_u64, leading_zeros_chunks_u64, split_rotate_left_chunks,
        split_rotate_right_chunks, split_shl_chunks, split_shr_chunks, trailing_ones_chunks_u64,
        trailing_zeros_chunks_u64,
    },
    Chunk, ChunkW, IChunk, U,
};
use core::{
    cmp::Ordering,
    iter::zip,
    ops::{Add, AddAssign, Shl, ShlAssign, Shr, ShrAssign},
};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "clone", derive(Clone))]
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
    pub fn overflowing_add_in_place(&mut self, rhs: Self) -> bool {
        self.carrying_add_in_place(rhs, false)
    }
    pub fn wrapping_add_in_place(&mut self, rhs: Self) {
        self.overflowing_add_in_place(rhs);
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
    pub fn split_overflowing_shl_in_place(
        &mut self,
        chunk_offset: usize,
        bit_offset: ChunkW,
    ) -> bool {
        let overflow = chunk_offset >= W;
        let chunk_offset = chunk_offset % W;
        assert!(bit_offset < Chunk::BITS as ChunkW);
        split_shl_chunks(&mut self.chunks, chunk_offset, bit_offset);
        overflow
    }
    pub fn split_overflowing_shr_in_place(
        &mut self,
        chunk_offset: usize,
        bit_offset: ChunkW,
    ) -> bool {
        let overflow = chunk_offset >= W;
        let chunk_offset = chunk_offset % W;
        assert!(bit_offset < Chunk::BITS as ChunkW);
        split_shr_chunks(&mut self.chunks, chunk_offset, bit_offset);
        overflow
    }
    pub fn overflowing_shl_u64_in_place(&mut self, rhs: u64) -> bool {
        let chunk_offset = rhs / Chunk::BITS as u64;
        let bit_offset = rhs % Chunk::BITS as u64;
        self.split_overflowing_shl_in_place(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn overflowing_shr_u64_in_place(&mut self, rhs: u64) -> bool {
        let chunk_offset = rhs / Chunk::BITS as u64;
        let bit_offset = rhs % Chunk::BITS as u64;
        self.split_overflowing_shr_in_place(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn split_wrapping_shl_in_place(&mut self, chunk_offset: usize, bit_offset: ChunkW) {
        self.split_overflowing_shl_in_place(chunk_offset, bit_offset);
    }
    pub fn split_wrapping_shr_in_place(&mut self, chunk_offset: usize, bit_offset: ChunkW) {
        self.split_overflowing_shr_in_place(chunk_offset, bit_offset);
    }
    pub fn wrapping_shl_u64_in_place(&mut self, rhs: u64) {
        self.overflowing_shr_u64_in_place(rhs);
    }
    pub fn wrapping_shr_u64_in_place(&mut self, rhs: u64) {
        self.overflowing_shr_u64_in_place(rhs);
    }
    pub fn split_overflowing_shl(
        mut self,
        chunk_offset: usize,
        bit_offset: ChunkW,
    ) -> (Self, bool) {
        let overflow = self.split_overflowing_shl_in_place(chunk_offset, bit_offset);
        (self, overflow)
    }
    pub fn split_overflowing_shr(
        mut self,
        chunk_offset: usize,
        bit_offset: ChunkW,
    ) -> (Self, bool) {
        let overflow = self.split_overflowing_shr_in_place(chunk_offset, bit_offset);
        (self, overflow)
    }
    pub fn overflowing_shl_u64(self, rhs: u64) -> (Self, bool) {
        let chunk_offset = rhs / Chunk::BITS as u64;
        let bit_offset = rhs % Chunk::BITS as u64;
        self.split_overflowing_shl(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn overflowing_shr_u64(self, rhs: u64) -> (Self, bool) {
        let chunk_offset = rhs / Chunk::BITS as u64;
        let bit_offset = rhs % Chunk::BITS as u64;
        self.split_overflowing_shr(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn split_wrapping_shl(self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        self.split_overflowing_shl(chunk_offset, bit_offset).0
    }
    pub fn split_wrapping_shr(self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        self.split_overflowing_shr(chunk_offset, bit_offset).0
    }
    pub fn wrapping_shl_u64(self, rhs: u64) -> Self {
        self.overflowing_shl_u64(rhs).0
    }
    pub fn wrapping_shr_u64(self, rhs: u64) -> Self {
        self.overflowing_shr_u64(rhs).0
    }
    pub fn split_checked_shl(self, chunk_offset: usize, bit_offset: ChunkW) -> Option<Self> {
        let result = self.split_overflowing_shl(chunk_offset, bit_offset);
        result.1.then_some(result.0)
    }
    pub fn split_checked_shr(self, chunk_offset: usize, bit_offset: ChunkW) -> Option<Self> {
        let result = self.split_overflowing_shr(chunk_offset, bit_offset);
        result.1.then_some(result.0)
    }
    pub fn checked_shl_u64(self, rhs: u64) -> Option<Self> {
        let result = self.overflowing_shl_u64(rhs);
        result.1.then_some(result.0)
    }
    pub fn checked_shr_u64(self, rhs: u64) -> Option<Self> {
        let result = self.overflowing_shr_u64(rhs);
        result.1.then_some(result.0)
    }
    pub fn split_rotate_left_in_place(&mut self, chunk_offset: usize, bit_offset: ChunkW) {
        assert!(bit_offset < Chunk::BITS as ChunkW);
        split_rotate_left_chunks(&mut self.chunks, chunk_offset, bit_offset);
    }
    pub fn split_rotate_right_in_place(&mut self, chunk_offset: usize, bit_offset: ChunkW) {
        assert!(bit_offset < Chunk::BITS as ChunkW);
        split_rotate_right_chunks(&mut self.chunks, chunk_offset, bit_offset);
    }
    pub fn rotate_left_u64_in_place(&mut self, n: u64) {
        let (chunk_offset, bit_offset) = (n / Chunk::BITS as u64, n % Chunk::BITS as u64);
        self.split_rotate_left_in_place(chunk_offset as usize, bit_offset as ChunkW);
    }
    pub fn rotate_right_u64_in_place(&mut self, n: u64) {
        let (chunk_offset, bit_offset) = (n / Chunk::BITS as u64, n % Chunk::BITS as u64);
        self.split_rotate_right_in_place(chunk_offset as usize, bit_offset as ChunkW);
    }
    pub fn split_rotate_left(mut self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        self.split_rotate_left_in_place(chunk_offset, bit_offset);
        self
    }
    pub fn split_rotate_right(mut self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        self.split_rotate_right_in_place(chunk_offset, bit_offset);
        self
    }
    pub fn rotate_left_u64(self, n: u64) -> Self {
        let chunk_offset = n / Chunk::BITS as u64;
        let bit_offset = n % Chunk::BITS as u64;
        self.split_rotate_left(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn rotate_right_u64(self, n: u64) -> Self {
        let chunk_offset = n / Chunk::BITS as u64;
        let bit_offset = n % Chunk::BITS as u64;
        self.split_rotate_right(chunk_offset as usize, bit_offset as ChunkW)
    }
    pub fn swap_bytes_in_place(&mut self) {
        self.chunks.reverse();
    }
    pub fn swap_bits_in_place(&mut self) {
        self.chunks.reverse();
        for chunk in &mut self.chunks {
            *chunk = chunk.reverse_bits();
        }
    }
    pub fn swap_bytes(mut self) -> Self {
        self.swap_bytes_in_place();
        self
    }
    pub fn swap_bits(mut self) -> Self {
        self.swap_bits_in_place();
        self
    }
    pub fn overflowing_add_unsigned(self, rhs: U<W>) -> (Self, bool) {
        let rhs = rhs.reinterpret_signed();
        let negative = rhs < Self::ZERO;
        let (result, overflow) = self.overflowing_add(rhs);
        (result, overflow ^ negative)
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

impl<const W: usize> Add for I<W> {
    type Output = Self;
    #[cfg(debug_assertions)]
    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs).expect("attempt to add with overflow")
    }
    #[cfg(not(debug_assertions))]
    fn add(self, rhs: Self) -> Self {
        self.overflowing_add(rhs).0
    }
}

impl<const W: usize> AddAssign for I<W> {
    #[cfg(debug_assertions)]
    fn add_assign(&mut self, rhs: Self) {
        assert!(
            self.overflowing_add_in_place(rhs),
            "attempt to add with overflow"
        );
    }
    #[cfg(not(debug_assertions))]
    fn add_assign(&mut self, rhs: Self) {
        self.wrapping_add_in_place(rhs);
    }
}

impl<const W: usize> Shl<u64> for I<W> {
    type Output = Self;
    #[cfg(debug_assertions)]
    fn shl(self, rhs: u64) -> Self {
        self.checked_shl_u64(rhs)
            .expect("attempt to shift left with overflow")
    }
    #[cfg(not(debug_assertions))]
    fn shl(self, rhs: u64) -> Self {
        self.wrapping_shl_u64(rhs)
    }
}

impl<const W: usize> Shr<u64> for I<W> {
    type Output = Self;
    #[cfg(debug_assertions)]
    fn shr(self, rhs: u64) -> Self {
        self.checked_shr_u64(rhs)
            .expect("attempt to shift right with overflow")
    }
    #[cfg(not(debug_assertions))]
    fn shr(self, rhs: u64) -> Self {
        self.wrapping_shr_u64(rhs)
    }
}

impl<const W: usize> ShlAssign<u64> for I<W> {
    #[cfg(debug_assertions)]
    fn shl_assign(&mut self, rhs: u64) {
        assert!(
            self.overflowing_shl_u64_in_place(rhs),
            "attempt to shift right with overflow"
        );
    }
    #[cfg(not(debug_assertions))]
    fn shl_assign(&mut self, rhs: u64) {
        self.wrapping_shr_u64_in_place(rhs);
    }
}

impl<const W: usize> ShrAssign<u64> for I<W> {
    #[cfg(debug_assertions)]
    fn shr_assign(&mut self, rhs: u64) {
        assert!(
            self.overflowing_shr_u64_in_place(rhs),
            "attempt to shift right with overflow"
        );
    }
    #[cfg(not(debug_assertions))]
    fn shr_assign(&mut self, rhs: u64) {
        self.wrapping_shr_u64_in_place(rhs);
    }
}
