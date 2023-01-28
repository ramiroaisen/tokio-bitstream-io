// Copyright 2017 Brian Langenberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Traits and implementations for writing bits to a stream.
//!
//! ## Example
//!
//! Writing the initial STREAMINFO block to a FLAC file,
//! as documented in its
//! [specification](https://xiph.org/flac/format.html#stream).
//!
//! ```
//! use std::convert::TryInto;
//! use tokio::io::{AsyncWrite, AsyncWriteExt};
//! use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite, ByteWriter, ByteWrite, LittleEndian};
//!  
//! #[derive(Debug, PartialEq, Eq)]
//! struct BlockHeader {
//!     last_block: bool,
//!     block_type: u8,
//!     block_size: u32,
//! }
//!
//! impl BlockHeader {
//!     async fn write<W: AsyncWrite + Send + Sync + Unpin>(&self, w: &mut BitWriter<W, BigEndian>) -> std::io::Result<()> {
//!         w.write_bit(self.last_block).await?;
//!         w.write(7, self.block_type).await?;
//!         w.write(24, self.block_size).await?;
//!         Ok(())
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
//!     async fn write<W: AsyncWrite + Unpin + Send + Sync>(&self, w: &mut BitWriter<W, BigEndian>) -> std::io::Result<()> {
//!         w.write(16, self.minimum_block_size).await?;
//!         w.write(16, self.maximum_block_size).await?;
//!         w.write(24, self.minimum_frame_size).await?;
//!         w.write(24, self.maximum_frame_size).await?;
//!         w.write(20, self.sample_rate).await?;
//!         w.write(3, self.channels - 1).await?;
//!         w.write(5, self.bits_per_sample - 1).await?;
//!         w.write(36, self.total_samples).await?;
//!         w.write_bytes(&self.md5).await?;
//!         Ok(())
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
//!     fn len(&self) -> usize {
//!         4 + self.vendor.len() + 4 + self.comment.iter().map(|c| 4 + c.len()).sum::<usize>()
//!     }
//!
//!     async fn write<W: AsyncWrite + Unpin + Send + Sync>(&self, w: &mut ByteWriter<W, LittleEndian>) -> std::io::Result<()> {
//!         use std::convert::TryInto;
//!
//!         async fn write_entry<W: AsyncWrite + Unpin + Send + Sync>(
//!             w: &mut ByteWriter<W, LittleEndian>,
//!             s: &str,
//!         ) -> std::io::Result<()> {
//!             w.write::<u32>(s.len().try_into().unwrap()).await?;
//!             w.write_bytes(s.as_bytes()).await?;
//!             Ok(())
//!         }
//!
//!         write_entry(w, &self.vendor).await?;
//!         w.write::<u32>(self.comment.len().try_into().unwrap()).await?;
//!         
//!         for s in &self.comment {
//!             write_entry(w, &s).await?    
//!         }
//! 
//!         Ok(())
//!     }
//! }
//!
//! 
//! # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
//! 
//! let mut flac: Vec<u8> = Vec::new();
//!
//! let mut writer = BitWriter::endian(&mut flac, BigEndian);
//! 
//! // stream marker
//! writer.write_bytes(b"fLaC").await.unwrap();
//!
//! // metadata block header
//! (BlockHeader { last_block: false, block_type: 0, block_size: 34 }).write(&mut writer).await.unwrap();
//!
//! // STREAMINFO block
//! (Streaminfo {
//!     minimum_block_size: 4096,
//!     maximum_block_size: 4096,
//!     minimum_frame_size: 1542,
//!     maximum_frame_size: 8546,
//!     sample_rate: 44100,
//!     channels: 2,
//!     bits_per_sample: 16,
//!     total_samples: 304844,
//!     md5: *b"\xFA\xF2\x69\x2F\xFD\xEC\x2D\x5B\x30\x01\x76\xB4\x62\x88\x7D\x92",
//! }).write(&mut writer).await.unwrap();
//!
//! let comment = VorbisComment {
//!     vendor: "reference libFLAC 1.1.4 20070213".to_string(),
//!     comment: vec![
//!         "title=2ch 44100  16bit".to_string(),
//!         "album=Test Album".to_string(),
//!         "artist=Assorted".to_string(),
//!         "tracknumber=1".to_string(),
//!     ],
//! };
//!
//! // metadata block header
//! (BlockHeader {
//!    last_block: false,
//!    block_type: 4,
//!    block_size: comment.len().try_into().unwrap(),
//! }).write(&mut writer).await.unwrap();
//!
//! // VORBIS_COMMENT block (little endian)
//! comment.write(&mut ByteWriter::new(writer.writer().unwrap())).await.unwrap();
//!
//! drop(writer);
//! 
//! assert_eq!(flac, vec![0x66,0x4c,0x61,0x43,0x00,0x00,0x00,0x22,
//!                      0x10,0x00,0x10,0x00,0x00,0x06,0x06,0x00,
//!                      0x21,0x62,0x0a,0xc4,0x42,0xf0,0x00,0x04,
//!                      0xa6,0xcc,0xfa,0xf2,0x69,0x2f,0xfd,0xec,
//!                      0x2d,0x5b,0x30,0x01,0x76,0xb4,0x62,0x88,
//!                      0x7d,0x92,0x04,0x00,0x00,0x7a,0x20,0x00,
//!                      0x00,0x00,0x72,0x65,0x66,0x65,0x72,0x65,
//!                      0x6e,0x63,0x65,0x20,0x6c,0x69,0x62,0x46,
//!                      0x4c,0x41,0x43,0x20,0x31,0x2e,0x31,0x2e,
//!                      0x34,0x20,0x32,0x30,0x30,0x37,0x30,0x32,
//!                      0x31,0x33,0x04,0x00,0x00,0x00,0x16,0x00,
//!                      0x00,0x00,0x74,0x69,0x74,0x6c,0x65,0x3d,
//!                      0x32,0x63,0x68,0x20,0x34,0x34,0x31,0x30,
//!                      0x30,0x20,0x20,0x31,0x36,0x62,0x69,0x74,
//!                      0x10,0x00,0x00,0x00,0x61,0x6c,0x62,0x75,
//!                      0x6d,0x3d,0x54,0x65,0x73,0x74,0x20,0x41,
//!                      0x6c,0x62,0x75,0x6d,0x0f,0x00,0x00,0x00,
//!                      0x61,0x72,0x74,0x69,0x73,0x74,0x3d,0x41,
//!                      0x73,0x73,0x6f,0x72,0x74,0x65,0x64,0x0d,
//!                      0x00,0x00,0x00,0x74,0x72,0x61,0x63,0x6b,
//!                      0x6e,0x75,0x6d,0x62,0x65,0x72,0x3d,0x31]);
//! # }); 
//! ```

