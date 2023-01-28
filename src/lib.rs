// Copyright 2017 Brian Langenberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Port of bitstream_io for usage with tokio runtime
//! Traits and helpers for async bitstream handling functionality
//!
//! Async Bitstream readers are for reading signed and unsigned integer
//! values from a stream whose sizes may not be whole bytes.
//! Bitstream writers are for writing signed and unsigned integer
//! values to a stream, also potentially un-aligned at a whole byte.
//!
//! Both big-endian and little-endian streams are supported.
//!
//! The only requirement for wrapped reader streams is that they must
//! implement tokio `AsyncRead` trait, and the only requirement
//! for writer streams is that they must implement tokio `AsyncWrite` trait.
//!
//! In addition, reader streams do not consume any more bytes
//! from the underlying reader than necessary, buffering only a
//! single partial byte as needed.
//! Writer streams also write out all whole bytes as they are accumulated.
//!
//! Readers and writers are also designed to work with integer
//! types of any possible size.
//! Many of Rust's built-in integer types are supported by default.
//!
//! Traits are implemented with [macro@async_trait] macro that is re-exported from this library

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::io;
use std::fmt::Debug;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::marker::PhantomData;
use std::mem;
use std::ops::{BitOrAssign, BitXor, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub};

pub use async_trait::async_trait;

pub mod huffman;
pub mod read;
pub mod write;
pub use read::{BitRead, BitReader, ByteRead, ByteReader, HuffmanRead};
pub use write::{
    BitCounter, BitRecorder, BitWrite, BitWriter, ByteWrite, ByteWriter, HuffmanWrite,
};

/// This trait extends many common integer types (both unsigned and signed)
/// with a few trivial methods so that they can be used
/// with the bitstream handling traits.
pub trait Numeric:
    Sized
    + Copy
    + Default
    + Debug
    + PartialOrd
    + Shl<u32, Output = Self>
    + ShlAssign<u32>
    + Shr<u32, Output = Self>
    + ShrAssign<u32>
    + Rem<Self, Output = Self>
    + RemAssign<Self>
    + BitOrAssign<Self>
    + BitXor<Self, Output = Self>
    + Not<Output = Self>
    + Sub<Self, Output = Self>
    + Send
    + Sync
{
    /// The raw byte representation of this numeric type
    type Bytes: AsRef<[u8]> + AsMut<[u8]> + Send + Sync;

    /// Size of type in bits
    const BITS_SIZE: u32;

    /// The value of 1 in this type
    const ONE: Self;

    /// Returns true if this value is 0, in its type
    fn is_zero(self) -> bool;

    /// Returns a `u8` value in this type
    fn from_u8(u: u8) -> Self;

    /// Assuming 0 <= value < 256, returns this value as a `u8` type
    fn to_u8(self) -> u8;

    /// Counts the number of 1 bits
    fn count_ones(self) -> u32;

    /// Counts the number of leading zeros
    fn leading_zeros(self) -> u32;

    /// Counts the number of trailing zeros
    fn trailing_zeros(self) -> u32;

    /// An empty buffer of this type's size
    fn buffer() -> Self::Bytes;

    /// Our value in big-endian bytes
    fn to_be_bytes(self) -> Self::Bytes;

    /// Our value in little-endian bytes
    fn to_le_bytes(self) -> Self::Bytes;

    /// Convert big-endian bytes to our value
    fn from_be_bytes(bytes: Self::Bytes) -> Self;

    /// Convert little-endian bytes to out value
    fn from_le_bytes(bytes: Self::Bytes) -> Self;

    /// Convert to a generic unsigned write value for stream recording purposes
    fn unsigned_value(self) -> write::UnsignedValue;
}

