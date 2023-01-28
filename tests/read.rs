#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::bool_assert_comparison)]
// Copyright 2017 Brian Langenberger
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate tokio_bitstream_io;
use std::io::Cursor;

#[tokio::test]
async fn test_read_queue_be() {
    use tokio_bitstream_io::{BitQueue, BE};
    let mut q: BitQueue<BE, u32> = BitQueue::new();
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
    q.push(8, 0b10_110_001);
    assert_eq!(q.len(), 8);
    assert_eq!(q.pop(2), 0b10);
    assert_eq!(q.len(), 6);
    assert_eq!(q.pop(3), 0b110);
    assert_eq!(q.len(), 3);
    q.push(8, 0b11_101_101);
    assert_eq!(q.len(), 11);
    assert_eq!(q.pop(5), 0b001_11);
    assert_eq!(q.len(), 6);
    assert_eq!(q.pop(3), 0b101);
    q.push(8, 0b00111011);
    q.push(8, 0b11000001);
    assert_eq!(q.pop(19), 0b101_00111011_11000001);
    assert!(q.is_empty());
    assert_eq!(q.value(), 0);
}

#[tokio::test]
async fn test_read_queue_le() {
    use tokio_bitstream_io::{BitQueue, LE};
    let mut q: BitQueue<LE, u32> = BitQueue::new();
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
    q.push(8, 0b101_100_01);
    assert_eq!(q.len(), 8);
    assert_eq!(q.pop(2), 0b01);
    assert_eq!(q.len(), 6);
    assert_eq!(q.pop(3), 0b100);
    assert_eq!(q.len(), 3);
    q.push(8, 0b111_011_01);
    assert_eq!(q.len(), 11);
    assert_eq!(q.pop(5), 0b01_101);
    assert_eq!(q.len(), 6);
    assert_eq!(q.pop(3), 0b011);
    q.push(8, 0b00111011);
    q.push(8, 0b11000001);
    assert_eq!(q.pop(19), 0b11000001_00111011_111);
    assert!(q.is_empty());
    assert_eq!(q.value(), 0);
}