#![warn(missing_docs)]

use std::convert::From;
use std::io;
use std::ops::{AddAssign, Rem};

use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::{huffman::WriteHuffmanTree, BitQueue, Endianness, Numeric, PhantomData, SignedNumeric};

/// For writing bit values to an underlying stream in a given endianness.
///
/// Because this only writes whole bytes to the underlying stream,
/// it is important that output is byte-aligned before the bitstream
/// writer's lifetime ends.
/// **Partial bytes will be lost** if the writer is disposed of
/// before they can be written.
pub struct BitWriter<W: AsyncWrite + Unpin + Send + Sync, E: Endianness> {
    writer: W,
    bitqueue: BitQueue<E, u8>,
}

impl<W: AsyncWrite + Unpin + Send + Sync, E: Endianness> BitWriter<W, E> {
    /// Wraps a BitWriter around something that implements `Write`
    pub fn new(writer: W) -> BitWriter<W, E> {
        BitWriter {
            writer,
            bitqueue: BitQueue::new(),
        }
    }

    /// Wraps a BitWriter around something that implements `Write`
    /// with the given endianness.
    pub fn endian(writer: W, _endian: E) -> BitWriter<W, E> {
        BitWriter {
            writer,
            bitqueue: BitQueue::new(),
        }
    }

    /// Unwraps internal writer and disposes of BitWriter.
    ///
    /// # Warning
    ///
    /// Any unwritten partial bits are discarded.
    #[inline]
    pub fn into_writer(self) -> W {
        self.writer
    }

    /// If stream is byte-aligned, provides mutable reference
    /// to internal writer.  Otherwise returns `None`
    #[inline]
    pub fn writer(&mut self) -> Option<&mut W> {
        if self.byte_aligned() {
            Some(&mut self.writer)
        } else {
            None
        }
    }

    /// Converts `BitWriter` to `ByteWriter` in the same endianness.
    ///
    /// # Warning
    ///
    /// Any written partial bits are discarded.
    #[inline]
    pub fn into_bytewriter(self) -> ByteWriter<W, E> {
        ByteWriter::new(self.into_writer())
    }

    /// If stream is byte-aligned, provides temporary `ByteWriter`
    /// in the same endianness.  Otherwise returns `None`
    ///
    /// # Warning
    ///
    /// Any unwritten bits left over when `ByteWriter` is dropped are lost.
    #[inline]
    pub fn bytewriter(&mut self) -> Option<ByteWriter<&mut W, E>> {
        self.writer().map(ByteWriter::new)
    }

