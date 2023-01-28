// Copyright 2017 Brian Langenberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Traits and implementations for reading bits from a stream.
//!
//! ## Example
//!
//! Reading the initial STREAMINFO block from a FLAC file,
//! as documented in its
//! [specification](https://xiph.org/flac/format.html#stream).
//!
//! ```
//! use std::io::{Cursor};
//! use tokio::io::{AsyncRead, AsyncReadExt};
//! use tokio_bitstream_io::{BigEndian, BitReader, BitRead, ByteReader, ByteRead, LittleEndian};
//! 
//! let flac: Vec<u8> = vec![0x66,0x4c,0x61,0x43,0x00,0x00,0x00,0x22,
//!                          0x10,0x00,0x10,0x00,0x00,0x06,0x06,0x00,
//!                          0x21,0x62,0x0a,0xc4,0x42,0xf0,0x00,0x04,
//!                          0xa6,0xcc,0xfa,0xf2,0x69,0x2f,0xfd,0xec,
//!                          0x2d,0x5b,0x30,0x01,0x76,0xb4,0x62,0x88,
//!                          0x7d,0x92,0x04,0x00,0x00,0x7a,0x20,0x00,
//!                          0x00,0x00,0x72,0x65,0x66,0x65,0x72,0x65,
//!                          0x6e,0x63,0x65,0x20,0x6c,0x69,0x62,0x46,
//!                          0x4c,0x41,0x43,0x20,0x31,0x2e,0x31,0x2e,
//!                          0x34,0x20,0x32,0x30,0x30,0x37,0x30,0x32,
//!                          0x31,0x33,0x04,0x00,0x00,0x00,0x16,0x00,
//!                          0x00,0x00,0x74,0x69,0x74,0x6c,0x65,0x3d,
//!                          0x32,0x63,0x68,0x20,0x34,0x34,0x31,0x30,
//!                          0x30,0x20,0x20,0x31,0x36,0x62,0x69,0x74,
//!                          0x10,0x00,0x00,0x00,0x61,0x6c,0x62,0x75,
//!                          0x6d,0x3d,0x54,0x65,0x73,0x74,0x20,0x41,
//!                          0x6c,0x62,0x75,0x6d,0x0f,0x00,0x00,0x00,
//!                          0x61,0x72,0x74,0x69,0x73,0x74,0x3d,0x41,
//!                          0x73,0x73,0x6f,0x72,0x74,0x65,0x64,0x0d,
//!                          0x00,0x00,0x00,0x74,0x72,0x61,0x63,0x6b,
//!                          0x6e,0x75,0x6d,0x62,0x65,0x72,0x3d,0x31];
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct BlockHeader {
//!     last_block: bool,
//!     block_type: u8,
//!     block_size: u32,
//! }
//!
//! impl BlockHeader {
//!     async fn read<R: AsyncRead + Send + Sync + Unpin>(r: &mut BitReader<R, BigEndian>) -> std::io::Result<Self> {
//!         Ok(Self {
//!             last_block: r.read_bit().await?,
//!             block_type: r.read(7).await?,
//!             block_size: r.read(24).await?,
//!         })
//!     }
//! }
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct Streaminfo {
//!     minimum_block_size: u16,
//!     maximum_block_size: u16,
//!     minimum_frame_size: u32,
//!     maximum_frame_size: u32,
//!     sample_rate: u32,
//!     channels: u8,
//!     bits_per_sample: u8,
//!     total_samples: u64,
//!     md5: [u8; 16],
//! }
//!
//! impl Streaminfo {
//!     async fn read<R: AsyncRead + Send + Sync + Unpin>(r: &mut BitReader<R, BigEndian>) -> std::io::Result<Self> {
//!         Ok(Self {
//!             minimum_block_size: r.read(16).await?,
//!             maximum_block_size: r.read(16).await?,
//!             minimum_frame_size: r.read(24).await?,
//!             maximum_frame_size: r.read(24).await?,
//!             sample_rate: r.read(20).await?,
//!             channels: r.read::<u8>(3).await? + 1,
//!             bits_per_sample: r.read::<u8>(5).await? + 1,
//!             total_samples: r.read(36).await?,
//!             md5: r.read_to_bytes().await?,
//!         })
//!     }
//! }
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct VorbisComment {
//!     vendor: String,
//!     comment: Vec<String>,
//! }
//!
//! impl VorbisComment {
//!    async fn read<R: AsyncRead + Send + Sync + Unpin>(
//!        r: &mut ByteReader<R, LittleEndian>,
//!    ) -> Result<Self, Box<dyn std::error::Error>> {
//!
//!        async fn read_entry<R: AsyncRead + Send + Sync + Unpin>(
//!            r: &mut ByteReader<R, LittleEndian>,
//!        ) -> Result<String, Box<dyn std::error::Error>> {
//!            use std::convert::TryInto;
//!            let size = r.read::<u32>().await?.try_into()?;
//!            Ok(String::from_utf8(r.read_to_vec(size).await?)?)
//!        }
//!         
//!        let vendor = read_entry(r).await?;  
//!        let mut comment = vec![];
//!        
//!        for n in 0..r.read::<u32>().await? {
//!           comment.push(read_entry(r).await?);
//!        }
//!     
//!        Ok(Self { vendor, comment })
//!    }
//! }
//!
//! let mut cursor = Cursor::new(&flac);
//!
//! let mut reader = BitReader::endian(&mut cursor, BigEndian);
//!
//! # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
//! // stream marker
//! assert_eq!(&reader.read_to_bytes().await.unwrap(), b"fLaC");
//!
//! // metadata block header
//! assert_eq!(
//!     BlockHeader::read(&mut reader).await.unwrap(),
//!     BlockHeader { last_block: false, block_type: 0, block_size: 34 }
//! );
//!
//! // STREAMINFO block
//! assert_eq!(
//!     Streaminfo::read(&mut reader).await.unwrap(),
//!     Streaminfo {
//!         minimum_block_size: 4096,
//!         maximum_block_size: 4096,
//!         minimum_frame_size: 1542,
//!         maximum_frame_size: 8546,
//!         sample_rate: 44100,
//!         channels: 2,
//!         bits_per_sample: 16,
//!         total_samples: 304844,
//!         md5: *b"\xFA\xF2\x69\x2F\xFD\xEC\x2D\x5B\x30\x01\x76\xB4\x62\x88\x7D\x92",
//!     }
//! );
//!
//! // metadata block header
//! assert_eq!(
//!     BlockHeader::read(&mut reader).await.unwrap(),
//!     BlockHeader { last_block: false, block_type: 4, block_size: 122 }
//! );
//!
//! // VORBIS_COMMENT block (little endian)
//! assert_eq!(
//!    VorbisComment::read(&mut ByteReader::new(reader.reader().unwrap())).await.unwrap(),
//!    VorbisComment {
//!        vendor: "reference libFLAC 1.1.4 20070213".to_string(),
//!        comment: vec![
//!            "title=2ch 44100  16bit".to_string(),
//!            "album=Test Album".to_string(),
//!            "artist=Assorted".to_string(),
//!            "tracknumber=1".to_string(),
//!        ],
//!    }
//! );
//! # });

#![warn(missing_docs)]

use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};

use super::{huffman::ReadHuffmanTree, BitQueue, Endianness, Numeric, PhantomData, SignedNumeric};

/// A trait for anything that can read a variable number of
/// potentially un-aligned values from an input stream
#[async_trait::async_trait]
pub trait BitRead: Send {
    /// Reads a single bit from the stream.
    /// `true` indicates 1, `false` indicates 0
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_bit(&mut self) -> io::Result<bool>;

    /// Reads an unsigned value from the stream with
    /// the given number of bits.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    /// Also returns an error if the output type is too small
    /// to hold the requested number of bits.
    async fn read<U>(&mut self, bits: u32) -> io::Result<U>
    where
        U: Numeric;

    /// Reads a twos-complement signed value from the stream with
    /// the given number of bits.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    /// Also returns an error if the output type is too small
    /// to hold the requested number of bits.
    async fn read_signed<S>(&mut self, bits: u32) -> io::Result<S>
    where
        S: SignedNumeric;

    /// Skips the given number of bits in the stream.
    /// Since this method does not need an accumulator,
    /// it may be slightly faster than reading to an empty variable.
    /// In addition, since there is no accumulator,
    /// there is no upper limit on the number of bits
    /// which may be skipped.
    /// These bits are still read from the stream, however,
    /// and are never skipped via a `seek` method.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn skip(&mut self, bits: u32) -> io::Result<()>;

    /// Completely fills the given buffer with whole bytes.
    /// If the stream is already byte-aligned, it will map
    /// to a faster `read_exact` call.  Otherwise it will read
    /// bytes individually in 8-bit increments.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        for b in buf.iter_mut() {
            *b = self.read(8).await?;
        }
        Ok(())
    }

    /// Completely fills a whole buffer with bytes and returns it.
    /// If the stream is already byte-aligned, it will map
    /// to a faster `read_exact` call.  Otherwise it will read
    /// bytes individually in 8-bit increments.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_to_bytes<const SIZE: usize>(&mut self) -> io::Result<[u8; SIZE]> {
        let mut buf = [0; SIZE];
        self.read_bytes(&mut buf).await?;
        Ok(buf)
    }

    /// Completely fills a vector of bytes and returns it.
    /// If the stream is already byte-aligned, it will map
    /// to a faster `read_exact` call.  Otherwise it will read
    /// bytes individually in 8-bit increments.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_to_vec(&mut self, bytes: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; bytes];
        self.read_bytes(&mut buf).await?;
        Ok(buf)
    }

    /// Counts the number of 1 bits in the stream until the next
    /// 0 bit and returns the amount read.
    /// Because this field is variably-sized and may be large,
    /// its output is always a `u32` type.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_unary0(&mut self) -> io::Result<u32> {
        let mut unary = 0;
        while self.read_bit().await? {
            unary += 1;
        }
        Ok(unary)
    }

    /// Counts the number of 0 bits in the stream until the next
    /// 1 bit and returns the amount read.
    /// Because this field is variably-sized and may be large,
    /// its output is always a `u32` type.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_unary1(&mut self) -> io::Result<u32> {
        let mut unary = 0;
        while !(self.read_bit().await?) {
            unary += 1;
        }
        Ok(unary)
    }

    /// Returns true if the stream is aligned at a whole byte.
    fn byte_aligned(&self) -> bool;

    /// Throws away all unread bit values until the next whole byte.
    /// Does nothing if the stream is already aligned.
    fn byte_align(&mut self);
}