#[tokio::test]
async fn test_reader_be() {
    use tokio_bitstream_io::{BigEndian, BitRead, BitReader};

    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];

    /*reading individual bits*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), true);

    /*reading unsigned values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert!(r.byte_aligned());
    assert_eq!(r.read::<u32>(2).await.unwrap(), 2);
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(3).await.unwrap(), 6);
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(5).await.unwrap(), 7);
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(3).await.unwrap(), 5);
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(19).await.unwrap(), 0x53BC1);
    assert!(r.byte_aligned());
    assert!(r.read::<u32>(1).await.is_err());

    /*skipping bits*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert_eq!(r.read::<u32>(2).await.unwrap(), 2);
    assert!(r.skip(3).await.is_ok());
    assert_eq!(r.read::<u32>(5).await.unwrap(), 7);
    assert!(r.skip(3).await.is_ok());
    assert_eq!(r.read::<u32>(19).await.unwrap(), 0x53BC1);

    /*reading signed values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert_eq!(r.read_signed::<i32>(2).await.unwrap(), -2);
    assert_eq!(r.read_signed::<i32>(3).await.unwrap(), -2);
    assert_eq!(r.read_signed::<i32>(5).await.unwrap(), 7);
    assert_eq!(r.read_signed::<i32>(3).await.unwrap(), -3);
    assert_eq!(r.read_signed::<i32>(19).await.unwrap(), -181311);

    /*reading unary 0 values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert_eq!(r.read_unary0().await.unwrap(), 1);
    assert_eq!(r.read_unary0().await.unwrap(), 2);
    assert_eq!(r.read_unary0().await.unwrap(), 0);
    assert_eq!(r.read_unary0().await.unwrap(), 0);
    assert_eq!(r.read_unary0().await.unwrap(), 4);

    /*reading unary 1 values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert_eq!(r.read_unary1().await.unwrap(), 0);
    assert_eq!(r.read_unary1().await.unwrap(), 1);
    assert_eq!(r.read_unary1().await.unwrap(), 0);
    assert_eq!(r.read_unary1().await.unwrap(), 3);
    assert_eq!(r.read_unary1().await.unwrap(), 0);

    /*byte aligning*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert_eq!(r.read::<u32>(3).await.unwrap(), 5);
    r.byte_align();
    assert_eq!(r.read::<u32>(3).await.unwrap(), 7);
    r.byte_align();
    r.byte_align();
    assert_eq!(r.read::<u32>(8).await.unwrap(), 59);
    r.byte_align();
    assert_eq!(r.read::<u32>(4).await.unwrap(), 12);

    /*reading bytes, aligned*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    let mut sub_data = [0; 2];
    assert!(r.read_bytes(&mut sub_data).await.is_ok());
    assert_eq!(&sub_data, b"\xB1\xED");

    /*reading bytes, un-aligned*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    let mut sub_data = [0; 2];
    assert_eq!(r.read::<u32>(4).await.unwrap(), 11);
    assert!(r.read_bytes(&mut sub_data).await.is_ok());
    assert_eq!(&sub_data, b"\x1E\xD3");
}

#[tokio::test]
async fn test_edge_cases_be() {
    use tokio_bitstream_io::{BigEndian, BitRead, BitReader};

    let data: Vec<u8> = vec![
        0, 0, 0, 0, 255, 255, 255, 255, 128, 0, 0, 0, 127, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0,
        255, 255, 255, 255, 255, 255, 255, 255, 128, 0, 0, 0, 0, 0, 0, 0, 127, 255, 255, 255, 255,
        255, 255, 255,
    ];

    /*0 bit reads*/
    let mut r = BitReader::endian(Cursor::new(vec![255]), BigEndian);
    assert_eq!(r.read::<u8>(0).await.unwrap(), 0);
    assert_eq!(r.read::<u16>(0).await.unwrap(), 0);
    assert_eq!(r.read::<u32>(0).await.unwrap(), 0);
    assert_eq!(r.read::<u64>(0).await.unwrap(), 0);
    assert_eq!(r.read::<u8>(8).await.unwrap(), 255);

    /*unsigned 32 and 64-bit values*/
    let mut r = BitReader::endian(Cursor::new(&data), BigEndian);
    assert_eq!(r.read::<u32>(32).await.unwrap(), 0);
    assert_eq!(r.read::<u32>(32).await.unwrap(), 4294967295);
    assert_eq!(r.read::<u32>(32).await.unwrap(), 2147483648);
    assert_eq!(r.read::<u32>(32).await.unwrap(), 2147483647);
    assert_eq!(r.read::<u64>(64).await.unwrap(), 0);
    assert_eq!(r.read::<u64>(64).await.unwrap(), 0xFFFFFFFFFFFFFFFF);
    assert_eq!(r.read::<u64>(64).await.unwrap(), 9223372036854775808);
    assert_eq!(r.read::<u64>(64).await.unwrap(), 9223372036854775807);

    /*signed 32 and 64-bit values*/
    let mut r = BitReader::endian(Cursor::new(&data), BigEndian);
    assert_eq!(r.read::<i32>(32).await.unwrap(), 0);
    assert_eq!(r.read::<i32>(32).await.unwrap(), -1);
    assert_eq!(r.read::<i32>(32).await.unwrap(), -2147483648);
    assert_eq!(r.read::<i32>(32).await.unwrap(), 2147483647);
    assert_eq!(r.read::<i64>(64).await.unwrap(), 0);
    assert_eq!(r.read::<i64>(64).await.unwrap(), -1);
    assert_eq!(r.read::<i64>(64).await.unwrap(), -9223372036854775808);
    assert_eq!(r.read::<i64>(64).await.unwrap(), 9223372036854775807);
}

#[tokio::test]
async fn test_reader_huffman_be() {
    use tokio_bitstream_io::huffman::compile_read_tree;
    use tokio_bitstream_io::{BigEndian, BitReader, HuffmanRead};

    let tree = compile_read_tree(vec![
        (0, vec![1, 1]),
        (1, vec![1, 0]),
        (2, vec![0, 1]),
        (3, vec![0, 0, 1]),
        (4, vec![0, 0, 0]),
    ])
    .unwrap();

    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);

    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 4);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 2);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 2);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 2);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 4);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 2);
}