    /// Consumes writer and returns any un-written partial byte
    /// as a `(bits, value)` tuple.
    ///
    /// # Examples
    /// ```
    /// use std::io::Write;
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// 
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// let mut data = Vec::new();
    /// let (bits, value) = {
    ///     let mut writer = BitWriter::endian(&mut data, BigEndian);
    ///     writer.write(15, 0b1010_0101_0101_101).await.unwrap();
    ///     writer.into_unwritten()
    /// };
    /// assert_eq!(data, [0b1010_0101]);
    /// assert_eq!(bits, 7);
    /// assert_eq!(value, 0b0101_101);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut data = Vec::new();
    /// let (bits, value) = {
    ///     let mut writer = BitWriter::endian(&mut data, BigEndian);
    ///     writer.write(8, 0b1010_0101).await.unwrap();
    ///     writer.into_unwritten()
    /// };
    /// assert_eq!(data, [0b1010_0101]);
    /// assert_eq!(bits, 0);
    /// assert_eq!(value, 0);
    /// # });
    /// ```
    #[inline(always)]
    pub fn into_unwritten(self) -> (u32, u8) {
        (self.bitqueue.len(), self.bitqueue.value())
    }

    /// Flushes output stream to disk, if necessary.
    /// Any partial bytes are not flushed.
    ///
    /// # Errors
    ///
    /// Passes along any errors from the underlying stream.
    #[inline(always)]
    pub async fn flush(&mut self) -> io::Result<()> {
        self.writer.flush().await
    }
}

/// A trait for anything that can write a variable number of
/// potentially un-aligned values to an output stream
#[async_trait::async_trait]
pub trait BitWrite: Send + Sync {
    /// Writes a single bit to the stream.
    /// `true` indicates 1, `false` indicates 0
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn write_bit(&mut self, bit: bool) -> io::Result<()>;

    /// Writes an unsigned value to the stream using the given
    /// number of bits.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    /// Returns an error if the input type is too small
    /// to hold the given number of bits.
    /// Returns an error if the value is too large
    /// to fit the given number of bits.
    async fn write<U>(&mut self, bits: u32, value: U) -> io::Result<()>
    where
        U: Numeric;

    /// Writes a twos-complement signed value to the stream
    /// with the given number of bits.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    /// Returns an error if the input type is too small
    /// to hold the given number of bits.
    /// Returns an error if the value is too large
    /// to fit the given number of bits.
    async fn write_signed<S>(&mut self, bits: u32, value: S) -> io::Result<()>
    where
        S: SignedNumeric;

    /// Writes the entirety of a byte buffer to the stream.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    ///
    /// # Example
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Write;
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    /// writer.write(8, 0x66).await.unwrap();
    /// writer.write(8, 0x6F).await.unwrap();
    /// writer.write(8, 0x6F).await.unwrap();
    /// writer.write_bytes(b"bar").await.unwrap();
    /// assert_eq!(writer.into_writer(), b"foobar");
    /// # });
    /// ```
    #[inline]
    async fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()> {
        //buf.iter().try_for_each(|b| self.write(8, *b))
        for b in buf {
            self.write(8, *b).await?
        }

        Ok(())
    }

    /// Writes `value` number of 1 bits to the stream
    /// and then writes a 0 bit.  This field is variably-sized.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underyling stream.
    ///
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    /// writer.write_unary0(0).await.unwrap();
    /// writer.write_unary0(3).await.unwrap();
    /// writer.write_unary0(10).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b01110111, 0b11111110]);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{LittleEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), LittleEndian);
    /// writer.write_unary0(0).await.unwrap();
    /// writer.write_unary0(3).await.unwrap();
    /// writer.write_unary0(10).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b11101110, 0b01111111]);
    /// # });
    /// ```
    async fn write_unary0(&mut self, value: u32) -> io::Result<()> {
        match value {
            
            0 => self.write_bit(false).await,
            
            bits @ 1..=31 => {
                // self.write(value, (1u32 << bits) - 1).and_then(|()| self.write_bit(false)),
                self.write(value, (1u32 << bits) - 1).await?;
                self.write_bit(false).await
            }

            32 => {
                // self.write(value, ).and_then(|()| self.write_bit(false)),
                self.write(value, 0xFFFF_FFFFu32).await?;
                self.write_bit(false).await
            }
            
            bits @ 33..=63 => {
                // self.write(value, (1u64 << bits) - 1).and_then(|()| self.write_bit(false)),
                self.write(value, (1u64 << bits) - 1).await?;
                self.write_bit(false).await
            }

            64 => {
                // self.write().and_then(|()| self.write_bit(false)),
                self.write(value, 0xFFFF_FFFF_FFFF_FFFFu64).await?;
                self.write_bit(false).await
            }
            
            mut bits => {
                while bits > 64 {
                    self.write(64, 0xFFFF_FFFF_FFFF_FFFFu64).await?;
                    bits -= 64;
                }
                self.write_unary0(bits).await
            }
        }
    }