/// A trait for anything that can read Huffman codes
/// of a given endianness from an input stream
#[async_trait::async_trait]
pub trait HuffmanRead<E: Endianness> {
    /// Given a compiled Huffman tree, reads bits from the stream
    /// until the next symbol is encountered.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_huffman<T>(&mut self, tree: &[ReadHuffmanTree<E, T>]) -> io::Result<T>
    where
        T: Clone + Send + Sync;
}

/// For reading non-aligned bits from a stream of bytes in a given endianness.
///
/// This will read exactly as many whole bytes needed to return
/// the requested number of bits.  It may cache up to a single partial byte
/// but no more.
#[derive(Clone)]
pub struct BitReader<R: AsyncRead + Unpin + Send + Sync, E: Endianness> {
    reader: R,
    bitqueue: BitQueue<E, u8>,
}

impl<R: AsyncRead + Unpin + Send + Sync, E: Endianness> BitReader<R, E> {
    /// Wraps a BitReader around something that implements `Read`
    pub fn new(reader: R) -> BitReader<R, E> {
        BitReader {
            reader,
            bitqueue: BitQueue::new(),
        }
    }

    /// Wraps a BitReader around something that implements `Read`
    /// with the given endianness.
    pub fn endian(reader: R, _endian: E) -> BitReader<R, E> {
        BitReader {
            reader,
            bitqueue: BitQueue::new(),
        }
    }