macro_rules! define_numeric {
    ($t:ty) => {
        impl Numeric for $t {
            type Bytes = [u8; mem::size_of::<$t>()];

            const BITS_SIZE: u32 = mem::size_of::<$t>() as u32 * 8;

            const ONE: Self = 1;

            #[inline(always)]
            fn is_zero(self) -> bool {
                self == 0
            }
            #[inline(always)]
            fn from_u8(u: u8) -> Self {
                u as $t
            }
            #[inline(always)]
            fn to_u8(self) -> u8 {
                self as u8
            }
            #[inline(always)]
            fn count_ones(self) -> u32 {
                self.count_ones()
            }
            #[inline(always)]
            fn leading_zeros(self) -> u32 {
                self.leading_zeros()
            }
            #[inline(always)]
            fn trailing_zeros(self) -> u32 {
                self.trailing_zeros()
            }
            #[inline(always)]
            fn buffer() -> Self::Bytes {
                [0; mem::size_of::<$t>()]
            }
            #[inline(always)]
            fn to_be_bytes(self) -> Self::Bytes {
                self.to_be_bytes()
            }
            #[inline(always)]
            fn to_le_bytes(self) -> Self::Bytes {
                self.to_le_bytes()
            }
            #[inline(always)]
            fn from_be_bytes(bytes: Self::Bytes) -> Self {
                <$t>::from_be_bytes(bytes)
            }
            #[inline(always)]
            fn from_le_bytes(bytes: Self::Bytes) -> Self {
                <$t>::from_le_bytes(bytes)
            }
            #[inline(always)]
            fn unsigned_value(self) -> write::UnsignedValue {
                self.into()
            }
        }
    };
}

/// This trait extends many common signed integer types
/// so that they can be used with the bitstream handling traits.
pub trait SignedNumeric: Numeric {
    /// Returns true if this value is negative
    fn is_negative(self) -> bool;

    /// Given a two-complement positive value and certain number of bits,
    /// returns this value as a negative number.
    fn as_negative(self, bits: u32) -> Self;

    /// Given a negative value and a certain number of bits,
    /// returns this value as a twos-complement positive number.
    fn as_unsigned(self, bits: u32) -> Self;

    /// Converts to a generic signed value for stream recording purposes.
    fn signed_value(self) -> write::SignedValue;
}

macro_rules! define_signed_numeric {
    ($t:ty) => {
        impl SignedNumeric for $t {
            #[inline(always)]
            fn is_negative(self) -> bool {
                self < 0
            }
            #[inline(always)]
            fn as_negative(self, bits: u32) -> Self {
                self + (-1 << (bits - 1))
            }
            #[inline(always)]
            fn as_unsigned(self, bits: u32) -> Self {
                self - (-1 << (bits - 1))
            }
            #[inline(always)]
            fn signed_value(self) -> write::SignedValue {
                self.into()
            }
        }
    };
}

define_numeric!(u8);
define_numeric!(i8);
define_numeric!(u16);
define_numeric!(i16);
define_numeric!(u32);
define_numeric!(i32);
define_numeric!(u64);
define_numeric!(i64);
define_numeric!(u128);
define_numeric!(i128);

define_signed_numeric!(i8);
define_signed_numeric!(i16);
define_signed_numeric!(i32);
define_signed_numeric!(i64);
define_signed_numeric!(i128);

/// A stream's endianness, or byte order, for determining
/// how bits should be read.
///
/// It comes in `BigEndian` and `LittleEndian` varieties
/// (which may be shortened to `BE` and `LE`)
/// and is not something programmers should have to implement
/// in most cases.
#[async_trait::async_trait]
pub trait Endianness: Sized + Send + Sync {
    /// Pushes the given bits and value onto an accumulator
    /// with the given bits and value.
    fn push<N>(queue: &mut BitQueue<Self, N>, bits: u32, value: N)
    where
        N: Numeric;

    /// Pops a value with the given number of bits from an accumulator
    /// with the given bits and value.
    fn pop<N>(queue: &mut BitQueue<Self, N>, bits: u32) -> N
    where
        N: Numeric;

    /// Drops the given number of bits from an accumulator
    /// with the given bits and value.
    fn drop<N>(queue: &mut BitQueue<Self, N>, bits: u32)
    where
        N: Numeric;

    /// Returns the next number of 0 bits from an accumulator
    /// with the given bits and value.
    fn next_zeros<N>(queue: &BitQueue<Self, N>) -> u32
    where
        N: Numeric;

    /// Returns the next number of 1 bits from an accumulator
    /// with the given bits and value.
    fn next_ones<N>(queue: &BitQueue<Self, N>) -> u32
    where
        N: Numeric;