    /// Writes `value` number of 0 bits to the stream
    /// and then writes a 1 bit.  This field is variably-sized.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underyling stream.
    ///
    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async { 
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    /// writer.write_unary1(0).await.unwrap();
    /// writer.write_unary1(3).await.unwrap();
    /// writer.write_unary1(10).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b10001000, 0b00000001]);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{LittleEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), LittleEndian);
    /// writer.write_unary1(0).await.unwrap();
    /// writer.write_unary1(3).await.unwrap();
    /// writer.write_unary1(10).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b00010001, 0b10000000]);
    /// # });
    /// ```
    async fn write_unary1(&mut self, value: u32) -> io::Result<()> {
        match value {
            0 => self.write_bit(true).await,
            
            1..=32 => {
                // self.write(value, 0u32).and_then(|()| self.write_bit(true)),
                self.write(value, 0u32).await?;
                self.write_bit(true).await
            }
            
            33..=64 => {
                // self.write(value, 0u64).and_then(|()| self.write_bit(true)),
                self.write(value, 0u64).await?;
                self.write_bit(true).await
            }
            
            mut bits => {
                while bits > 64 {
                    self.write(64, 0u64).await?;
                    bits -= 64;
                }
                self.write_unary1(bits).await
            }
        }
    }

    /// Returns true if the stream is aligned at a whole byte.
    fn byte_aligned(&self) -> bool;

    /// Pads the stream with 0 bits until it is aligned at a whole byte.
    /// Does nothing if the stream is already aligned.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underyling stream.
    ///
    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use std::io::Write;
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    /// writer.write(1, 0).await.unwrap();
    /// writer.byte_align().await.unwrap();
    /// writer.write(8, 0xFF).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0x00, 0xFF]);
    /// # });
    /// ```
    async fn byte_align(&mut self) -> io::Result<()> {
        while !self.byte_aligned() {
            self.write_bit(false).await?;
        }
        Ok(())
    }
}

/// A trait for anything that can write Huffman codes
/// of a given endianness to an output stream
#[async_trait::async_trait]
pub trait HuffmanWrite<E: Endianness> {
    /// Writes Huffman code for the given symbol to the stream.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn write_huffman<T>(&mut self, tree: &WriteHuffmanTree<E, T>, symbol: T) -> io::Result<()>
    where
        T: Ord + Copy + Send + Sync;
}

#[async_trait::async_trait]
impl<W: AsyncWrite + Unpin + Send + Sync, E: Endianness> BitWrite for BitWriter<W, E> {
    /// # Examples
    /// ```
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(false).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(false).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b10110111]);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{LittleEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), LittleEndian);
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(false).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// writer.write_bit(false).await.unwrap();
    /// writer.write_bit(true).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b10110111]);
    /// # });
    /// ```
    async fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        self.bitqueue.push(1, u8::from(bit));
        if self.bitqueue.is_full() {
            write_byte(&mut self.writer, self.bitqueue.pop(8)).await
        } else {
            Ok(())
        }
    }

    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    /// writer.write(1, 0b1).await.unwrap();
    /// writer.write(2, 0b01).await.unwrap();
    /// writer.write(5, 0b10111).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b10110111]);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{LittleEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), LittleEndian);
    /// writer.write(1, 0b1).await.unwrap();
    /// writer.write(2, 0b11).await.unwrap();
    /// writer.write(5, 0b10110).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b10110111]);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio::io::sink;
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut w = BitWriter::endian(sink(), BigEndian);
    /// assert!(w.write(9, 0u8).await.is_err());    // can't write  u8 in 9 bits
    /// assert!(w.write(17, 0u16).await.is_err());  // can't write u16 in 17 bits
    /// assert!(w.write(33, 0u32).await.is_err());  // can't write u32 in 33 bits
    /// assert!(w.write(65, 0u64).await.is_err());  // can't write u64 in 65 bits
    /// assert!(w.write(1, 2).await.is_err());      // can't write   2 in 1 bit
    /// assert!(w.write(2, 4).await.is_err());      // can't write   4 in 2 bits
    /// assert!(w.write(3, 8).await.is_err());      // can't write   8 in 3 bits
    /// assert!(w.write(4, 16).await.is_err());     // can't write  16 in 4 bits
    /// # });
    /// ```
    async fn write<U>(&mut self, bits: u32, value: U) -> io::Result<()>
    where
        U: Numeric,
    {
        if bits > U::BITS_SIZE {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive bits for type written",
            ))
        } else if (bits < U::BITS_SIZE) && (value >= (U::ONE << bits)) {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive value for bits written",
            ))
        } else if bits < self.bitqueue.remaining_len() {
            self.bitqueue.push(bits, value.to_u8());
            Ok(())
        } else {
            let mut acc = BitQueue::from_value(value, bits);
            write_unaligned(&mut self.writer, &mut acc, &mut self.bitqueue).await?;
            write_aligned(&mut self.writer, &mut acc).await?;
            self.bitqueue.push(acc.len(), acc.value().to_u8());
            Ok(())
        }
    }

    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    /// writer.write_signed(4, -5).await.unwrap();
    /// writer.write_signed(4, 7).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b10110111]);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{LittleEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(Vec::new(), LittleEndian);
    /// writer.write_signed(4, 7).await.unwrap();
    /// writer.write_signed(4, -5).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b10110111]);
    /// # });
    /// ```
    #[inline]
    async fn write_signed<S>(&mut self, bits: u32, value: S) -> io::Result<()>
    where
        S: SignedNumeric,
    {
        E::write_signed(self, bits, value).await
    }

    #[inline]
    async fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()> {
        if self.byte_aligned() {
            self.writer.write_all(buf).await
        } else {
            // buf.iter().try_for_each(|b| self.write(8, *b))
            for b in buf {
                self.write(8, *b).await?;
            }
            Ok(())
        }
    }

    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio::io::sink;
    /// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite};
    /// let mut writer = BitWriter::endian(sink(), BigEndian);
    /// assert_eq!(writer.byte_aligned(), true);
    /// writer.write(1, 0).await.unwrap();
    /// assert_eq!(writer.byte_aligned(), false);
    /// writer.write(7, 0).await.unwrap();
    /// assert_eq!(writer.byte_aligned(), true);
    /// # });
    /// ```
    #[inline(always)]
    fn byte_aligned(&self) -> bool {
        self.bitqueue.is_empty()
    }
}