    /// Unwraps internal reader and disposes of BitReader.
    ///
    /// # Warning
    ///
    /// Any unread partial bits are discarded.
    #[inline]
    pub fn into_reader(self) -> R {
        self.reader
    }

    /// If stream is byte-aligned, provides mutable reference
    /// to internal reader.  Otherwise returns `None`
    #[inline]
    pub fn reader(&mut self) -> Option<&mut R> {
        if self.byte_aligned() {
            Some(&mut self.reader)
        } else {
            None
        }
    }

    /// Converts `BitReader` to `ByteReader` in the same endianness.
    ///
    /// # Warning
    ///
    /// Any unread partial bits are discarded.
    #[inline]
    pub fn into_bytereader(self) -> ByteReader<R, E> {
        ByteReader::new(self.into_reader())
    }

    /// If stream is byte-aligned, provides temporary `ByteReader`
    /// in the same endianness.  Otherwise returns `None`
    ///
    /// # Warning
    ///
    /// Any reader bits left over when `ByteReader` is dropped are lost.
    #[inline]
    pub fn bytereader(&mut self) -> Option<ByteReader<&mut R, E>> {
        self.reader().map(ByteReader::new)
    }

    /// Consumes reader and returns any un-read partial byte
    /// as a `(bits, value)` tuple.
    ///
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0b1010_0101, 0b0101_1010];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read::<u16>(9).await.unwrap(), 0b1010_0101_0);
    /// let (bits, value) = reader.into_unread();
    /// assert_eq!(bits, 7);
    /// assert_eq!(value, 0b101_1010);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0b1010_0101, 0b0101_1010];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read::<u16>(8).await.unwrap(), 0b1010_0101);
    /// let (bits, value) = reader.into_unread();
    /// assert_eq!(bits, 0);
    /// assert_eq!(value, 0);
    /// # });
    /// ```
    #[inline]
    pub fn into_unread(self) -> (u32, u8) {
        (self.bitqueue.len(), self.bitqueue.value())
    }
}