    /// Reads signed value from reader in this endianness
    async fn read_signed<R, S>(r: &mut R, bits: u32) -> io::Result<S>
    where
        R: BitRead,
        S: SignedNumeric;

    /// Writes signed value to writer in this endianness
    async fn write_signed<W, S>(w: &mut W, bits: u32, value: S) -> io::Result<()>
    where
        W: BitWrite,
        S: SignedNumeric;

    /// Reads entire numeric value from reader in this endianness
    async fn read_numeric<R, N>(r: R) -> io::Result<N>
    where
        R: AsyncRead + Unpin + Send + Sync,
        N: Numeric;

    /// Writes entire numeric value from reader in this endianness
    async fn write_numeric<W, N>(w: W, value: N) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send + Sync,
        N: Numeric;
}

/// Big-endian, or most significant bits first
#[derive(Copy, Clone)]
pub struct BigEndian;

/// Big-endian, or most significant bits first
pub type BE = BigEndian;

#[async_trait::async_trait]
impl Endianness for BigEndian {
    #[inline]
    fn push<N>(queue: &mut BitQueue<Self, N>, bits: u32, value: N)
    where
        N: Numeric,
    {
        if !queue.value.is_zero() {
            queue.value <<= bits;
        }
        queue.value |= value;
        queue.bits += bits;
    }

    #[inline]
    fn pop<N>(queue: &mut BitQueue<Self, N>, bits: u32) -> N
    where
        N: Numeric,
    {
        if bits < queue.bits {
            let offset = queue.bits - bits;
            let to_return = queue.value >> offset;
            queue.value %= N::ONE << offset;
            queue.bits -= bits;
            to_return
        } else {
            let to_return = queue.value;
            queue.value = N::default();
            queue.bits = 0;
            to_return
        }
    }

    #[inline]
    fn drop<N>(queue: &mut BitQueue<Self, N>, bits: u32)
    where
        N: Numeric,
    {
        if bits < queue.bits {
            queue.value %= N::ONE << (queue.bits - bits);
            queue.bits -= bits;
        } else {
            queue.value = N::default();
            queue.bits = 0;
        }
    }

    #[inline]
    fn next_zeros<N>(queue: &BitQueue<Self, N>) -> u32
    where
        N: Numeric,
    {
        queue.value.leading_zeros() - (N::BITS_SIZE - queue.bits)
    }

    #[inline]
    fn next_ones<N>(queue: &BitQueue<Self, N>) -> u32
    where
        N: Numeric,
    {
        if queue.bits < N::BITS_SIZE {
            (queue.value ^ ((N::ONE << queue.bits) - N::ONE)).leading_zeros()
                - (N::BITS_SIZE - queue.bits)
        } else {
            (!queue.value).leading_zeros()
        }
    }

    async fn read_signed<R, S>(r: &mut R, bits: u32) -> io::Result<S>
    where
        R: BitRead,
        S: SignedNumeric,
    {
        if bits <= S::BITS_SIZE {
            let is_negative = r.read_bit().await?;
            let unsigned = r.read::<S>(bits - 1).await?;
            Ok(if is_negative {
                unsigned.as_negative(bits)
            } else {
                unsigned
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive bits for type read",
            ))
        }
    }

    async fn write_signed<W, S>(w: &mut W, bits: u32, value: S) -> io::Result<()>
    where
        W: BitWrite,
        S: SignedNumeric,
    {
        if bits > S::BITS_SIZE {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive bits for type written",
            ))
        } else if bits == S::BITS_SIZE {
            w.write_bytes(value.to_be_bytes().as_ref()).await
        } else if value.is_negative() {
            // w.write_bit(true).and_then(|()| w.write(bits - 1, value.as_unsigned(bits)))
            w.write_bit(true).await?;
            w.write(bits - 1, value.as_unsigned(bits)).await
        } else {
            // w.write_bit(false).and_then(|()| w.write(bits - 1, value))
            w.write_bit(false).await?;
            w.write(bits - 1, value).await
        }
    }

    #[inline]
    async fn read_numeric<R, N>(mut r: R) -> io::Result<N>
    where
        R: AsyncRead + Unpin + Send + Sync,
        N: Numeric,
    {
        let mut buffer = N::buffer();
        r.read_exact(buffer.as_mut()).await?;
        Ok(N::from_be_bytes(buffer))
    }

    #[inline]
    async fn write_numeric<W, N>(mut w: W, value: N) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send + Sync,
        N: Numeric,
    {
        w.write_all(value.to_be_bytes().as_ref()).await
    }
}