#[async_trait::async_trait]
impl<W: AsyncWrite + Unpin + Send + Sync, E: Endianness> HuffmanWrite<E> for BitWriter<W, E> {
    /// # Example
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{BigEndian, BitWriter, HuffmanWrite};
    /// use tokio_bitstream_io::huffman::compile_write_tree;
    /// 
    /// let tree = compile_write_tree(
    /// vec![('a', vec![0]),
    ///     ('b', vec![1, 0]),
    ///     ('c', vec![1, 1, 0]),
    ///     ('d', vec![1, 1, 1])]).unwrap();
    /// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    /// writer.write_huffman(&tree, 'b').await.unwrap();
    /// writer.write_huffman(&tree, 'c').await.unwrap();
    /// writer.write_huffman(&tree, 'd').await.unwrap();
    /// # });
    /// ```
    #[inline]
    async fn write_huffman<T>(&mut self, tree: &WriteHuffmanTree<E, T>, symbol: T) -> io::Result<()>
    where
        T: Ord + Copy + Send + Sync,
    {
        // tree.get(&symbol).try_for_each(|(bits, value)| self.write(*bits, *value))
        for (bits, value) in tree.get(&symbol) {
            self.write(*bits, *value).await?
        }

        Ok(())
    }
}

/// For counting the number of bits written but generating no output.
///
/// # Example
/// ```
/// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
/// use tokio_bitstream_io::{BigEndian, BitWrite, BitCounter};
/// let mut writer: BitCounter<u32, BigEndian> = BitCounter::new();
/// writer.write(1, 0b1).await.unwrap();
/// writer.write(2, 0b01).await.unwrap();
/// writer.write(5, 0b10111).await.unwrap();
/// assert_eq!(writer.written(), 8);
/// # });
/// ```
#[derive(Default)]
pub struct BitCounter<N, E: Endianness> {
    bits: N,
    phantom: PhantomData<E>,
}

impl<N: Default + Copy, E: Endianness> BitCounter<N, E> {
    /// Creates new counter
    #[inline]
    pub fn new() -> Self {
        BitCounter {
            bits: N::default(),
            phantom: PhantomData,
        }
    }

    /// Returns number of bits written
    #[inline]
    pub fn written(&self) -> N {
        self.bits
    }
}