#[async_trait::async_trait]
impl<R: AsyncRead + Unpin + Send + Sync, E: Endianness> BitRead for BitReader<R, E> {
    /// # Examples
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), false);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), false);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{LittleEndian, BitReader, BitRead};
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), LittleEndian);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), false);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// assert_eq!(reader.read_bit().await.unwrap(), false);
    /// assert_eq!(reader.read_bit().await.unwrap(), true);
    /// # });
    /// ```
    #[inline(always)]
    async fn read_bit(&mut self) -> io::Result<bool> {
        if self.bitqueue.is_empty() {
            self.bitqueue.set(read_byte(&mut self.reader).await?, 8);
        }
        Ok(self.bitqueue.pop(1) == 1)
    }

    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read::<u8>(1).await.unwrap(), 0b1);
    /// assert_eq!(reader.read::<u8>(2).await.unwrap(), 0b01);
    /// assert_eq!(reader.read::<u8>(5).await.unwrap(), 0b10111);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{LittleEndian, BitReader, BitRead};
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), LittleEndian);
    /// assert_eq!(reader.read::<u8>(1).await.unwrap(), 0b1);
    /// assert_eq!(reader.read::<u8>(2).await.unwrap(), 0b11);
    /// assert_eq!(reader.read::<u8>(5).await.unwrap(), 0b10110);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0;10];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert!(reader.read::<u8>(9).await.is_err());    // can't read  9 bits to u8
    /// assert!(reader.read::<u16>(17).await.is_err());  // can't read 17 bits to u16
    /// assert!(reader.read::<u32>(33).await.is_err());  // can't read 33 bits to u32
    /// assert!(reader.read::<u64>(65).await.is_err());  // can't read 65 bits to u64
    /// # });
    /// ```
    async fn read<U>(&mut self, mut bits: u32) -> io::Result<U>
    where
        U: Numeric,
    {
        if bits <= U::BITS_SIZE {
            let bitqueue_len = self.bitqueue.len();
            if bits <= bitqueue_len {
                Ok(U::from_u8(self.bitqueue.pop(bits)))
            } else {
                let mut acc =
                    BitQueue::from_value(U::from_u8(self.bitqueue.pop_all()), bitqueue_len);
                bits -= bitqueue_len;

                read_aligned(&mut self.reader, bits / 8, &mut acc).await?;
                read_unaligned(&mut self.reader, bits % 8, &mut acc, &mut self.bitqueue).await?;
                Ok(acc.value())
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive bits for type read",
            ))
        }
    }

    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read_signed::<i8>(4).await.unwrap(), -5);
    /// assert_eq!(reader.read_signed::<i8>(4).await.unwrap(), 7);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{LittleEndian, BitReader, BitRead};
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), LittleEndian);
    /// assert_eq!(reader.read_signed::<i8>(4).await.unwrap(), 7);
    /// assert_eq!(reader.read_signed::<i8>(4).await.unwrap(), -5);
    /// # }); 
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0;10];
    /// let mut r = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert!(r.read_signed::<i8>(9).await.is_err());   // can't read 9 bits to i8
    /// assert!(r.read_signed::<i16>(17).await.is_err()); // can't read 17 bits to i16
    /// assert!(r.read_signed::<i32>(33).await.is_err()); // can't read 33 bits to i32
    /// assert!(r.read_signed::<i64>(65).await.is_err()); // can't read 65 bits to i64
    /// # });
    /// ```
    #[inline]
    async fn read_signed<S>(&mut self, bits: u32) -> io::Result<S>
    where
        S: SignedNumeric,
    {
        E::read_signed(self, bits).await
    }

    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert!(reader.skip(3).await.is_ok());
    /// assert_eq!(reader.read::<u8>(5).await.unwrap(), 0b10111);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{LittleEndian, BitReader, BitRead};
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), LittleEndian);
    /// assert!(reader.skip(3).await.is_ok());
    /// assert_eq!(reader.read::<u8>(5).await.unwrap(), 0b10110);
    /// # });
    /// ```
    async fn skip(&mut self, mut bits: u32) -> io::Result<()> {
        use std::cmp::min;

        let to_drop = min(self.bitqueue.len(), bits);
        if to_drop != 0 {
            self.bitqueue.drop(to_drop);
            bits -= to_drop;
        }

        skip_aligned(&mut self.reader, bits / 8).await?;
        skip_unaligned(&mut self.reader, bits % 8, &mut self.bitqueue).await
    }

    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = b"foobar";
    /// let mut reader = BitReader::endian(Cursor::new(data), BigEndian);
    /// assert!(reader.skip(24).await.is_ok());
    /// let mut buf = [0;3];
    /// assert!(reader.read_bytes(&mut buf).await.is_ok());
    /// assert_eq!(&buf, b"bar");
    /// # });
    /// ```
    async fn read_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if self.byte_aligned() {
            self.reader.read_exact(buf).await?;
            Ok(())
        } else {
            for b in buf.iter_mut() {
                *b = self.read(8).await?;
            }
            Ok(())
        }
    }

    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0b01110111, 0b11111110];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read_unary0().await.unwrap(), 0);
    /// assert_eq!(reader.read_unary0().await.unwrap(), 3);
    /// assert_eq!(reader.read_unary0().await.unwrap(), 10);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{LittleEndian, BitReader, BitRead};
    /// let data = [0b11101110, 0b01111111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), LittleEndian);
    /// assert_eq!(reader.read_unary0().await.unwrap(), 0);
    /// assert_eq!(reader.read_unary0().await.unwrap(), 3);
    /// assert_eq!(reader.read_unary0().await.unwrap(), 10);
    /// # });
    /// ```
    async fn read_unary0(&mut self) -> io::Result<u32> {
        if self.bitqueue.is_empty() {
            read_aligned_unary(&mut self.reader, 0b1111_1111, &mut self.bitqueue).await
                .map(|u| u + self.bitqueue.pop_1())
        } else if self.bitqueue.all_1() {
            let base = self.bitqueue.len();
            self.bitqueue.clear();
            read_aligned_unary(&mut self.reader, 0b1111_1111, &mut self.bitqueue).await
                .map(|u| base + u + self.bitqueue.pop_1())
        } else {
            Ok(self.bitqueue.pop_1())
        }
    }

    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0b10001000, 0b00000001];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read_unary1().await.unwrap(), 0);
    /// assert_eq!(reader.read_unary1().await.unwrap(), 3);
    /// assert_eq!(reader.read_unary1().await.unwrap(), 10);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{LittleEndian, BitReader, BitRead};
    /// let data = [0b00010001, 0b10000000];
    /// let mut reader = BitReader::endian(Cursor::new(&data), LittleEndian);
    /// assert_eq!(reader.read_unary1().await.unwrap(), 0);
    /// assert_eq!(reader.read_unary1().await.unwrap(), 3);
    /// assert_eq!(reader.read_unary1().await.unwrap(), 10);
    /// # });
    /// ```
    async fn read_unary1(&mut self) -> io::Result<u32> {
        if self.bitqueue.is_empty() {
            read_aligned_unary(&mut self.reader, 0b0000_0000, &mut self.bitqueue).await
                .map(|u| u + self.bitqueue.pop_0())
        } else if self.bitqueue.all_0() {
            let base = self.bitqueue.len();
            self.bitqueue.clear();
            read_aligned_unary(&mut self.reader, 0b0000_0000, &mut self.bitqueue).await
                .map(|u| base + u + self.bitqueue.pop_0())
        } else {
            Ok(self.bitqueue.pop_0())
        }
    }

    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.byte_aligned(), true);
    /// assert!(reader.skip(1).await.is_ok());
    /// assert_eq!(reader.byte_aligned(), false);
    /// assert!(reader.skip(7).await.is_ok());
    /// assert_eq!(reader.byte_aligned(), true);
    /// # });
    /// ```
    #[inline]
    fn byte_aligned(&self) -> bool {
        self.bitqueue.is_empty()
    }

    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0x00, 0xFF];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read::<u8>(4).await.unwrap(), 0);
    /// reader.byte_align();
    /// assert_eq!(reader.read::<u8>(8).await.unwrap(), 0xFF);
    /// # });
    /// ```
    #[inline]
    fn byte_align(&mut self) {
        self.bitqueue.clear()
    }
}

