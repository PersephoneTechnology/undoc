use super::*;
use std::io::{Read, Write};

fn fixture(version: cfb::Version, len: usize) -> Vec<u8> {
    let mut container =
        cfb::CompoundFile::create_with_version(version, Cursor::new(Vec::new())).unwrap();
    container
        .create_stream("/Workbook")
        .unwrap()
        .write_all(&vec![42; len])
        .unwrap();
    container.flush().unwrap();
    container.into_inner().into_inner()
}

// Change only the final unused FAT slot; its containing FAT sector can be in the DIFAT.
fn end_padding(mut bytes: Vec<u8>) -> Vec<u8> {
    let size = 1_usize << u16::from_le_bytes(bytes[30..32].try_into().unwrap());
    let count = u32_at(&bytes, 44).unwrap() as usize;
    let last = if count <= 109 {
        u32_at(&bytes, 76 + (count - 1) * 4).unwrap()
    } else {
        let difat = u32_at(&bytes, 68).unwrap() as usize;
        assert_eq!(u32_at(&bytes, 72), Some(1));
        u32_at(&bytes, (difat + 1) * size + (count - 110) * 4).unwrap()
    };
    let offset = (last as usize + 2) * size - 4;
    assert_eq!(u32_at(&bytes, offset), Some(FREE));
    bytes[offset..offset + 4].copy_from_slice(&END.to_le_bytes());
    bytes
}

#[test]
fn compatibility_handles_both_sector_sizes_and_difat_without_changing_streams() {
    for (version, len) in [
        (cfb::Version::V3, 8192),
        (cfb::Version::V4, 8192),
        (cfb::Version::V3, 8 * 1024 * 1024),
    ] {
        let original = fixture(version, len);
        assert!(normalize_padding(&original).is_none());
        let malformed = end_padding(original.clone());
        assert!(cfb::CompoundFile::open(Cursor::new(&malformed)).is_err());
        assert_eq!(normalize_padding(&malformed).unwrap(), original);
        let mut container = open(&malformed).unwrap();
        let mut contents = Vec::new();
        container
            .open_stream("/Workbook")
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, vec![42; len]);
    }
}

#[test]
fn compatibility_rejects_invalid_fat_addressing_and_unknown_padding() {
    let original = end_padding(fixture(cfb::Version::V3, 8192));
    for (offset, value) in [(44, u32::MAX), (68, 0), (72, 1), (76, 10000)] {
        let mut bytes = original.clone();
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(normalize_padding(&bytes).is_none(), "offset {offset}");
    }
    let mut bytes = original;
    let fat = u32_at(&bytes, 76).unwrap() as usize;
    let last = (fat + 2) * 512 - 4;
    bytes[last..last + 4].copy_from_slice(&12345_u32.to_le_bytes());
    assert!(normalize_padding(&bytes).is_none());
}

#[test]
fn compatibility_preserves_encrypted_classification() {
    let mut container =
        cfb::CompoundFile::create_with_version(cfb::Version::V3, Cursor::new(Vec::new())).unwrap();
    container
        .create_stream("/EncryptedPackage")
        .unwrap()
        .write_all(&[42; 8192])
        .unwrap();
    container.flush().unwrap();
    let bytes = end_padding(container.into_inner().into_inner());
    assert!(matches!(
        crate::parse_bytes(&bytes),
        Err(crate::Error::Encrypted)
    ));
}