#[async_trait::async_trait]
impl<N, E> BitWrite for BitCounter<N, E>
where
    E: Endianness,
    N: Copy + AddAssign + From<u32> + Rem<Output = N> + PartialEq + Send + Sync,
{
    #[inline]
    async fn write_bit(&mut self, _bit: bool) -> io::Result<()> {
        self.bits += 1.into();
        Ok(())
    }

    #[inline]
    async fn write<U>(&mut self, bits: u32, value: U) -> io::Result<()>
    where
        U: Numeric,
    {
        if bits > U::BITS_SIZE {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive bits for type written",
            ))
        } else if (bits < U::BITS_SIZE) && (value >= (U::ONE << bits)) {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "excessive value for bits written",
            ))
        } else {
            self.bits += bits.into();
            Ok(())
        }
    }

    #[inline]
    async fn write_signed<S>(&mut self, bits: u32, value: S) -> io::Result<()>
    where
        S: SignedNumeric,
    {
        E::write_signed(self, bits, value).await
    }

    #[inline]
    async fn write_unary1(&mut self, value: u32) -> io::Result<()> {
        self.bits += (value + 1).into();
        Ok(())
    }

    #[inline]
    async fn write_unary0(&mut self, value: u32) -> io::Result<()> {
        self.bits += (value + 1).into();
        Ok(())
    }

    #[inline]
    async fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()> {
        self.bits += (buf.len() as u32 * 8).into();
        Ok(())
    }

    #[inline]
    fn byte_aligned(&self) -> bool {
        self.bits % 8.into() == 0.into()
    }
}

#[async_trait::async_trait]
impl<N, E> HuffmanWrite<E> for BitCounter<N, E>
where
    E: Endianness,
    N: AddAssign + From<u32> + Send + Sync,
{
    async fn write_huffman<T>(&mut self, tree: &WriteHuffmanTree<E, T>, symbol: T) -> io::Result<()>
    where
        T: Ord + Copy + Send + Sync,
    {
        for &(bits, _) in tree.get(&symbol) {
            let bits: N = bits.into();
            self.bits += bits;
        }
        Ok(())
    }
}

/// A generic unsigned value for stream recording purposes
pub struct UnsignedValue(InnerUnsignedValue);

enum InnerUnsignedValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

macro_rules! define_unsigned_value {
    ($t:ty, $n:ident) => {
        impl From<$t> for UnsignedValue {
            #[inline]
            fn from(v: $t) -> Self {
                UnsignedValue(InnerUnsignedValue::$n(v))
            }
        }
    };
}
define_unsigned_value!(u8, U8);
define_unsigned_value!(u16, U16);
define_unsigned_value!(u32, U32);
define_unsigned_value!(u64, U64);
define_unsigned_value!(u128, U128);
define_unsigned_value!(i8, I8);
define_unsigned_value!(i16, I16);
define_unsigned_value!(i32, I32);
define_unsigned_value!(i64, I64);
define_unsigned_value!(i128, I128);

/// A generic signed value for stream recording purposes
pub struct SignedValue(InnerSignedValue);

enum InnerSignedValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

macro_rules! define_signed_value {
    ($t:ty, $n:ident) => {
        impl From<$t> for SignedValue {
            #[inline]
            fn from(v: $t) -> Self {
                SignedValue(InnerSignedValue::$n(v))
            }
        }
    };
}
define_signed_value!(i8, I8);
define_signed_value!(i16, I16);
define_signed_value!(i32, I32);
define_signed_value!(i64, I64);
define_signed_value!(i128, I128);

enum WriteRecord {
    Bit(bool),
    Unsigned { bits: u32, value: UnsignedValue },
    Signed { bits: u32, value: SignedValue },
    Unary0(u32),
    Unary1(u32),
    Bytes(Box<[u8]>),
}

