pub const PE_HEADER_LIMIT: usize = 1024 * 1024;
pub const PATTERN_LEN: usize = 10;

#[derive(Debug, PartialEq, Eq)]
pub struct Section {
    pub name: [u8; 8],
    pub rva: u32,
    pub size: u32,
    pub characteristics: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Image {
    pub size: u32,
    pub sections: Vec<Section>,
}

fn bytes<const N: usize>(data: &[u8], offset: usize) -> Result<[u8; N], String> {
    let end = offset.checked_add(N).ok_or("header offset overflow")?;
    data.get(offset..end)
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| format!("truncated PE header at {offset:#x}"))
}

fn u16_at(data: &[u8], offset: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(bytes(data, offset)?))
}

fn u32_at(data: &[u8], offset: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(bytes(data, offset)?))
}

pub fn pe_coff_header_len(dos: &[u8]) -> Result<usize, String> {
    if dos.get(..2) != Some(b"MZ") {
        return Err("missing DOS signature".into());
    }
    let pe = u32_at(dos, 0x3c)? as usize;
    if !(64..=PE_HEADER_LIMIT - 24).contains(&pe) {
        return Err("PE header offset outside supported bounds".into());
    }
    Ok(pe + 24)
}

pub fn pe_header_len(prefix: &[u8]) -> Result<usize, String> {
    let optional = pe_coff_header_len(prefix)?;
    let pe = optional - 24;
    if bytes::<4>(prefix, pe)? != *b"PE\0\0" {
        return Err("missing PE signature".into());
    }
    if u16_at(prefix, pe + 4)? != 0x8664 {
        return Err("target image is not AMD64".into());
    }
    let sections = u16_at(prefix, pe + 6)? as usize;
    if !(1..=96).contains(&sections) {
        return Err("invalid PE section count".into());
    }
    let optional_size = u16_at(prefix, pe + 20)? as usize;
    if optional_size < 112 {
        return Err("PE32+ optional header is too short".into());
    }
    let end = optional
        .checked_add(optional_size)
        .and_then(|value| value.checked_add(sections * 40))
        .ok_or("PE section table overflow")?;
    if end > PE_HEADER_LIMIT {
        return Err("PE headers exceed supported size".into());
    }
    Ok(end)
}

pub fn parse_pe(headers: &[u8]) -> Result<Image, String> {
    let header_len = pe_header_len(headers)?;
    if headers.len() < header_len {
        return Err("truncated PE section table".into());
    }
    let optional = pe_coff_header_len(headers)?;
    if u16_at(headers, optional)? != 0x20b {
        return Err("target image is not PE32+".into());
    }
    let size = u32_at(headers, optional + 56)?;
    let mapped_headers = u32_at(headers, optional + 60)?;
    if mapped_headers as usize > PE_HEADER_LIMIT
        || (mapped_headers as usize) < header_len
        || mapped_headers > size
    {
        return Err("invalid PE image/header size".into());
    }
    let section_table = optional + u16_at(headers, optional - 4)? as usize;
    let mut sections: Vec<Section> = Vec::new();
    for start in (section_table..header_len).step_by(40) {
        let section = Section {
            name: bytes(headers, start)?,
            rva: u32_at(headers, start + 12)?,
            size: u32_at(headers, start + 8)?.max(u32_at(headers, start + 16)?),
            characteristics: u32_at(headers, start + 36)?,
        };
        let end = section
            .rva
            .checked_add(section.size)
            .ok_or("PE section range overflow")?;
        if end > size || (section.size != 0 && section.rva < mapped_headers) {
            return Err("PE section outside image or overlaps headers".into());
        }
        if section.size != 0
            && sections.iter().any(|previous| {
                previous.size != 0
                    && section.rva < previous.rva + previous.size
                    && previous.rva < end
            })
        {
            return Err("overlapping PE sections".into());
        }
        sections.push(section);
    }
    Ok(Image { size, sections })
}

fn matches_pattern(window: &[u8]) -> bool {
    window[0] == 0x8b
        && window[1] == 0x0d
        && window[6] == 0xeb
        && window[8] == 0x33
        && window[9] == 0xc0
}

pub fn pattern_offsets(data: &[u8]) -> Vec<usize> {
    data.windows(PATTERN_LEN)
        .enumerate()
        .filter_map(|(offset, window)| matches_pattern(window).then_some(offset))
        .collect()
}

