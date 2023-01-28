// Copyright 2017 Brian Langenberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate tokio_bitstream_io;
use tokio_bitstream_io::{BigEndian, BitRead, BitReader, BitWrite, BitWriter, LittleEndian};
use std::io::Cursor;

macro_rules! define_roundtrip {
    ($func_name:ident, $endianness:ident) => {
        #[tokio::test]
        async fn $func_name() {
            /*unsigned values*/
            for bits in 1..17 {
                let max = 1 << bits;
                let mut output: Vec<u8> = Vec::with_capacity(max);
                {
                    let mut writer = BitWriter::endian(&mut output, $endianness);
                    for value in 0..max {
                        writer.write(bits, value as u32).await.unwrap();
                    }
                    writer.byte_align().await.unwrap();
                }
                {
                    let mut c = Cursor::new(&output);
                    let mut reader = BitReader::endian(&mut c, $endianness);
                    for value in 0..max {
                        assert_eq!(reader.read::<u32>(bits).await.unwrap(), value as u32);
                    }
                }
            }

            /*signed values*/
            for bits in 2..17 {
                let min = -1i32 << (bits - 1);
                let max = 1i32 << (bits - 1);
                let mut output: Vec<u8> = Vec::with_capacity(max as usize);
                {
                    let mut writer = BitWriter::endian(&mut output, $endianness);
                    for value in min..max {
                        writer.write_signed(bits, value as i32).await.unwrap();
                    }
                    writer.byte_align().await.unwrap();
                }
                {
                    let mut c = Cursor::new(&output);
                    let mut reader = BitReader::endian(&mut c, $endianness);
                    for value in min..max {
                        assert_eq!(reader.read_signed::<i32>(bits).await.unwrap(), value as i32);
                    }
                }
            }
        }
    };
}

define_roundtrip!(test_roundtrip_be, BigEndian);
define_roundtrip!(test_roundtrip_le, LittleEndian);

macro_rules! define_unary_roundtrip {
    ($func_name:ident, $endianness:ident) => {
        #[tokio::test]
        async fn $func_name() {
            let mut output: Vec<u8> = Vec::new();
            {
                let mut writer = BitWriter::endian(&mut output, $endianness);
                for value in 0..1024 {
                    writer.write_unary0(value).await.unwrap();
                }
                writer.byte_align().await.unwrap();
            }
            {
                let mut c = Cursor::new(&output);
                let mut reader = BitReader::endian(&mut c, $endianness);
                for value in 0..1024 {
                    assert_eq!(reader.read_unary0().await.unwrap(), value);
                }
            }

            let mut output: Vec<u8> = Vec::new();
            {
                let mut writer = BitWriter::endian(&mut output, $endianness);
                for value in 0..1024 {
                    writer.write_unary1(value).await.unwrap();
                }
                writer.byte_align().await.unwrap();
            }
            {
                let mut c = Cursor::new(&output);
                let mut reader = BitReader::endian(&mut c, $endianness);
                for value in 0..1024 {
                    assert_eq!(reader.read_unary1().await.unwrap(), value);
                }
            }
        }
    };
}

define_unary_roundtrip!(test_unary_roundtrip_be, BigEndian);
define_unary_roundtrip!(test_unary_roundtrip_le, LittleEndian);