#[tokio::test]
async fn test_reader_le() {
    use tokio_bitstream_io::{BitRead, BitReader, LittleEndian};

    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];

    /*reading individual bits*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), false);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);
    assert_eq!(r.read_bit().await.unwrap(), true);

    /*reading unsigned values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert!(r.byte_aligned());
    assert_eq!(r.read::<u32>(2).await.unwrap(), 1);
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(3).await.unwrap(), 4);
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(5).await.unwrap(), 13);
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(3).await.unwrap(), 3);
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(19).await.unwrap(), 0x609DF);
    assert!(r.byte_aligned());
    assert!(r.read::<u32>(1).await.is_err());

    /*skipping bits*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert_eq!(r.read::<u32>(2).await.unwrap(), 1);
    assert!(r.skip(3).await.is_ok());
    assert_eq!(r.read::<u32>(5).await.unwrap(), 13);
    assert!(r.skip(3).await.is_ok());
    assert_eq!(r.read::<u32>(19).await.unwrap(), 0x609DF);

    /*reading signed values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert_eq!(r.read_signed::<i32>(2).await.unwrap(), 1);
    assert_eq!(r.read_signed::<i32>(3).await.unwrap(), -4);
    assert_eq!(r.read_signed::<i32>(5).await.unwrap(), 13);
    assert_eq!(r.read_signed::<i32>(3).await.unwrap(), 3);
    assert_eq!(r.read_signed::<i32>(19).await.unwrap(), -128545);

    /*reading unary 0 values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert_eq!(r.read_unary0().await.unwrap(), 1);
    assert_eq!(r.read_unary0().await.unwrap(), 0);
    assert_eq!(r.read_unary0().await.unwrap(), 0);
    assert_eq!(r.read_unary0().await.unwrap(), 2);
    assert_eq!(r.read_unary0().await.unwrap(), 2);

    /*reading unary 1 values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert_eq!(r.read_unary1().await.unwrap(), 0);
    assert_eq!(r.read_unary1().await.unwrap(), 3);
    assert_eq!(r.read_unary1().await.unwrap(), 0);
    assert_eq!(r.read_unary1().await.unwrap(), 1);
    assert_eq!(r.read_unary1().await.unwrap(), 0);

    /*byte aligning*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert_eq!(r.read::<u32>(3).await.unwrap(), 1);
    r.byte_align();
    assert_eq!(r.read::<u32>(3).await.unwrap(), 5);
    r.byte_align();
    r.byte_align();
    assert_eq!(r.read::<u32>(8).await.unwrap(), 59);
    r.byte_align();
    assert_eq!(r.read::<u32>(4).await.unwrap(), 1);

    /*reading bytes, aligned*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    let mut sub_data = [0; 2];
    assert!(r.read_bytes(&mut sub_data).await.is_ok());
    assert_eq!(&sub_data, b"\xB1\xED");

    /*reading bytes, un-aligned*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    let mut sub_data = [0; 2];
    assert_eq!(r.read::<u32>(4).await.unwrap(), 1);
    assert!(r.read_bytes(&mut sub_data).await.is_ok());
    assert_eq!(&sub_data, b"\xDB\xBE");
}

#[tokio::test]
async fn test_edge_cases_le() {
    use tokio_bitstream_io::{BitRead, BitReader, LittleEndian};

    let data: Vec<u8> = vec![
        0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 128, 255, 255, 255, 127, 0, 0, 0, 0, 0, 0, 0, 0,
        255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 128, 255, 255, 255, 255, 255,
        255, 255, 127,
    ];

    /*0 bit reads*/
    let mut r = BitReader::endian(Cursor::new(vec![255]), LittleEndian);
    assert_eq!(r.read::<u8>(0).await.unwrap(), 0);
    assert_eq!(r.read::<u16>(0).await.unwrap(), 0);
    assert_eq!(r.read::<u32>(0).await.unwrap(), 0);
    assert_eq!(r.read::<u64>(0).await.unwrap(), 0);
    assert_eq!(r.read::<u8>(8).await.unwrap(), 255);

    /*unsigned 32 and 64-bit values*/
    let mut r = BitReader::endian(Cursor::new(&data), LittleEndian);
    assert_eq!(r.read::<u32>(32).await.unwrap(), 0);
    assert_eq!(r.read::<u32>(32).await.unwrap(), 4294967295);
    assert_eq!(r.read::<u32>(32).await.unwrap(), 2147483648);
    assert_eq!(r.read::<u32>(32).await.unwrap(), 2147483647);
    assert_eq!(r.read::<u64>(64).await.unwrap(), 0);
    assert_eq!(r.read::<u64>(64).await.unwrap(), 0xFFFFFFFFFFFFFFFF);
    assert_eq!(r.read::<u64>(64).await.unwrap(), 9223372036854775808);
    assert_eq!(r.read::<u64>(64).await.unwrap(), 9223372036854775807);

    let mut r = BitReader::endian(Cursor::new(&data), LittleEndian);
    assert_eq!(r.read_signed::<i32>(32).await.unwrap(), 0);
    assert_eq!(r.read_signed::<i32>(32).await.unwrap(), -1);
    assert_eq!(r.read_signed::<i32>(32).await.unwrap(), -2147483648);
    assert_eq!(r.read_signed::<i32>(32).await.unwrap(), 2147483647);
    assert_eq!(r.read_signed::<i64>(64).await.unwrap(), 0);
    assert_eq!(r.read_signed::<i64>(64).await.unwrap(), -1);
    assert_eq!(r.read_signed::<i64>(64).await.unwrap(), -9223372036854775808);
    assert_eq!(r.read_signed::<i64>(64).await.unwrap(), 9223372036854775807);
}

