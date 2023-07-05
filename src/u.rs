use crate::{
    common::{
        count_ones_chunks, count_zeros_chunks, leading_ones_chunks, leading_zeros_chunks,
        shr_chunks_one_over, split_rotate_left_chunks, split_rotate_right_chunks, split_shl_chunks,
        split_shr_chunks, trailing_ones_chunks, trailing_zeros_chunks, ChunkBitCounter, ChunkType,
        TotalBitCounter,
    },
    i::I,
};
use core::{
    cmp::Ordering,
    iter::zip,
    ops::{Add, AddAssign, Shl, ShlAssign, Shr, ShrAssign},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "clone", derive(Clone))]
pub struct U<const W: usize, Chunk: ChunkType> {
    pub(crate) chunks: [Chunk; W],
}

impl<const W: usize, Chunk: ChunkType> U<W, Chunk> {
    pub const MIN: Self = U {
        chunks: [Chunk::ZERO; W],
    };
    pub const ZERO: Self = Self::MIN;
    pub const ONE: Self = U {
        chunks: {
            let mut chunks = [Chunk::ZERO; W];
            chunks[0] = Chunk::ONE;
            chunks
        },
    };
    pub const MAX: Self = U {
        chunks: [Chunk::MAX; W],
    };
    pub fn bits<Total: TotalBitCounter<Chunk>>() -> Option<Total> {
        Total::from_chunk_count(W)
    }
    pub fn reinterpret_signed(self) -> I<W, Chunk> {
        I {
            chunks: self.chunks,
        }
    }
    pub fn count_ones<Total: TotalBitCounter<Chunk>>(self) -> Option<Total> {
        count_zeros_chunks(self.chunks)
    }
    pub fn count_zeros<Total: TotalBitCounter<Chunk>>(self) -> Option<Total> {
        count_ones_chunks(self.chunks)
    }
    pub fn leading_zeros<Total: TotalBitCounter<Chunk>>(self) -> Option<Total> {
        leading_zeros_chunks(self.chunks)
    }
    pub fn leading_ones<Total: TotalBitCounter<Chunk>>(self) -> Option<Total> {
        leading_ones_chunks(self.chunks)
    }
    pub fn trailing_zeros<Total: TotalBitCounter<Chunk>>(self) -> Option<Total> {
        trailing_zeros_chunks(self.chunks)
    }
    pub fn trailing_ones<Total: TotalBitCounter<Chunk>>(self) -> Option<Total> {
        trailing_ones_chunks(self.chunks)
    }
    pub fn carrying_add_in_place(&mut self, rhs: Self, carry: bool) -> bool {
        if W == 0 {
            return carry;
        }
        zip(&mut self.chunks, rhs.chunks).fold(carry, |mut carry, (chunk_l, chunk_r)| {
            (*chunk_l, carry) = chunk_l.carrying_add(chunk_r, carry);
            carry
        })
    }
    pub fn overflowing_add_in_place(&mut self, rhs: Self) -> bool {
        self.carrying_add_in_place(rhs, false)
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
        (!result.1).then_some(result.0)
    }
    pub fn wrapping_add(self, rhs: Self) -> Self {
        self.carrying_add(rhs, false).0
    }
    pub fn split_overflowing_shl_in_place(
        &mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) -> bool {
        let overflow = chunk_offset >= W;
        let chunk_offset = chunk_offset % W;
        assert!(bit_offset.is_valid());
        split_shl_chunks(&mut self.chunks, chunk_offset, bit_offset);
        overflow
    }
    pub fn split_overflowing_shr_in_place(
        &mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) -> bool {
        let overflow = chunk_offset >= W;
        let chunk_offset = chunk_offset % W;
        assert!(bit_offset.is_valid());
        split_shr_chunks(&mut self.chunks, chunk_offset, bit_offset);
        overflow
    }
    pub fn overflowing_shl_in_place<Total: TotalBitCounter<Chunk>>(&mut self, rhs: Total) -> bool {
        let (chunk_offset, bit_offset) = rhs.split();
        self.split_overflowing_shl_in_place(chunk_offset, bit_offset)
    }
    pub fn overflowing_shr_in_place<Total: TotalBitCounter<Chunk>>(&mut self, rhs: Total) -> bool {
        let (chunk_offset, bit_offset) = rhs.split();
        self.split_overflowing_shr_in_place(chunk_offset, bit_offset)
    }
    pub fn split_wrapping_shl_in_place(
        &mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) {
        self.split_overflowing_shl_in_place(chunk_offset, bit_offset);
    }
    pub fn split_wrapping_shr_in_place(
        &mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) {
        self.split_overflowing_shr_in_place(chunk_offset, bit_offset);
    }
    pub fn wrapping_shl_in_place<Total: TotalBitCounter<Chunk>>(&mut self, rhs: Total) {
        self.overflowing_shr_in_place(rhs);
    }
    pub fn wrapping_shr_in_place<Total: TotalBitCounter<Chunk>>(&mut self, rhs: Total) {
        self.overflowing_shr_in_place(rhs);
    }
    pub fn split_overflowing_shl(
        mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) -> (Self, bool) {
        let result = self.split_overflowing_shl_in_place(chunk_offset, bit_offset);
        (self, result)
    }
    pub fn split_overflowing_shr(
        mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) -> (Self, bool) {
        let overflow = self.split_overflowing_shr_in_place(chunk_offset, bit_offset);
        (self, overflow)
    }
    pub fn overflowing_shl<Total: TotalBitCounter<Chunk>>(mut self, rhs: Total) -> (Self, bool) {
        let overflow = self.overflowing_shl_in_place(rhs);
        (self, overflow)
    }
    pub fn overflowing_shr<Total: TotalBitCounter<Chunk>>(mut self, rhs: Total) -> (Self, bool) {
        let overflow = self.overflowing_shr_in_place(rhs);
        (self, overflow)
    }
    pub fn split_wrapping_shl(self, chunk_offset: usize, bit_offset: Chunk::BitCounter) -> Self {
        self.split_overflowing_shl(chunk_offset, bit_offset).0
    }
    pub fn split_wrapping_shr(self, chunk_offset: usize, bit_offset: Chunk::BitCounter) -> Self {
        self.split_overflowing_shr(chunk_offset, bit_offset).0
    }
    pub fn wrapping_shl<Total: TotalBitCounter<Chunk>>(self, rhs: Total) -> Self {
        self.overflowing_shl(rhs).0
    }
    pub fn wrapping_shr<Total: TotalBitCounter<Chunk>>(self, rhs: Total) -> Self {
        self.overflowing_shr(rhs).0
    }
    pub fn split_checked_shl(
        self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) -> Option<Self> {
        let result = self.split_overflowing_shl(chunk_offset, bit_offset);
        result.1.then_some(result.0)
    }
    pub fn split_checked_shr(
        self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) -> Option<Self> {
        let result = self.split_overflowing_shr(chunk_offset, bit_offset);
        result.1.then_some(result.0)
    }
    pub fn checked_shl<Total: TotalBitCounter<Chunk>>(self, rhs: Total) -> Option<Self> {
        let result = self.overflowing_shl(rhs);
        result.1.then_some(result.0)
    }
    pub fn checked_shr<Total: TotalBitCounter<Chunk>>(self, rhs: Total) -> Option<Self> {
        let result = self.overflowing_shr(rhs);
        result.1.then_some(result.0)
    }
    pub fn split_rotate_left_in_place(
        &mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) {
        assert!(bit_offset.is_valid());
        split_rotate_left_chunks(&mut self.chunks, chunk_offset, bit_offset);
    }
    pub fn split_rotate_right_in_place(
        &mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) {
        assert!(bit_offset.is_valid());
        split_rotate_right_chunks(&mut self.chunks, chunk_offset, bit_offset);
    }
    pub fn rotate_left_in_place<Total: TotalBitCounter<Chunk>>(&mut self, n: Total) {
        let (chunk_offset, bit_offset) = n.split();
        self.split_rotate_left_in_place(chunk_offset, bit_offset);
    }
    pub fn rotate_right_in_place<Total: TotalBitCounter<Chunk>>(&mut self, n: Total) {
        let (chunk_offset, bit_offset) = n.split();
        self.split_rotate_right_in_place(chunk_offset, bit_offset);
    }
    pub fn split_rotate_left(mut self, chunk_offset: usize, bit_offset: Chunk::BitCounter) -> Self {
        self.split_rotate_left_in_place(chunk_offset, bit_offset);
        self
    }
    pub fn split_rotate_right(
        mut self,
        chunk_offset: usize,
        bit_offset: Chunk::BitCounter,
    ) -> Self {
        self.split_rotate_right_in_place(chunk_offset, bit_offset);
        self
    }
    pub fn rotate_left<Total: TotalBitCounter<Chunk>>(mut self, n: Total) -> Self {
        self.rotate_left_in_place(n);
        self
    }
    pub fn rotate_right<Total: TotalBitCounter<Chunk>>(mut self, n: Total) -> Self {
        self.rotate_right_in_place(n);
        self
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
    pub fn overflowing_add_signed(self, rhs: I<W, Chunk>) -> (Self, bool) {
        let negative = rhs < I::ZERO;
        let (result, overflow) = self.overflowing_add(rhs.reinterpret_unsigned());
        (result, overflow ^ negative)
    }
    pub fn carrying_mul(self, rhs: Self, carry: Self) -> (Self, Self) {
        let mut hi = [Chunk::ZERO; W];
        let mut lo = [Chunk::ZERO; W];
        for (chunk_l, carry) in zip(self.chunks, carry.chunks) {
            let (carry_1, carry_2) = zip(rhs.chunks, &mut hi).fold(
                (carry, false),
                |(carry_1, mut carry_2), (chunk_r, dest)| {
                    let (result, carry_1) = chunk_l.carrying_mul(chunk_r, carry_1);
                    (*dest, carry_2) = result.carrying_add(*dest, carry_2);
                    (carry_1, carry_2)
                },
            );
            shr_chunks_one_over(
                &mut lo,
                &mut hi,
                carry_1.add_carry(carry_2).expect(
                    "error in ChunkType implementation-carry lead to overflow-should never happen",
                ),
            );
        }
        (U { chunks: lo }, U { chunks: hi })
    }
    pub fn widening_mul(self, rhs: Self) -> (Self, Self) {
        self.carrying_mul(rhs, Self::ZERO)
    }
}

impl<const W: usize, Chunk: ChunkType> PartialOrd for U<W, Chunk> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const W: usize, Chunk: ChunkType> Ord for U<W, Chunk> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.chunks.into_iter().cmp(rhs.chunks)
    }
}