impl WriteRecord {
    async fn playback<W: BitWrite>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            WriteRecord::Bit(v) => writer.write_bit(*v).await,
            WriteRecord::Unsigned {
                bits,
                value: UnsignedValue(value),
            } => match value {
                InnerUnsignedValue::U8(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::U16(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::U32(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::U64(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::U128(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::I8(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::I16(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::I32(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::I64(v) => writer.write(*bits, *v).await,
                InnerUnsignedValue::I128(v) => writer.write(*bits, *v).await,
            },
            WriteRecord::Signed {
                bits,
                value: SignedValue(value),
            } => match value {
                InnerSignedValue::I8(v) => writer.write_signed(*bits, *v).await,
                InnerSignedValue::I16(v) => writer.write_signed(*bits, *v).await,
                InnerSignedValue::I32(v) => writer.write_signed(*bits, *v).await,
                InnerSignedValue::I64(v) => writer.write_signed(*bits, *v).await,
                InnerSignedValue::I128(v) => writer.write_signed(*bits, *v).await,
            },
            WriteRecord::Unary0(v) => writer.write_unary0(*v).await,
            WriteRecord::Unary1(v) => writer.write_unary1(*v).await,
            WriteRecord::Bytes(bytes) => writer.write_bytes(bytes).await,
        }
    }
}

/// For recording writes in order to play them back on another writer
/// # Example
/// ```
/// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
/// use tokio_bitstream_io::{BigEndian, BitWriter, BitWrite, BitRecorder};
/// let mut recorder: BitRecorder<u32, BigEndian> = BitRecorder::new();
/// recorder.write(1, 0b1).await.unwrap();
/// recorder.write(2, 0b01).await.unwrap();
/// recorder.write(5, 0b10111).await.unwrap();
/// assert_eq!(recorder.written(), 8);
/// let mut writer = BitWriter::endian(Vec::new(), BigEndian);
/// recorder.playback(&mut writer).await.unwrap();
/// assert_eq!(writer.into_writer(), [0b10110111]);
/// # });
/// ```
#[derive(Default)]
pub struct BitRecorder<N, E: Endianness> {
    counter: BitCounter<N, E>,
    records: Vec<WriteRecord>,
}

impl<N: Default + Copy + Send + Sync, E: Endianness> BitRecorder<N, E> {
    /// Creates new recorder
    #[inline]
    pub fn new() -> Self {
        BitRecorder {
            counter: BitCounter::new(),
            records: Vec::new(),
        }
    }

    /// Creates new recorder sized for the given number of writes
    #[inline]
    pub fn with_capacity(writes: usize) -> Self {
        BitRecorder {
            counter: BitCounter::new(),
            records: Vec::with_capacity(writes),
        }
    }

    /// Creates new recorder with the given endiannness
    #[inline]
    pub fn endian(_endian: E) -> Self {
        BitRecorder {
            counter: BitCounter::new(),
            records: Vec::new(),
        }
    }

    /// Returns number of bits written
    #[inline]
    pub fn written(&self) -> N {
        self.counter.written()
    }

    /// Plays recorded writes to the given writer
    #[inline]
    pub async fn playback<W: BitWrite>(&self, writer: &mut W) -> io::Result<()> {
        // self.records
        //     .iter()
        //     .try_for_each(|record| record.playback(writer))
        for record in self.records.iter() {
            record.playback(writer).await?
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<N, E> BitWrite for BitRecorder<N, E>
where
    E: Endianness,
    N: Copy + From<u32> + AddAssign + Rem<Output = N> + Eq + Send + Sync,
{
    #[inline]
    async fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        self.records.push(WriteRecord::Bit(bit));
        self.counter.write_bit(bit).await
    }

    #[inline]
    async fn write<U>(&mut self, bits: u32, value: U) -> io::Result<()>
    where
        U: Numeric,
    {
        self.counter.write(bits, value).await?;
        self.records.push(WriteRecord::Unsigned {
            bits,
            value: value.unsigned_value(),
        });
        Ok(())
    }

    #[inline]
    async fn write_signed<S>(&mut self, bits: u32, value: S) -> io::Result<()>
    where
        S: SignedNumeric,
    {
        self.counter.write_signed(bits, value).await?;
        self.records.push(WriteRecord::Signed {
            bits,
            value: value.signed_value(),
        });
        Ok(())
    }

    #[inline]
    async fn write_unary0(&mut self, value: u32) -> io::Result<()> {
        self.records.push(WriteRecord::Unary0(value));
        self.counter.write_unary0(value).await
    }

    #[inline]
    async fn write_unary1(&mut self, value: u32) -> io::Result<()> {
        self.records.push(WriteRecord::Unary1(value));
        self.counter.write_unary1(value).await
    }

    #[inline]
    async fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()> {
        self.records.push(WriteRecord::Bytes(buf.into()));
        self.counter.write_bytes(buf).await
    }

    #[inline]
    fn byte_aligned(&self) -> bool {
        self.counter.byte_aligned()
    }
}

#[async_trait::async_trait]
impl<N, E> HuffmanWrite<E> for BitRecorder<N, E>
where
    E: Endianness,
    N: Copy + From<u32> + AddAssign + Rem<Output = N> + Eq + Send + Sync,
{
    #[inline]
    async fn write_huffman<T>(&mut self, tree: &WriteHuffmanTree<E, T>, symbol: T) -> io::Result<()>
    where
        T: Ord + Copy + Send + Sync,
    {
        // tree.get(&symbol)
        //     .try_for_each(|(bits, value)| self.write(*bits, *value))
        for (bits, value) in tree.get(&symbol) {
            self.write(*bits, *value).await?
        }
        Ok(())
    }
}

#[inline]
async fn write_byte<W>(mut writer: W, byte: u8) -> io::Result<()>
where
    W: AsyncWrite + Unpin + Send + Sync,
{
    let buf = [byte];
    writer.write_all(&buf).await
}

async fn write_unaligned<W, E, N>(
    writer: W,
    acc: &mut BitQueue<E, N>,
    rem: &mut BitQueue<E, u8>,
) -> io::Result<()>
where
    W: AsyncWrite + Unpin + Send + Sync,
    E: Endianness,
    N: Numeric,
{
    if rem.is_empty() {
        Ok(())
    } else {
        use std::cmp::min;
        let bits_to_transfer = min(8 - rem.len(), acc.len());
        rem.push(bits_to_transfer, acc.pop(bits_to_transfer).to_u8());
        if rem.len() == 8 {
            write_byte(writer, rem.pop(8)).await
        } else {
            Ok(())
        }
    }
}

async fn write_aligned<W, E, N>(mut writer: W, acc: &mut BitQueue<E, N>) -> io::Result<()>
where
    W: AsyncWrite + Unpin + Send + Sync,
    E: Endianness,
    N: Numeric,
{
    let to_write = (acc.len() / 8) as usize;
    if to_write > 0 {
        let mut buf = N::buffer();
        let buf_ref: &mut [u8] = buf.as_mut();
        for b in buf_ref[0..to_write].iter_mut() {
            *b = acc.pop(8).to_u8();
        }
        writer.write_all(&buf_ref[0..to_write]).await
    } else {
        Ok(())
    }
}

/// For writing aligned bytes to a stream of bytes in a given endianness.
///
/// This only writes aligned values and maintains no internal state.
pub struct ByteWriter<W: AsyncWrite + Unpin + Send + Sync, E: Endianness> {
    phantom: PhantomData<E>,
    writer: W,
}

impl<W: AsyncWrite + Unpin + Send + Sync, E: Endianness> ByteWriter<W, E> {
    /// Wraps a ByteWriter around something that implements `Write`
    pub fn new(writer: W) -> ByteWriter<W, E> {
        ByteWriter {
            phantom: PhantomData,
            writer,
        }
    }

    /// Wraps a BitWriter around something that implements `Write`
    /// with the given endianness.
    pub fn endian(writer: W, _endian: E) -> ByteWriter<W, E> {
        ByteWriter {
            phantom: PhantomData,
            writer,
        }
    }

    /// Unwraps internal writer and disposes of `ByteWriter`.
    /// Any unwritten partial bits are discarded.
    #[inline]
    pub fn into_writer(self) -> W {
        self.writer
    }

    /// Provides mutable reference to internal writer.
    #[inline]
    pub fn writer(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Converts `ByteWriter` to `BitWriter` in the same endianness.
    #[inline]
    pub fn into_bitwriter(self) -> BitWriter<W, E> {
        BitWriter::new(self.into_writer())
    }

    /// Provides temporary `BitWriter` in the same endianness.
    ///
    /// # Warning
    ///
    /// Any unwritten bits left over when `BitWriter` is dropped are lost.
    #[inline]
    pub fn bitwriter(&mut self) -> BitWriter<&mut W, E> {
        BitWriter::new(self.writer())
    }
}

/// A trait for anything that can write aligned values to an output stream
#[async_trait::async_trait]
pub trait ByteWrite {
    /// Writes whole numeric value to stream
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    /// # Examples
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{BigEndian, ByteWriter, ByteWrite};
    /// let mut writer = ByteWriter::endian(Vec::new(), BigEndian);
    /// writer.write(0b0000000011111111u16).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b00000000, 0b11111111]);
    /// # });
    /// ```
    ///
    /// ```
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// use tokio_bitstream_io::{LittleEndian, ByteWriter, ByteWrite};
    /// let mut writer = ByteWriter::endian(Vec::new(), LittleEndian);
    /// writer.write(0b0000000011111111u16).await.unwrap();
    /// assert_eq!(writer.into_writer(), [0b11111111, 0b00000000]);
    /// # });
    /// ```
    async fn write<N: Numeric>(&mut self, value: N) -> io::Result<()>;

    /// Writes the entirety of a byte buffer to the stream.
    ///
    /// # Errors
    ///
    /// Passes along any I/O error from the underlying stream.
    async fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()>;
}

#[async_trait::async_trait]
impl<W: AsyncWrite + Unpin + Send + Sync, E: Endianness> ByteWrite for ByteWriter<W, E> {
    #[inline]
    async fn write<N: Numeric>(&mut self, value: N) -> io::Result<()> {
        E::write_numeric(&mut self.writer, value).await
    }

    #[inline]
    async fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf).await
    }
}