impl<R, E> BitReader<R, E>
where
    E: Endianness,
    R: AsyncRead + AsyncSeek + Unpin + Send + Sync,
{
    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::{Cursor, SeekFrom};
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0x00, 0xFF];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.position_in_bits().await.unwrap(), 0);
    ///
    /// let pos = reader.seek_bits(SeekFrom::Start(5)).await.unwrap();
    /// assert!(pos == 5 && 5 == reader.position_in_bits().await.unwrap());
    ///
    /// let pos = reader.seek_bits(SeekFrom::Current(-2)).await.unwrap();
    /// assert!(pos == 3 && 3 == reader.position_in_bits().await.unwrap());    ///
    ///
    /// let pos = reader.seek_bits(SeekFrom::End(5)).await.unwrap();
    /// assert!(pos == 11 && 11 == reader.position_in_bits().await.unwrap());
    /// # });
    /// ```
    pub async fn seek_bits(&mut self, from: io::SeekFrom) -> io::Result<u64> {
        let mut from = from;
        loop {
            match from {
                io::SeekFrom::Start(from_start_pos) => {
                    let (bytes, bits) = (from_start_pos / 8, (from_start_pos % 8) as u32);
                    self.byte_align();
                    self.reader.seek(io::SeekFrom::Start(bytes)).await?;
                    self.skip(bits).await?;
                    return Ok(from_start_pos)
                }
                io::SeekFrom::End(from_end_pos) => {
                    let reader_end = self.reader.seek(io::SeekFrom::End(0)).await?;
                    let new_pos = (reader_end * 8) as i64 - from_end_pos;
                    assert!(new_pos >= 0, "The final position should be greater than 0");
                    //self.seek_bits(io::SeekFrom::Start(new_pos as u64)).await
                    from = io::SeekFrom::Start(new_pos as u64);
                    continue;
                }
                io::SeekFrom::Current(offset) => {
                    let new_pos = self.position_in_bits().await? as i64 + offset;
                    assert!(new_pos >= 0, "The final position should be greater than 0");
                    //self.seek_bits(io::SeekFrom::Start(new_pos as u64)).await
                    from = io::SeekFrom::Start(new_pos as u64);
                    continue;
                }
            }
        }
    }

    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::fs::read;
    /// use std::io::{Cursor, SeekFrom};
    /// use tokio_bitstream_io::{BigEndian, BitReader, BitRead};
    /// let data = [0x00, 0xFF];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.position_in_bits().await.unwrap(), 0);
    ///
    /// let _: i32 = reader.read_signed(5).await.unwrap();
    /// assert_eq!(reader.position_in_bits().await.unwrap(), 5);
    ///
    /// reader.read_bit().await.unwrap();
    /// assert_eq!(reader.position_in_bits().await.unwrap(), 6);
    /// # });
    /// ```
    #[inline]
    pub async fn position_in_bits(&mut self) -> io::Result<u64> {
        let bytes = self.reader.stream_position().await?;
        Ok(bytes * 8 - (self.bitqueue.len() as u64))
    }
}