#[tokio::test]
async fn test_reader_huffman_le() {
    use tokio_bitstream_io::huffman::compile_read_tree;
    use tokio_bitstream_io::{BitReader, HuffmanRead, LittleEndian};

    let tree = compile_read_tree(vec![
        (0, vec![1, 1]),
        (1, vec![1, 0]),
        (2, vec![0, 1]),
        (3, vec![0, 0, 1]),
        (4, vec![0, 0, 0]),
    ])
    .unwrap();

    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);

    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 3);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 2);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 0);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 1);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 2);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 4);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), 3);
}

#[tokio::test]
async fn test_reader_io_errors_be() {
    use tokio_bitstream_io::{BigEndian, BitRead, BitReader};
    use std::io::ErrorKind;

    let actual_data: [u8; 1] = [0xB1];

    /*individual bits*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert_eq!(r.read_bit().await.unwrap_err().kind(), ErrorKind::UnexpectedEof);

    /*skipping bits*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert!(r.read::<u32>(7).await.is_ok());
    assert_eq!(r.skip(5).await.unwrap_err().kind(), ErrorKind::UnexpectedEof);

    /*signed values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert!(r.read_signed::<i32>(2).await.is_ok());
    assert!(r.read_signed::<i32>(3).await.is_ok());
    assert_eq!(
        r.read_signed::<i32>(5).await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    /*unary 0 values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert!(r.read_unary0().await.is_ok());
    assert!(r.read_unary0().await.is_ok());
    assert!(r.read_unary0().await.is_ok());
    assert!(r.read_unary0().await.is_ok());
    assert_eq!(
        r.read_unary0().await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    /*unary 1 values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert!(r.read_unary1().await.is_ok());
    assert!(r.read_unary1().await.is_ok());
    assert!(r.read_unary1().await.is_ok());
    assert!(r.read_unary1().await.is_ok());
    assert_eq!(
        r.read_unary1().await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    /*reading bytes, aligned*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    let mut sub_data = [0; 2];
    assert_eq!(
        r.read_bytes(&mut sub_data).await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    /*reading bytes, un-aligned*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    let mut sub_data = [0; 2];
    assert!(r.read::<u32>(4).await.is_ok());
    assert_eq!(
        r.read_bytes(&mut sub_data).await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
}

#[tokio::test]
async fn test_reader_io_errors_le() {
    use tokio_bitstream_io::{BitRead, BitReader, LittleEndian};
    use std::io::ErrorKind;

    let actual_data: [u8; 1] = [0xB1];

    /*individual bits*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert!(r.read_bit().await.is_ok());
    assert_eq!(r.read_bit().await.unwrap_err().kind(), ErrorKind::UnexpectedEof);

    /*skipping bits*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert!(r.read::<u32>(7).await.is_ok());
    assert_eq!(r.skip(5).await.unwrap_err().kind(), ErrorKind::UnexpectedEof);

    /*signed values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert!(r.read_signed::<i32>(2).await.is_ok());
    assert!(r.read_signed::<i32>(3).await.is_ok());
    assert_eq!(
        r.read_signed::<i32>(5).await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    /*unary 0 values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert!(r.read_unary0().await.is_ok());
    assert!(r.read_unary0().await.is_ok());
    assert!(r.read_unary0().await.is_ok());
    assert!(r.read_unary0().await.is_ok());
    assert_eq!(
        r.read_unary0().await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    /*unary 1 values*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    assert!(r.read_unary1().await.is_ok());
    assert!(r.read_unary1().await.is_ok());
    assert!(r.read_unary1().await.is_ok());
    assert!(r.read_unary1().await.is_ok());
    assert_eq!(
        r.read_unary1().await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    /*reading bytes, aligned*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    let mut sub_data = [0; 2];
    assert_eq!(
        r.read_bytes(&mut sub_data).await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );

    /*reading bytes, un-aligned*/
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    let mut sub_data = [0; 2];
    assert!(r.read::<u32>(4).await.is_ok());
    assert_eq!(
        r.read_bytes(&mut sub_data).await.unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
}

#[tokio::test]
async fn test_reader_bits_errors() {
    use tokio_bitstream_io::{BigEndian, BitRead, BitReader, LittleEndian};
    use std::io::ErrorKind;
    let actual_data = [0u8; 10];

    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);

    assert_eq!(r.read::<u8>(9).await.unwrap_err().kind(), ErrorKind::InvalidInput);
    assert_eq!(
        r.read::<u16>(17).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read::<u32>(33).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read::<u64>(65).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );

    assert_eq!(
        r.read_signed::<i8>(9).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read_signed::<i16>(17).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read_signed::<i32>(33).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read_signed::<i64>(65).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );

    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);

    assert_eq!(r.read::<u8>(9).await.unwrap_err().kind(), ErrorKind::InvalidInput);
    assert_eq!(
        r.read::<u16>(17).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read::<u32>(33).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read::<u64>(65).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );

    assert_eq!(
        r.read_signed::<i8>(9).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read_signed::<i16>(17).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read_signed::<i32>(33).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
    assert_eq!(
        r.read_signed::<i64>(65).await.unwrap_err().kind(),
        ErrorKind::InvalidInput
    );
}