pub fn fps_address(instruction: usize, instruction_bytes: &[u8]) -> Result<usize, String> {
    let window = instruction_bytes
        .get(..PATTERN_LEN)
        .filter(|window| matches_pattern(window))
        .ok_or("FPS instruction signature mismatch")?;
    let displacement = i32::from_le_bytes(bytes(window, 2)?) as isize;
    instruction
        .checked_add(6)
        .and_then(|next| next.checked_add_signed(displacement))
        .ok_or_else(|| "FPS address overflow".into())
}

#[derive(Default)]
pub struct PatternScanner {
    next: Option<usize>,
    tail: Vec<u8>,
}

impl PatternScanner {
    pub fn feed(&mut self, address: usize, data: &[u8]) -> Result<Vec<usize>, String> {
        let next = address
            .checked_add(data.len())
            .ok_or("scan region address overflow")?;
        if self.next != Some(address) {
            self.tail.clear();
        }
        let start = address
            .checked_sub(self.tail.len())
            .ok_or("scan overlap address underflow")?;
        let mut buffer = std::mem::take(&mut self.tail);
        buffer.extend_from_slice(data);
        let matches = pattern_offsets(&buffer)
            .into_iter()
            .map(|offset| start + offset)
            .collect();
        self.tail = buffer[buffer.len().saturating_sub(PATTERN_LEN - 1)..].to_vec();
        self.next = Some(next);
        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATCH: [u8; PATTERN_LEN] = [0x8b, 0x0d, 0x10, 0, 0, 0, 0xeb, 0x13, 0x33, 0xc0];

    fn header() -> Vec<u8> {
        let mut header = vec![0; 512];
        header[..2].copy_from_slice(b"MZ");
        header[60..64].copy_from_slice(&128u32.to_le_bytes());
        header[128..132].copy_from_slice(b"PE\0\0");
        header[132..134].copy_from_slice(&0x8664u16.to_le_bytes());
        header[134..136].copy_from_slice(&2u16.to_le_bytes());
        header[148..150].copy_from_slice(&240u16.to_le_bytes());
        header[152..154].copy_from_slice(&0x20bu16.to_le_bytes());
        header[208..212].copy_from_slice(&0x4000u32.to_le_bytes());
        header[212..216].copy_from_slice(&512u32.to_le_bytes());
        for (index, name) in [*b".text\0\0\0", *b".data\0\0\0"].into_iter().enumerate() {
            let start = 392 + 40 * index;
            header[start..start + 8].copy_from_slice(&name);
            header[start + 8..start + 12].copy_from_slice(&0x800u32.to_le_bytes());
            header[start + 12..start + 16]
                .copy_from_slice(&(0x1000 + index as u32 * 0x1000).to_le_bytes());
            header[start + 16..start + 20].copy_from_slice(&0x1000u32.to_le_bytes());
            header[start + 36..start + 40].copy_from_slice(&0x6000_0020u32.to_le_bytes());
        }
        header
    }

    #[test]
    fn bounded_headers_and_malformed_input() {
        let good = header();
        assert_eq!(pe_coff_header_len(&good[..64]).unwrap(), 152);
        assert_eq!(pe_header_len(&good[..152]).unwrap(), 472);
        let image = parse_pe(&good).unwrap();
        assert_eq!(image.size, 0x4000);
        assert_eq!(image.sections[0].name, *b".text\0\0\0");
        assert_eq!(image.sections[0].size, 0x1000);
        assert_eq!(image.sections[1].rva, 0x2000);
        for len in 0..472 {
            assert!(parse_pe(&good[..len]).is_err(), "accepted {len} bytes");
        }
        for (offset, replacement) in [
            (0, vec![0, 0]),
            (60, u32::MAX.to_le_bytes().to_vec()),
            (60, 63u32.to_le_bytes().to_vec()),
            (128, vec![0, 0, 0, 0]),
            (132, 0x14cu16.to_le_bytes().to_vec()),
            (134, 0u16.to_le_bytes().to_vec()),
            (134, 97u16.to_le_bytes().to_vec()),
            (148, 111u16.to_le_bytes().to_vec()),
            (152, 0x10bu16.to_le_bytes().to_vec()),
            (208, 0u32.to_le_bytes().to_vec()),
            (212, 471u32.to_le_bytes().to_vec()),
            (404, (u32::MAX - 10).to_le_bytes().to_vec()),
            (404, 0x4000u32.to_le_bytes().to_vec()),
            (404, 0x100u32.to_le_bytes().to_vec()),
            (444, 0x1800u32.to_le_bytes().to_vec()),
        ] {
            let mut bad = good.clone();
            bad[offset..offset + replacement.len()].copy_from_slice(&replacement);
            assert!(parse_pe(&bad).is_err(), "accepted mutation at {offset}");
        }
        let mut large_offset = vec![0; PE_HEADER_LIMIT];
        large_offset[..64].copy_from_slice(&good[..64]);
        let pe = PE_HEADER_LIMIT - 24;
        large_offset[60..64].copy_from_slice(&(pe as u32).to_le_bytes());
        large_offset[pe..].copy_from_slice(&good[128..152]);
        assert!(pe_header_len(&large_offset).is_err());
    }

    #[test]
    fn section_table_follows_declared_offsets_and_optional_header_size() {
        let good = header();
        let pe = 8192;
        let optional = pe + 24;
        let section_table = optional + 112;
        let mut moved = vec![0; section_table + 80];
        moved[..64].copy_from_slice(&good[..64]);
        moved[60..64].copy_from_slice(&(pe as u32).to_le_bytes());
        moved[pe..optional + 112].copy_from_slice(&good[128..152 + 112]);
        moved[pe + 20..pe + 22].copy_from_slice(&112u16.to_le_bytes());
        moved[optional + 56..optional + 60].copy_from_slice(&0x6000u32.to_le_bytes());
        moved[optional + 60..optional + 64].copy_from_slice(&0x3000u32.to_le_bytes());
        moved[section_table..].copy_from_slice(&good[392..472]);
        moved[section_table + 12..section_table + 16].copy_from_slice(&0x4000u32.to_le_bytes());
        moved[section_table + 52..section_table + 56].copy_from_slice(&0x5000u32.to_le_bytes());
        assert_eq!(pe_coff_header_len(&moved[..64]).unwrap(), optional);
        assert_eq!(pe_header_len(&moved[..optional]).unwrap(), moved.len());
        let image = parse_pe(&moved).unwrap();
        assert_eq!(image.sections[0].rva, 0x4000);
        assert_eq!(image.sections[1].rva, 0x5000);
    }

    #[test]
    fn scan_includes_equal_length_last_slot_and_multiple_candidates() {
        assert!(pattern_offsets(&[]).is_empty());
        assert!(pattern_offsets(&MATCH[..9]).is_empty());
        assert_eq!(pattern_offsets(&MATCH), [0]);
        let mut data = vec![0; 7];
        data.extend_from_slice(&MATCH);
        assert_eq!(pattern_offsets(&data), [7]);
        data.extend_from_slice(&MATCH);
        assert_eq!(pattern_offsets(&data), [7, 17]);
    }

    #[test]
    fn streams_contiguous_chunks_without_crossing_gaps_or_duplicate_matches() {
        for split in 1..PATTERN_LEN {
            let mut scanner = PatternScanner::default();
            assert!(scanner.feed(100, &MATCH[..split]).unwrap().is_empty());
            assert_eq!(scanner.feed(100 + split, &MATCH[split..]).unwrap(), [100]);
            assert_eq!(scanner.feed(110, &MATCH).unwrap(), [110]);
            assert!(scanner.feed(120, &[0; 10]).unwrap().is_empty());
            let mut gap = PatternScanner::default();
            assert!(gap.feed(100, &MATCH[..split]).unwrap().is_empty());
            assert!(gap.feed(101 + split, &MATCH[split..]).unwrap().is_empty());
        }
        let mut scanner = PatternScanner::default();
        for (offset, byte) in MATCH.iter().enumerate() {
            let found = scanner.feed(100 + offset, &[*byte]).unwrap();
            assert_eq!(found, if offset == 9 { vec![100] } else { vec![] });
        }
        assert!(scanner.feed(usize::MAX, &[0]).is_err());
    }

    #[test]
    fn rip_relative_addresses_are_signed_checked_and_signature_bound() {
        assert_eq!(fps_address(100, &MATCH).unwrap(), 122);
        let mut negative = MATCH;
        negative[2..6].copy_from_slice(&(-20i32).to_le_bytes());
        assert_eq!(fps_address(100, &negative).unwrap(), 86);
        assert!(fps_address(0, &negative).is_err());
        assert!(fps_address(usize::MAX - 5, &MATCH).is_err());
        assert!(fps_address(usize::MAX - 6, &MATCH).is_err());
        assert!(fps_address(100, &MATCH[..9]).is_err());
        negative[6] = 0;
        assert!(fps_address(100, &negative).is_err());
    }
}