#[async_trait::async_trait]
impl<R: AsyncRead + Unpin + Send + Sync, E: Endianness> HuffmanRead<E> for BitReader<R, E> {
    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, BitReader, HuffmanRead};
    /// use tokio_bitstream_io::huffman::compile_read_tree;
    /// let tree = compile_read_tree(
    ///     vec![('a', vec![0]),
    ///          ('b', vec![1, 0]),
    ///          ('c', vec![1, 1, 0]),
    ///          ('d', vec![1, 1, 1])]).unwrap();
    /// let data = [0b10110111];
    /// let mut reader = BitReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read_huffman(&tree).await.unwrap(), 'b');
    /// assert_eq!(reader.read_huffman(&tree).await.unwrap(), 'c');
    /// assert_eq!(reader.read_huffman(&tree).await.unwrap(), 'd');
    /// # });
    /// ```
    async fn read_huffman<T>(&mut self, tree: &[ReadHuffmanTree<E, T>]) -> io::Result<T>
    where
        T: Clone + Send + Sync,
    {
        let mut result: &ReadHuffmanTree<E, T> = &tree[self.bitqueue.to_state()];
        loop {
            match result {
                ReadHuffmanTree::Done(ref value, ref queue_val, ref queue_bits, _) => {
                    self.bitqueue.set(*queue_val, *queue_bits);
                    return Ok(value.clone());
                }
                ReadHuffmanTree::Continue(ref tree) => {
                    result = &tree[read_byte(&mut self.reader).await? as usize];
                }
                ReadHuffmanTree::InvalidState => {
                    panic!("invalid state");
                }
            }
        }
    }
}

