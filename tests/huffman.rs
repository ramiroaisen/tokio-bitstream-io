#![allow(clippy::same_item_push)]

extern crate tokio_bitstream_io;
use tokio_bitstream_io::huffman::{compile_read_tree, compile_write_tree, HuffmanTreeError};

#[tokio::test]
async fn test_huffman_errors() {
    use tokio_bitstream_io::BE;

    let empty: Vec<(i32, Vec<u8>)> = Vec::new();
    assert!(if let Err(err) = compile_read_tree::<BE, i32>(empty) {
        err == HuffmanTreeError::MissingLeaf
    } else {
        false
    });

    assert!(
        if let Err(err) = compile_read_tree::<BE, u32>(vec![(0u32, vec![0, 1, 2])]) {
            err == HuffmanTreeError::InvalidBit
        } else {
            false
        }
    );

    assert!(if let Err(err) =
        compile_read_tree::<BE, u32>(vec![(0u32, vec![1]), (1u32, vec![0, 1])])
    {
        err == HuffmanTreeError::MissingLeaf
    } else {
        false
    });

    assert!(if let Err(err) = compile_read_tree::<BE, u32>(vec![
        (0u32, vec![1]),
        (1u32, vec![0, 1]),
        (2u32, vec![0, 0]),
        (3u32, vec![0, 0]),
    ]) {
        err == HuffmanTreeError::DuplicateLeaf
    } else {
        false
    });

    assert!(if let Err(err) = compile_read_tree::<BE, u32>(vec![
        (0u32, vec![1]),
        (1u32, vec![0]),
        (2u32, vec![0, 0]),
        (3u32, vec![0, 1]),
    ]) {
        err == HuffmanTreeError::OrphanedLeaf
    } else {
        false
    });

    assert!(
        if let Err(err) = compile_write_tree::<BE, u32>(vec![(0, vec![1, 1, 2])]) {
            err == HuffmanTreeError::InvalidBit
        } else {
            false
        }
    );
}

#[tokio::test]
async fn test_huffman_values() {
    use tokio_bitstream_io::huffman::compile_read_tree;
    use tokio_bitstream_io::{BigEndian, BitReader, HuffmanRead};
    use std::io::Cursor;
    use std::ops::Deref;
    use std::sync::Arc;

    let data = [0xB1, 0xED];

    // we can lookup values that aren't just integers also
    let tree = compile_read_tree(vec![
        (Some(0), vec![0]),
        (Some(1), vec![1, 0]),
        (Some(2), vec![1, 1, 0]),
        (None, vec![1, 1, 1]),
    ])
    .unwrap();
    let mut r = BitReader::endian(Cursor::new(&data), BigEndian);
    assert_eq!(r.read_huffman(&tree).await.unwrap(), Some(1));
    assert_eq!(r.read_huffman(&tree).await.unwrap(), Some(2));
    assert_eq!(r.read_huffman(&tree).await.unwrap(), Some(0));
    assert_eq!(r.read_huffman(&tree).await.unwrap(), Some(0));
    assert_eq!(r.read_huffman(&tree).await.unwrap(), None);

    // we can even lookup potentially large values,
    // preferably using Rc to avoid making copies of each one
    let tree = compile_read_tree(vec![
        (Arc::new("foo".to_owned()), vec![0]),
        (Arc::new("bar".to_owned()), vec![1, 0]),
        (Arc::new("baz".to_owned()), vec![1, 1, 0]),
        (Arc::new("kelp".to_owned()), vec![1, 1, 1]),
    ])
    .unwrap();
    let mut r = BitReader::endian(Cursor::new(&data), BigEndian);
    assert_eq!(r.read_huffman(&tree).await.unwrap().deref(), "bar");
    assert_eq!(r.read_huffman(&tree).await.unwrap().deref(), "baz");
    assert_eq!(r.read_huffman(&tree).await.unwrap().deref(), "foo");
    assert_eq!(r.read_huffman(&tree).await.unwrap().deref(), "foo");
    assert_eq!(r.read_huffman(&tree).await.unwrap().deref(), "kelp");
}

#[tokio::test]
async fn test_lengthy_huffman_values() {
    use tokio_bitstream_io::huffman::{compile_read_tree, compile_write_tree};
    use tokio_bitstream_io::{BitReader, BitWrite, BitWriter, HuffmanRead, HuffmanWrite, BE, LE};
    use std::io::Cursor;

    let max_bits = 70;
    let mut spec = Vec::new();
    for bits in 0..max_bits {
        let mut entry = Vec::new();
        for _ in 0..bits {
            entry.push(0);
        }
        entry.push(1);
        spec.push((Some(bits), entry));
    }
    let mut entry = Vec::new();
    for _ in 0..max_bits {
        entry.push(0);
    }
    spec.push((None, entry));

    let read_tree_be = compile_read_tree::<BE, Option<i32>>(spec.clone()).unwrap();
    let write_tree_be = compile_write_tree::<BE, Option<i32>>(spec.clone()).unwrap();
    let read_tree_le = compile_read_tree::<LE, Option<i32>>(spec.clone()).unwrap();
    let write_tree_le = compile_write_tree::<LE, Option<i32>>(spec).unwrap();

    let mut data_be = Vec::new();
    let mut data_le = Vec::new();
    {
        let mut writer_be = BitWriter::new(&mut data_be);
        let mut writer_le = BitWriter::new(&mut data_le);
        for _ in 0..20 {
            for bits in 0..max_bits {
                writer_be.write_huffman(&write_tree_be, Some(bits)).await.unwrap();
                writer_le.write_huffman(&write_tree_le, Some(bits)).await.unwrap();
            }
        }
        writer_be.byte_align().await.unwrap();
        writer_le.byte_align().await.unwrap();
    }
    {
        let mut cursor_be = Cursor::new(&data_be);
        let mut cursor_le = Cursor::new(&data_le);
        let mut reader_be = BitReader::new(&mut cursor_be);
        let mut reader_le = BitReader::new(&mut cursor_le);
        for _ in 0..20 {
            for bits in 0..max_bits {
                assert_eq!(reader_be.read_huffman(&read_tree_be).await.unwrap(), Some(bits));
                assert_eq!(reader_le.read_huffman(&read_tree_le).await.unwrap(), Some(bits));
            }
        }
    }
}
