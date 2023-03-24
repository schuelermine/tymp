#![no_std]
#![feature(get_many_mut)]
#![forbid(unsafe_code)]

use cfg_if::cfg_if;
use core::{fmt::Display, ops::ControlFlow};
use itertools::izip;

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

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct I<const W: usize> {
    chunks: [Chunk; W],
}

impl<const W: usize> I<W> {
    pub const BITS: Option<usize> = W.checked_mul(8);
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
    pub fn count_ones_u64(self) -> u64 {
        self.chunks.into_iter().fold(0, |count, chunk| {
            count
                .checked_add(chunk.count_ones() as u64)
                .expect("positive overflow")
        })
    }
    pub fn count_zeros_u64(self) -> u64 {
        self.chunks.into_iter().fold(0, |count, chunk| {
            count
                .checked_add(chunk.count_ones() as u64)
                .expect("positive overflow")
        })
    }
    pub fn leading_zeros_u64(self) -> u64 {
        get_control_flow(self.chunks.into_iter().try_rfold(0, |count: u64, chunk| {
            let all_zeros = chunk == 0;
            let chunk_leading_zeros = match all_zeros {
                true => Chunk::BITS,
                false => chunk.leading_zeros(),
            } as u64;
            let count = count
                .checked_add(chunk_leading_zeros)
                .expect("positive overflow");
            match all_zeros {
                true => ControlFlow::Continue(count),
                false => ControlFlow::Break(count),
            }
        }))
    }
    pub fn leading_ones_u64(self) -> u64 {
        get_control_flow(self.chunks.into_iter().try_rfold(0, |acc: u64, chunk| {
            let all_ones = chunk == Chunk::MAX;
            let chunk_leading_ones = match all_ones {
                true => Chunk::BITS,
                false => chunk.leading_ones(),
            } as u64;
            let result = acc
                .checked_add(chunk_leading_ones)
                .expect("positive overflow");
            match all_ones {
                true => ControlFlow::Continue(result),
                false => ControlFlow::Break(result),
            }
        }))
    }
    pub fn trailing_zeros_u64(self) -> u64 {
        get_control_flow(self.chunks.into_iter().try_rfold(0, |acc: u64, chunk| {
            let all_zeros = chunk == 0;
            let chunk_trailing_zeros = match all_zeros {
                true => Chunk::BITS,
                false => chunk.trailing_zeros(),
            } as u64;
            let result = acc
                .checked_add(chunk_trailing_zeros)
                .expect("positive overflow");
            match all_zeros {
                true => ControlFlow::Continue(result),
                false => ControlFlow::Break(result),
            }
        }))
    }
    pub fn trailing_ones_u64(self) -> u64 {
        get_control_flow(self.chunks.into_iter().try_rfold(0, |acc: u64, chunk| {
            let all_ones = chunk == Chunk::MAX;
            let chunk_trailing_ones = match all_ones {
                true => Chunk::BITS,
                false => chunk.trailing_ones(),
            } as u64;
            let result = acc
                .checked_add(chunk_trailing_ones)
                .expect("positive overflow");
            break_if(!all_ones, result)
        }))
    }
    pub fn carrying_add(self, rhs: Self, mut carry: bool) -> (Self, bool) {
        if W == 0 {
            return (I { chunks: [0; W] }, carry);
        }
        let mut chunks = [0; W];
        let mut iter = izip!(self.chunks, rhs.chunks, &mut chunks);
        let (last_chunk_l, last_chunk_r, last_dest) = iter.next_back().unwrap();
        carry = iter.fold(carry, |mut carry, (chunk_l, chunk_r, dest)| {
            (*dest, carry) = carrying_add_chunk(chunk_l, chunk_r, carry);
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
        assert!(bit_offset < Chunk::BITS as u8);
        if chunk_offset == 0 {
            self.chunks.iter_mut().fold(0, |mut infill, chunk| {
                (*chunk, infill) = shl_chunk_full(*chunk, bit_offset, infill);
                infill
            });
            self
        } else {
            let mut infill = 0;
            for i in 0..W - chunk_offset {
                let [src, dest] = self
                    .chunks
                    .get_many_mut([i, i + chunk_offset])
                    .ok()
                    .unwrap();
                (*dest, infill) = shl_chunk_full(*src, bit_offset, infill);
            }
            self.chunks[0..chunk_offset].fill(0);
            self
        }
    }
    pub fn split_rotate_left(mut self, chunk_offset: usize, bit_offset: ChunkW) -> Self {
        self.chunks.rotate_right(chunk_offset);
        let infill = self.chunks.iter_mut().fold(0, |mut infill, chunk| {
            (*chunk, infill) = shl_chunk_full(*chunk, bit_offset, infill);
            infill
        });
        self.chunks[0] |= infill;
        self
    }
}

fn get_control_flow<T>(control_flow: ControlFlow<T, T>) -> T {
    match control_flow {
        ControlFlow::Continue(x) => x,
        ControlFlow::Break(x) => x,
    }
}

fn break_if<T>(do_break: bool, value: T) -> ControlFlow<T, T> {
    match do_break {
        true => ControlFlow::Break(value),
        false => ControlFlow::Continue(value),
    }
}

fn carrying_add_chunk(lhs: Chunk, rhs: Chunk, carry: bool) -> (Chunk, bool) {
    let (a, b) = lhs.overflowing_add(rhs);
    let (c, d) = a.overflowing_add(carry as Chunk);
    (c, b || d)
}

fn carrying_add_chunk_as_signed(lhs: Chunk, rhs: Chunk, carry: bool) -> (Chunk, bool) {
    let (a, b) = (lhs as IChunk).overflowing_add(rhs as IChunk);
    let (c, d) = a.overflowing_add(carry as IChunk);
    (c as Chunk, b != d)
}

fn shl_chunk_full(value: Chunk, shamt: ChunkW, infill: Chunk) -> (Chunk, Chunk) {
    if shamt == 0 {
        (value | infill, 0)
    } else {
        (
            value << shamt | infill,
            (chunk_mask(0, shamt) & value) >> (Chunk::BITS as ChunkW - shamt),
        )
    }
}

fn chunk_mask(zeros_l: ChunkW, zeros_r: ChunkW) -> Chunk {
    Chunk::MAX << zeros_r & Chunk::MAX >> zeros_l
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseIntError {
    kind: IntErrorKind,
}

impl ParseIntError {
    pub fn kind(&self) -> &IntErrorKind {
        &self.kind
    }
}

impl Display for ParseIntError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.kind.description().fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntErrorKind {
    Empty,
    InvalidDigit,
    PosOverflow,
    NegOverflow,
    Zero,
}

impl IntErrorKind {
    fn description(&self) -> &str {
        match self {
            IntErrorKind::Empty => "cannot parse integer from empty string",
            IntErrorKind::InvalidDigit => "invalid digit found in string",
            IntErrorKind::PosOverflow => "number too large to fit in target type",
            IntErrorKind::NegOverflow => "number too small to fit in target type",
            IntErrorKind::Zero => "number would be zero for non-zero type",
        }
    }
}