#[inline]
async fn read_byte<R>(mut reader: R) -> io::Result<u8>
where
    R: AsyncRead + Unpin + Send + Sync,
{
    let mut buf = [0; 1];
    reader.read_exact(&mut buf).await.map(|_| buf[0])
}

async fn read_aligned<R, E, N>(mut reader: R, bytes: u32, acc: &mut BitQueue<E, N>) -> io::Result<()>
where
    R: AsyncRead + Unpin + Send + Sync,
    E: Endianness,
    N: Numeric,
{
    if bytes > 0 {
        let mut buf = N::buffer();
        reader.read_exact(&mut buf.as_mut()[0..bytes as usize]).await?;
        for b in &buf.as_ref()[0..bytes as usize] {
            acc.push(8, N::from_u8(*b));
        }
    }
    Ok(())
}

async fn skip_aligned<R>(mut reader: R, mut bytes: u32) -> io::Result<()>
where
    R: AsyncRead + Unpin + Send + Sync,
{
    use std::cmp::min;

    /*skip up to 8 bytes at a time
    (unlike with read_aligned, "bytes" may be larger than any native type)*/
    let mut buf = [0; 8];
    while bytes > 0 {
        let to_read = min(8, bytes);
        reader.read_exact(&mut buf[0..to_read as usize]).await?;
        bytes -= to_read;
    }
    Ok(())
}

#[inline]
async fn read_unaligned<R, E, N>(
    reader: R,
    bits: u32,
    acc: &mut BitQueue<E, N>,
    rem: &mut BitQueue<E, u8>,
) -> io::Result<()>
where
    R: AsyncRead + Unpin + Send + Sync,
    E: Endianness,
    N: Numeric,
{
    debug_assert!(bits <= 8);

    if bits > 0 {
        rem.set(read_byte(reader).await?, 8);
        acc.push(bits, N::from_u8(rem.pop(bits)));
    }
    Ok(())
}

#[inline]
async fn skip_unaligned<R, E>(reader: R, bits: u32, rem: &mut BitQueue<E, u8>) -> io::Result<()>
where
    R: AsyncRead + Unpin + Send + Sync,
    E: Endianness,
{
    debug_assert!(bits <= 8);

    if bits > 0 {
        rem.set(read_byte(reader).await?, 8);
        rem.pop(bits);
    }
    Ok(())
}

