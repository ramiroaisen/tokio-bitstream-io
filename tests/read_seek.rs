#![allow(clippy::bool_assert_comparison)]

use std::io;
use std::io::{Cursor, SeekFrom};

use tokio_bitstream_io::{BigEndian, BitRead, BitReader, Endianness, LittleEndian};

#[tokio::test]
async fn test_reader_pos_be() -> io::Result<()> {
    test_reader_pos::<BigEndian>().await
}

#[tokio::test]
async fn test_reader_pos_le() -> io::Result<()> {
    test_reader_pos::<LittleEndian>().await
}

async fn test_reader_pos<E: Endianness>() -> io::Result<()> {
    let actual_data: [u8; 7] = [
        0b1000_1110,
        0b1000_1110,
        0b1000_1110,
        0b1000_1110,
        0b1000_1110,
        0b1000_1110,
        0b1000_1110,
    ];
    let mut r: BitReader<_, E> = BitReader::new(Cursor::new(&actual_data));

    assert_eq!(r.position_in_bits().await?, 0);
    r.read_bit().await?;
    r.read_bit().await?;
    assert_eq!(r.position_in_bits().await?, 2);
    let _: u32 = r.read(5).await?;
    assert_eq!(r.position_in_bits().await?, 7);
    let _: u32 = r.read(4).await?;
    assert_eq!(r.position_in_bits().await?, 11);
    let mut buf = [0u8; 2];
    r.read_bytes(&mut buf).await?;
    assert_eq!(r.position_in_bits().await?, 27);
    r.read_bit().await?;
    r.read_bit().await?;
    r.read_bit().await?;
    r.read_bit().await?;
    r.read_bit().await?;
    r.read_bit().await?;
    r.read_bit().await?;
    let _: i32 = r.read_signed(9).await?;
    assert_eq!(r.position_in_bits().await?, 43);
    let _: i32 = r.read_signed(5).await?;
    assert_eq!(r.position_in_bits().await?, 48);

    Ok(())
}

#[tokio::test]
pub async fn test_reader_seek_start() -> io::Result<()> {
    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);

    r.seek_bits(SeekFrom::Start(0)).await?;
    assert_eq!(r.position_in_bits().await?, 0);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.position_in_bits().await?, 8);

    r.seek_bits(SeekFrom::Start(2)).await?;
    assert_eq!(r.position_in_bits().await?, 2);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.position_in_bits().await?, 8);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.position_in_bits().await?, 10);

    r.seek_bits(SeekFrom::Start(7)).await?;
    assert_eq!(r.position_in_bits().await?, 7);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, false);

    Ok(())
}

#[tokio::test]
pub async fn test_reader_seek_current() -> io::Result<()> {
    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);

    r.seek_bits(SeekFrom::Current(2)).await?;
    assert_eq!(r.position_in_bits().await?, 2);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, false);
    let _: i32 = r.read_signed(11).await?;
    assert_eq!(r.position_in_bits().await?, 17);

    r.seek_bits(SeekFrom::Current(-3)).await?;
    assert_eq!(r.position_in_bits().await?, 14);
    r.skip(10).await?;
    assert_eq!(r.position_in_bits().await?, 24);
    r.seek_bits(SeekFrom::Current(0)).await?;
    assert_eq!(r.position_in_bits().await?, 24);

    Ok(())
}

#[tokio::test]
pub async fn test_reader_seek_end() -> io::Result<()> {
    let actual_data: [u8; 4] = [0xB1, 0xED, 0x3B, 0xC1];
    let mut r = BitReader::endian(Cursor::new(&actual_data), BigEndian);

    r.seek_bits(SeekFrom::End(7)).await?;
    assert_eq!(r.position_in_bits().await?, 25);
    assert_eq!(r.read_bit().await?, true);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.read_bit().await?, false);
    assert_eq!(r.position_in_bits().await?, 29);
    r.seek_bits(SeekFrom::End(0)).await?;
    assert_eq!(r.position_in_bits().await?, 32);

    Ok(())
}