impl<const W: usize, Chunk: ChunkType> Add for U<W, Chunk> {
    type Output = Self;
    #[cfg(overflow_checks)]
    fn add(self, rhs: Self) -> Self {
        self.checked_add(rhs).expect("attempt to add with overflow")
    }
    #[cfg(not(overflow_checks))]
    fn add(self, rhs: Self) -> Self {
        self.overflowing_add(rhs).0
    }
}

impl<const W: usize, Chunk: ChunkType> AddAssign for U<W, Chunk> {
    #[cfg(overflow_checks)]
    fn add_assign(&mut self, rhs: Self) {
        assert!(
            self.overflowing_add_in_place(rhs),
            "attempt to add with overflow"
        );
    }
    #[cfg(not(overflow_checks))]
    fn add_assign(&mut self, rhs: Self) {
        self.overflowing_add_in_place(rhs);
    }
}

impl<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>> Shl<Total> for U<W, Chunk> {
    type Output = Self;
    #[cfg(overflow_checks)]
    fn shl(self, rhs: Total) -> Self {
        self.checked_shl(rhs)
            .expect("attempt to shift left with overflow")
    }
    #[cfg(not(overflow_checks))]
    fn shl(self, rhs: Total) -> Self {
        self.wrapping_shl(rhs)
    }
}