#[inline]
async fn read_aligned_unary<R, E>(
    mut reader: R,
    continue_val: u8,
    rem: &mut BitQueue<E, u8>,
) -> io::Result<u32>
where
    R: AsyncRead + Unpin + Send + Sync,
    E: Endianness,
{
    let mut acc = 0;
    let mut byte = read_byte(&mut reader).await?;
    while byte == continue_val {
        acc += 8;
        byte = read_byte(&mut reader).await?;
    }
    rem.set(byte, 8);
    Ok(acc)
}

/// A trait for anything that can read aligned values from an input stream
#[async_trait::async_trait]
pub trait ByteRead {
    /// Reads whole numeric value from stream
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    ///
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{BigEndian, ByteReader, ByteRead};
    /// let data = [0b00000000, 0b11111111];
    /// let mut reader = ByteReader::endian(Cursor::new(&data), BigEndian);
    /// assert_eq!(reader.read::<u16>().await.unwrap(), 0b0000000011111111);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Cursor;
    /// use tokio_bitstream_io::{LittleEndian, ByteReader, ByteRead};
    /// let data = [0b00000000, 0b11111111];
    /// let mut reader = ByteReader::endian(Cursor::new(&data), LittleEndian);
    /// assert_eq!(reader.read::<u16>().await.unwrap(), 0b1111111100000000);
    /// # });
    /// ```
    async fn read<N: Numeric>(&mut self) -> Result<N, io::Error>;

    /// Completely fills the given buffer with whole bytes.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        for b in buf.iter_mut() {
            *b = self.read().await?;
        }
        Ok(())
    }

    /// Completely fills a whole buffer with bytes and returns it.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_to_bytes<const SIZE: usize>(&mut self) -> io::Result<[u8; SIZE]> {
        let mut buf = [0; SIZE];
        self.read_bytes(&mut buf).await?;
        Ok(buf)
    }

    /// Completely fills a vector of bytes and returns it.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn read_to_vec(&mut self, bytes: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; bytes];
        self.read_bytes(&mut buf).await?;
        Ok(buf)
    }

    /// Skips the given number of bytes in the stream.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn skip(&mut self, bytes: u32) -> io::Result<()>;
}

/// For reading aligned bytes from a stream of bytes in a given endianness.
///
/// This only reads aligned values and maintains no internal state.
pub struct ByteReader<R: AsyncRead + Unpin + Send + Sync, E: Endianness> {
    phantom: PhantomData<E>,
    reader: R,
}

impl<R: AsyncRead + Unpin + Send + Sync, E: Endianness> ByteReader<R, E> {
    /// Wraps a ByteReader around something that implements `Read`
    pub fn new(reader: R) -> ByteReader<R, E> {
        ByteReader {
            phantom: PhantomData,
            reader,
        }
    }

    /// Wraps a ByteReader around something that implements `Read`
    /// with the given endianness.
    pub fn endian(reader: R, _endian: E) -> ByteReader<R, E> {
        ByteReader {
            phantom: PhantomData,
            reader,
        }
    }

    /// Unwraps internal reader and disposes of `ByteReader`.
    #[inline]
    pub fn into_reader(self) -> R {
        self.reader
    }

    /// Provides mutable reference to internal reader
    #[inline]
    pub fn reader(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Converts `ByteReader` to `BitReader` in the same endianness.
    #[inline]
    pub fn into_bitreader(self) -> BitReader<R, E> {
        BitReader::new(self.into_reader())
    }

    /// Provides temporary `BitReader` in the same endianness.
    ///
    /// # Warning
    ///
    /// Any unread bits left over when `BitReader` is dropped are lost.
    #[inline]
    pub fn bitreader(&mut self) -> BitReader<&mut R, E> {
        BitReader::new(self.reader())
    }
}

#[async_trait::async_trait]
impl<R: AsyncRead + Unpin + Send + Sync, E: Endianness> ByteRead for ByteReader<R, E> {
    #[inline]
    async fn read<N: Numeric>(&mut self) -> Result<N, io::Error> {
        E::read_numeric(&mut self.reader).await
    }

    #[inline]
    async fn read_bytes(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact(buf).await?;
        Ok(())
    }

    #[inline]
    async fn skip(&mut self, bytes: u32) -> io::Result<()> {
        skip_aligned(&mut self.reader, bytes).await
    }
}