/// Little-endian, or least significant bits first
#[derive(Copy, Clone)]
pub struct LittleEndian;

/// Little-endian, or least significant bits first
pub type LE = LittleEndian;

#[async_trait::async_trait]
impl Endianness for LittleEndian {
    #[inline]
    fn push<N>(queue: &mut BitQueue<Self, N>, bits: u32, mut value: N)
    where
        N: Numeric,
    {
        if !value.is_zero() {
            value <<= queue.bits;
            queue.value |= value;
        }
        queue.bits += bits;
    }

    #[inline]
    fn pop<N>(queue: &mut BitQueue<Self, N>, bits: u32) -> N
    where
        N: Numeric,
    {
        if bits < queue.bits {
            let to_return = queue.value % (N::ONE << bits);
            queue.value >>= bits;
            queue.bits -= bits;
            to_return
        } else {
            let to_return = queue.value;
            queue.value = N::default();
            queue.bits = 0;
            to_return
        }
    }

    #[inline]
    fn drop<N>(queue: &mut BitQueue<Self, N>, bits: u32)
    where
        N: Numeric,
    {
        if bits < queue.bits {
            queue.value >>= bits;
            queue.bits -= bits;
        } else {
            queue.value = N::default();
            queue.bits = 0;
        }
    }

    #[inline(always)]
    fn next_zeros<N>(queue: &BitQueue<Self, N>) -> u32
    where
        N: Numeric,
    {
        queue.value.trailing_zeros()
    }

    #[inline]
    fn next_ones<N>(queue: &BitQueue<Self, N>) -> u32
    where
        N: Numeric,
    {
        (queue.value ^ !N::default()).trailing_zeros()
    }

    async fn read_signed<R, S>(r: &mut R, bits: u32) -> io::Result<S>
    where
        R: BitRead,
        S: SignedNumeric,
    {
        if bits <= S::BITS_SIZE {
            let unsigned = r.read::<S>(bits - 1).await?;
            let is_negative = r.read_bit().await?;
            Ok(if is_negative {
                unsigned.as_negative(bits)
            } else {
                unsigned
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive bits for type read",
            ))
        }
    }

    async fn write_signed<W, S>(w: &mut W, bits: u32, value: S) -> io::Result<()>
    where
        W: BitWrite,
        S: SignedNumeric,
    {
        if bits > S::BITS_SIZE {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive bits for type written",
            ))
        } else if bits == S::BITS_SIZE {
            w.write_bytes(value.to_le_bytes().as_ref()).await
        } else if value.is_negative() {
            // w.write(bits - 1, value.as_unsigned(bits))
            //     .and_then(|()| w.write_bit(true))
            w.write(bits - 1, value.as_unsigned(bits)).await?;
            w.write_bit(true).await
        } else {
            // w.write(bits - 1, value).and_then(|()| w.write_bit(false))
            w.write(bits - 1, value).await?;
            w.write_bit(false).await
        }
    }

    async fn read_numeric<R, N>(mut r: R) -> io::Result<N>
    where
        R: AsyncRead + Unpin + Send + Sync,
        N: Numeric,
    {
        let mut buffer = N::buffer();
        r.read_exact(buffer.as_mut()).await?;
        Ok(N::from_le_bytes(buffer))
    }

    #[inline]
    async fn write_numeric<W, N>(mut w: W, value: N) -> io::Result<()>
    where
        W: AsyncWrite + Unpin + Send + Sync,
        N: Numeric,
    {
        w.write_all(value.to_le_bytes().as_ref()).await
    }
}

/// A queue for efficiently pushing bits onto a value
/// and popping them off a value.
#[derive(Clone, Default)]
pub struct BitQueue<E: Endianness, N: Numeric> {
    phantom: PhantomData<E>,
    value: N,
    bits: u32,
}