#[tokio::test]
async fn test_clone() {
    use tokio_bitstream_io::{BigEndian, BitRead, BitReader};

    // Reading unsigned examples, cloning while unaligned.
    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    assert!(r.byte_aligned());
    assert_eq!(r.read::<u32>(4).await.unwrap(), 0xB);
    let mut r2 = r.clone();
    assert!(!r.byte_aligned());
    assert_eq!(r.read::<u32>(4).await.unwrap(), 0x1);
    assert_eq!(r.read::<u32>(8).await.unwrap(), 0xED);
    assert!(!r2.byte_aligned());
    assert_eq!(r2.read::<u32>(4).await.unwrap(), 0x1);
    assert_eq!(r2.read::<u32>(8).await.unwrap(), 0xED);

    // Can still instantiate a BitReader when the backing std::io::Read is
    // !Clone.
    use tokio::io::AsyncRead;
    struct NotCloneRead<'a>(&'a [u8]);
    impl<'a> AsyncRead for NotCloneRead<'a> {
        fn poll_read(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
                buf: &mut tokio::io::ReadBuf<'_>,
            ) -> std::task::Poll<std::io::Result<()>> {
                std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
            }
        }
        // fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        //     self.0.read(buf)
        // }
            
    let _r = BitReader::endian(NotCloneRead(&actual_data[..]), BigEndian);
}

#[tokio::test]
async fn test_read_bytes() {
    use tokio_bitstream_io::{BigEndian, BitRead, BitReader, LittleEndian};

    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);
    let read_data: [u8; 4] = r.read_to_bytes().await.unwrap();
    assert_eq!(actual_data, read_data);

    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];
    let mut r = BitReader::endian(Cursor::new(&actual_data), LittleEndian);
    let read_data: [u8; 4] = r.read_to_bytes().await.unwrap();
    assert_eq!(actual_data, read_data);
}
