//! A minimal EXIF reader: the capture date of a JPEG, nothing else.
//!
//! The capture date is `DateTimeOriginal` (tag 0x9003) in the EXIF sub-IFD.
//! When it is missing, IFD0's `DateTime` (tag 0x0132, the last modification
//! inside the camera or editor) is used instead. Those two can differ, and a
//! reader that gets the sub-IFD pointer wrong silently returns the wrong one;
//! the fixture `DSC_9999.jpeg` in the test suite has different values for
//! the two tags for exactly that reason.

/// The capture date as EXIF writes it, `YYYY:MM:DD HH:MM:SS`.
pub fn capture_date(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let mut i = 2;
    while i + 4 <= bytes.len() {
        if bytes[i] != 0xFF {
            return None;
        }
        let marker = bytes[i + 1];
        let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        if marker == 0xE1 {
            let end = (i + 2 + len).min(bytes.len());
            let segment = &bytes[(i + 4).min(end)..end];
            if let Some(tiff) = segment.strip_prefix(b"Exif\0\0") {
                return parse_tiff(tiff);
            }
        }
        if marker == 0xDA {
            // Start of scan: no more headers.
            return None;
        }
        i += 2 + len;
    }
    None
}

fn parse_tiff(t: &[u8]) -> Option<String> {
    let big_endian = match t.get(0..2)? {
        b"MM" => true,
        b"II" => false,
        _ => return None,
    };
    let r16 = |o: usize| -> Option<u16> {
        let b = t.get(o..o + 2)?;
        Some(if big_endian {
            u16::from_be_bytes([b[0], b[1]])
        } else {
            u16::from_le_bytes([b[0], b[1]])
        })
    };
    let r32 = |o: usize| -> Option<u32> {
        let b = t.get(o..o + 4)?;
        Some(if big_endian {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        })
    };
    if r16(2)? != 42 {
        return None;
    }
    let ifd0 = r32(4)? as usize;

    // Find an entry: (type, count, offset of the value slot).
    let find = |ifd: usize, tag: u16| -> Option<(u16, u32, usize)> {
        let n = r16(ifd)? as usize;
        for k in 0..n.min(4096) {
            let e = ifd + 2 + k * 12;
            if r16(e)? == tag {
                return Some((r16(e + 2)?, r32(e + 4)?, e + 8));
            }
        }
        None
    };
    let ascii = |(ty, count, slot): (u16, u32, usize)| -> Option<String> {
        if ty != 2 {
            return None;
        }
        let count = count as usize;
        // Values of four bytes or fewer live in the slot itself.
        let off = if count <= 4 { slot } else { r32(slot)? as usize };
        let s = t.get(off..off + count)?;
        let s = std::str::from_utf8(s).ok()?.trim_end_matches('\0').trim().to_string();
        if s.len() < 19 || !s.starts_with(|c: char| c.is_ascii_digit()) {
            return None;
        }
        Some(s)
    };

    // The EXIF sub-IFD pointer is a LONG with count 1, so the pointer is the
    // value slot itself, not the count field before it.
    if let Some((_, _, slot)) = find(ifd0, 0x8769)
        && let Some(entry) = r32(slot).and_then(|p| find(p as usize, 0x9003))
        && let Some(date) = ascii(entry)
    {
        return Some(date);
    }
    find(ifd0, 0x0132).and_then(ascii)
}

#[cfg(test)]
mod tests {
    use super::capture_date;

    /// Build a JPEG with only the headers, as the fixture generator does.
    fn jpeg(original: &str, modified: &str) -> Vec<u8> {
        let header = b"MM\x00\x2a\x00\x00\x00\x08";
        let ifd0_off = 8usize;
        let ifd0_size = 2 + 2 * 12 + 4;
        let datetime_off = ifd0_off + ifd0_size;
        let exif_ifd_off = datetime_off + 20;
        let exif_size = 2 + 12 + 4;
        let dto_off = exif_ifd_off + exif_size;
        let mut tiff = header.to_vec();
        tiff.extend_from_slice(&2u16.to_be_bytes());
        tiff.extend_from_slice(&0x0132u16.to_be_bytes());
        tiff.extend_from_slice(&2u16.to_be_bytes());
        tiff.extend_from_slice(&20u32.to_be_bytes());
        tiff.extend_from_slice(&(datetime_off as u32).to_be_bytes());
        tiff.extend_from_slice(&0x8769u16.to_be_bytes());
        tiff.extend_from_slice(&4u16.to_be_bytes());
        tiff.extend_from_slice(&1u32.to_be_bytes());
        tiff.extend_from_slice(&(exif_ifd_off as u32).to_be_bytes());
        tiff.extend_from_slice(&0u32.to_be_bytes());
        tiff.extend_from_slice(modified.as_bytes());
        tiff.push(0);
        tiff.extend_from_slice(&1u16.to_be_bytes());
        tiff.extend_from_slice(&0x9003u16.to_be_bytes());
        tiff.extend_from_slice(&2u16.to_be_bytes());
        tiff.extend_from_slice(&20u32.to_be_bytes());
        tiff.extend_from_slice(&(dto_off as u32).to_be_bytes());
        tiff.extend_from_slice(&0u32.to_be_bytes());
        tiff.extend_from_slice(original.as_bytes());
        tiff.push(0);
        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend_from_slice(&tiff);
        let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
        out.extend_from_slice(&((app1.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(&app1);
        out.extend_from_slice(&[0xFF, 0xD9]);
        out
    }

    #[test]
    fn prefers_the_original_date_over_the_modification_date() {
        let j = jpeg("2024:12:31 23:59:59", "2025:01:01 00:00:10");
        assert_eq!(capture_date(&j).as_deref(), Some("2024:12:31 23:59:59"));
    }

    #[test]
    fn rejects_non_jpeg_and_truncated_input() {
        assert_eq!(capture_date(b""), None);
        assert_eq!(capture_date(b"\xFF\xD8\xFF\xD9"), None);
        let j = jpeg("2026:10:03 14:22:31", "2026:10:03 14:22:31");
        assert_eq!(capture_date(&j[..40]), None);
        assert_eq!(capture_date(b"PNG\r\n"), None);
    }
}