impl<E: Endianness, N: Numeric> BitQueue<E, N> {
    /// Returns a new empty queue
    #[inline]
    pub fn new() -> BitQueue<E, N> {
        BitQueue {
            phantom: PhantomData,
            value: N::default(),
            bits: 0,
        }
    }

    /// Creates a new queue from the given value with the given size
    /// Panics if the value is larger than the given number of bits.
    #[inline]
    pub fn from_value(value: N, bits: u32) -> BitQueue<E, N> {
        assert!(if bits < N::BITS_SIZE {
            value < (N::ONE << bits)
        } else {
            bits <= N::BITS_SIZE
        });
        BitQueue {
            phantom: PhantomData,
            value,
            bits,
        }
    }

    /// Sets the queue to a given value with the given number of bits
    /// Panics if the value is larger than the given number of bits
    #[inline]
    pub fn set(&mut self, value: N, bits: u32) {
        assert!(if bits < N::BITS_SIZE {
            value < (N::ONE << bits)
        } else {
            bits <= N::BITS_SIZE
        });
        self.value = value;
        self.bits = bits;
    }

    /// Consumes the queue and returns its current value
    #[inline(always)]
    pub fn value(self) -> N {
        self.value
    }

    /// Returns the total bits in the queue
    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.bits
    }

    /// Returns the maximum bits the queue can hold
    #[inline(always)]
    pub fn max_len(&self) -> u32 {
        N::BITS_SIZE
    }

    /// Returns the remaining bits the queue can hold
    #[inline(always)]
    pub fn remaining_len(&self) -> u32 {
        self.max_len() - self.len()
    }

    /// Returns true if the queue is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Returns true if the queue is full
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.bits == N::BITS_SIZE
    }

    /// Drops all values in the queue
    #[inline(always)]
    pub fn clear(&mut self) {
        self.set(N::default(), 0)
    }

    /// Returns true if all bits remaining in the queue are 0
    #[inline(always)]
    pub fn all_0(&self) -> bool {
        self.value.count_ones() == 0
    }

    /// Returns true if all bits remaining in the queue are 1
    #[inline(always)]
    pub fn all_1(&self) -> bool {
        self.value.count_ones() == self.bits
    }

    /// Pushes a value with the given number of bits onto the tail of the queue
    /// Panics if the number of bits pushed is larger than the queue can hold.
    #[inline(always)]
    pub fn push(&mut self, bits: u32, value: N) {
        assert!(bits <= self.remaining_len()); // check for overflow
        E::push(self, bits, value)
    }

    /// Pops a value with the given number of bits from the head of the queue
    /// Panics if the number of bits popped is larger than the number
    /// of bits in the queue.
    #[inline(always)]
    pub fn pop(&mut self, bits: u32) -> N {
        assert!(bits <= self.len()); // check for underflow
        E::pop(self, bits)
    }

    /// Pops all the current bits from the queue
    /// and resets it to an empty state.
    #[inline]
    pub fn pop_all(&mut self) -> N {
        let to_return = self.value;
        self.value = N::default();
        self.bits = 0;
        to_return
    }

    /// Drops the given number of bits from the head of the queue
    /// without returning them.
    /// Panics if the number of bits dropped is larger than the
    /// number of bits in the queue.
    #[inline(always)]
    pub fn drop(&mut self, bits: u32) {
        assert!(bits <= self.len()); // check for underflow
        E::drop(self, bits)
    }

    /// Pops all 0 bits up to and including the next 1 bit
    /// and returns the amount of 0 bits popped
    #[inline]
    pub fn pop_0(&mut self) -> u32 {
        let zeros = E::next_zeros(self);
        self.drop(zeros + 1);
        zeros
    }

    /// Pops all 1 bits up to and including the next 0 bit
    /// and returns the amount of 1 bits popped
    #[inline]
    pub fn pop_1(&mut self) -> u32 {
        let ones = E::next_ones(self);
        self.drop(ones + 1);
        ones
    }
}

impl<E: Endianness> BitQueue<E, u8> {
    /// Returns the state of the queue as a single value
    /// which can be used to perform lookups.
    #[inline(always)]
    pub fn to_state(&self) -> usize {
        (1 << self.bits) | (self.value as usize)
    }
}
