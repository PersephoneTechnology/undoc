//! Read-only compatibility for surplus ENDOFCHAIN padding in legacy XLS containers.

use std::{
    borrow::Cow,
    collections::HashSet,
    io::{self, Cursor},
};

const FREE: u32 = 0xffff_ffff;
const END: u32 = 0xffff_fffe;

pub(crate) fn open(data: &[u8]) -> io::Result<cfb::CompoundFile<Cursor<Cow<'_, [u8]>>>> {
    match cfb::CompoundFile::open(Cursor::new(Cow::Borrowed(data))) {
        Ok(container) => Ok(container),
        Err(original) => {
            let Some(normalized) = normalize_padding(data) else {
                return Err(original);
            };
            // All ordinary cfb validation still runs, including validation of live chains.
            cfb::CompoundFile::open(Cursor::new(Cow::Owned(normalized)))
        }
    }
}

fn u32_at(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        data.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

/// Correct only FAT entries for sectors beyond the physical end of the file.
/// Never change a live allocation link, directory, stream, or DIFAT entry.
fn normalize_padding(data: &[u8]) -> Option<Vec<u8>> {
    if data.get(..8)? != crate::detect::CFB_MAGIC || data.get(28..30)? != [0xfe, 0xff] {
        return None;
    }
    let sector_size = match (data.get(26..28)?, data.get(30..32)?) {
        ([3, 0], [9, 0]) => 512_usize,
        ([4, 0], [12, 0]) => 4096_usize,
        _ => return None,
    };
    if data.len() < sector_size * 2 || data.len() % sector_size != 0 {
        return None;
    }
    let sectors = data.len() / sector_size - 1;
    let fat_count = u32_at(data, 44)? as usize;
    let difat_count = u32_at(data, 72)? as usize;
    if fat_count == 0 || fat_count > sectors || difat_count > sectors {
        return None;
    }
    let entries = sector_size / 4;
    let sector_offset = |id: u32| {
        let id = id as usize;
        (id < sectors).then(|| (id + 1) * sector_size)
    };
    let mut seen = HashSet::new();
    let mut fat = Vec::with_capacity(fat_count);
    for i in 0..fat_count.min(109) {
        let id = u32_at(data, 76 + i * 4)?;
        sector_offset(id)?;
        if !seen.insert(id) {
            return None;
        }
        fat.push(id);
    }
    let mut next = u32_at(data, 68)?;
    for _ in 0..difat_count {
        let offset = sector_offset(next)?;
        if !seen.insert(next) {
            return None;
        }
        for i in 0..(entries - 1) {
            let id = u32_at(data, offset + i * 4)?;
            if fat.len() < fat_count {
                sector_offset(id)?;
                if !seen.insert(id) {
                    return None;
                }
                fat.push(id);
            } else if id != FREE {
                return None;
            }
        }
        next = u32_at(data, offset + sector_size - 4)?;
    }
    if fat.len() != fat_count || !matches!(next, END | FREE) {
        return None;
    }
    if fat_count.checked_mul(entries)? <= sectors {
        return None;
    }
    let mut changes = Vec::new();
    for logical in sectors..fat_count.checked_mul(entries)? {
        let offset = sector_offset(fat[logical / entries])? + (logical % entries) * 4;
        match u32_at(data, offset)? {
            END => changes.push(offset),
            FREE => {}
            _ => return None,
        }
    }
    if changes.is_empty() {
        return None;
    }
    let mut normalized = data.to_vec();
    for offset in changes {
        normalized[offset..offset + 4].copy_from_slice(&FREE.to_le_bytes());
    }
    Some(normalized)
}

#[cfg(test)]
mod tests;