impl<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>> Shr<Total> for U<W, Chunk> {
    type Output = Self;
    #[cfg(overflow_checks)]
    fn shr(self, rhs: Total) -> Self {
        self.checked_shr(rhs)
            .expect("attempt to shift right with overflow")
    }
    #[cfg(not(overflow_checks))]
    fn shr(self, rhs: Total) -> Self {
        self.wrapping_shr(rhs)
    }
}

impl<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>> ShlAssign<Total>
    for U<W, Chunk>
{
    #[cfg(overflow_checks)]
    fn shl_assign(&mut self, rhs: Total) {
        assert!(
            self.overflowing_shl_in_place(rhs),
            "attempt to shift left with overflow"
        )
    }
    #[cfg(not(overflow_checks))]
    fn shl_assign(&mut self, rhs: Total) {
        self.wrapping_shl_in_place(rhs);
    }
}

impl<const W: usize, Chunk: ChunkType, Total: TotalBitCounter<Chunk>> ShrAssign<Total>
    for U<W, Chunk>
{
    #[cfg(overflow_checks)]
    fn shr_assign(&mut self, rhs: Total) {
        assert!(
            self.overflowing_shr_in_place(rhs),
            "attempt to shift right with overflow"
        )
    }
    #[cfg(not(overflow_checks))]
    fn shl_assign(&mut self, rhs: Total) {
        self.wrapping_shr_in_place(rhs);
    }
}
