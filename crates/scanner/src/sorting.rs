//! Path sorting and parsing using segment-based natural sort.
//!
//! Filenames are split into segments (alternating numeric/text runs separated by
//! delimiters like `_`, `-`, `.`, whitespace). Segments are compared naturally:
//! numbers compare numerically, text compares lexicographically (case-insensitive).
//!
//! This handles all common manga naming conventions:
//! - `001.jpg`, `002.jpg` (pure numeric)
//! - `067_067.webp` (doubled page numbers)
//! - `007_p007.webp` (prefix + page)
//! - `Vol01_Ch03_p015.png` (volume-chapter-page)
//! - `1-15.jpg` (volume-chapter with dash)

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// A parsed segment of a filename, used for natural sorting.
#[derive(Debug, PartialEq, Eq)]
enum Segment {
    Number(u64),
    Text(String),
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Segment {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Segment::Number(a), Segment::Number(b)) => a.cmp(b),
            (Segment::Text(a), Segment::Text(b)) => a.cmp(b),
            // Numbers sort before text
            (Segment::Number(_), Segment::Text(_)) => Ordering::Less,
            (Segment::Text(_), Segment::Number(_)) => Ordering::Greater,
        }
    }
}

/// Parses a string into segments by splitting on delimiters and classifying
/// each part as numeric or text.
///
/// Delimiters (`_`, `-`, `.`, whitespace) are consumed as separators.
/// Within each token, alternating digit/non-digit runs become separate segments.
fn parse_segments(input: &str) -> Vec<Segment> {
    let lower = input.to_lowercase();
    let mut segments = Vec::new();

    for token in lower.split(|c: char| c == '_' || c == '-' || c == '.' || c.is_whitespace()) {
        if token.is_empty() {
            continue;
        }

        let mut chars = token.chars().peekable();
        while let Some(&c) = chars.peek() {
            let is_digit = c.is_ascii_digit();
            let mut run = String::new();

            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() != is_digit {
                    break;
                }
                run.push(c);
                chars.next();
            }

            if is_digit {
                segments.push(Segment::Number(run.parse::<u64>().unwrap_or(0)));
            } else if !run.is_empty() {
                segments.push(Segment::Text(run));
            }
        }
    }

    segments
}

/// Natural sort comparison for two strings.
fn natural_sort_str(a: &str, b: &str) -> Ordering {
    let a_segs = parse_segments(a);
    let b_segs = parse_segments(b);

    for (sa, sb) in a_segs.iter().zip(b_segs.iter()) {
        let ord = sa.cmp(sb);
        if ord != Ordering::Equal {
            return ord;
        }
    }

    a_segs.len().cmp(&b_segs.len())
}

/// Natural sort comparison for two paths based on their filename stem (no extension).
///
/// This is the primary sort function for image files.
pub fn natural_sort(a: &PathBuf, b: &PathBuf) -> Ordering {
    let a_stem = a.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let b_stem = b.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    natural_sort_str(a_stem, b_stem)
}

/// Natural sort comparison for two paths based on their full filename (with extension).
///
/// Useful for sorting directories by name.
pub fn natural_sort_by_name(a: &PathBuf, b: &PathBuf) -> Ordering {
    let a_name = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let b_name = b.file_name().and_then(|s| s.to_str()).unwrap_or("");

    natural_sort_str(a_name, b_name)
}

/// Extracts volume and chapter numbers from a path name.
///
/// Looks for patterns like:
/// - `Vol01_Ch03` → (Some(1), Some(3))
/// - `1-15` → (Some(1), Some(15))
/// - `Chapter_5` → (None, Some(5))
/// - `003` → (None, None) — ambiguous single number
pub fn extract_volume_chapter(path: &Path) -> (Option<u64>, Option<u64>) {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let segments = parse_segments(name);

    let mut volume: Option<u64> = None;
    let mut chapter: Option<u64> = None;

    let mut i = 0;
    while i < segments.len() {
        if let Segment::Text(ref text) = segments[i] {
            if (text == "vol" || text == "volume" || text == "v") && i + 1 < segments.len() {
                if let Segment::Number(n) = segments[i + 1] {
                    volume = Some(n);
                    i += 2;
                    continue;
                }
            }
            if (text == "ch" || text == "chapter" || text == "c") && i + 1 < segments.len() {
                if let Segment::Number(n) = segments[i + 1] {
                    chapter = Some(n);
                    i += 2;
                    continue;
                }
            }
        }
        i += 1;
    }

    // Fallback: if segments are exactly [Num, Num] (like "1-15"), treat as (vol, ch)
    if volume.is_none()
        && chapter.is_none()
        && segments.len() == 2
        && matches!(segments[0], Segment::Number(_))
        && matches!(segments[1], Segment::Number(_))
    {
        if let (Segment::Number(v), Segment::Number(c)) = (&segments[0], &segments[1]) {
            volume = Some(*v);
            chapter = Some(*c);
        }
    }

    (volume, chapter)
}

// --- Deprecated functions (kept for transition) ---

#[deprecated(note = "Use natural_sort instead")]
#[inline]
pub fn sort_by_stem_number(a: &PathBuf, b: &PathBuf) -> Ordering {
    natural_sort(a, b)
}

#[deprecated(note = "Use natural_sort instead")]
#[inline]
pub fn sort_name_by_number(a: &PathBuf, b: &PathBuf) -> Ordering {
    natural_sort(a, b)
}

#[deprecated(note = "Use natural_sort instead")]
#[inline]
pub fn sort_by_name_volume_chapter(a: &PathBuf, b: &PathBuf) -> Ordering {
    natural_sort(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_segments_pure_numeric() {
        assert_eq!(parse_segments("001"), vec![Segment::Number(1)]);
        assert_eq!(parse_segments("100"), vec![Segment::Number(100)]);
    }

    #[test]
    fn test_parse_segments_doubled() {
        assert_eq!(
            parse_segments("067_067"),
            vec![Segment::Number(67), Segment::Number(67)]
        );
    }

    #[test]
    fn test_parse_segments_prefix_page() {
        assert_eq!(
            parse_segments("007_p007"),
            vec![
                Segment::Number(7),
                Segment::Text("p".into()),
                Segment::Number(7)
            ]
        );
    }

    #[test]
    fn test_parse_segments_vol_chapter_page() {
        assert_eq!(
            parse_segments("Vol01_Ch03_p015"),
            vec![
                Segment::Text("vol".into()),
                Segment::Number(1),
                Segment::Text("ch".into()),
                Segment::Number(3),
                Segment::Text("p".into()),
                Segment::Number(15),
            ]
        );
    }

    #[test]
    fn test_parse_segments_dash_separated() {
        assert_eq!(
            parse_segments("1-15"),
            vec![Segment::Number(1), Segment::Number(15)]
        );
    }

    #[test]
    fn test_natural_sort_numeric_files() {
        let mut paths: Vec<PathBuf> = vec![
            "010.jpg".into(),
            "002.jpg".into(),
            "100.jpg".into(),
            "001.jpg".into(),
        ];
        paths.sort_by(natural_sort);
        let names: Vec<&str> = paths.iter().filter_map(|p| p.to_str()).collect();
        assert_eq!(names, vec!["001.jpg", "002.jpg", "010.jpg", "100.jpg"]);
    }

    #[test]
    fn test_natural_sort_doubled_numbers() {
        let mut paths: Vec<PathBuf> = vec![
            "068_068.webp".into(),
            "065_065.webp".into(),
            "067_067.webp".into(),
        ];
        paths.sort_by(natural_sort);
        let names: Vec<&str> = paths.iter().filter_map(|p| p.to_str()).collect();
        assert_eq!(
            names,
            vec!["065_065.webp", "067_067.webp", "068_068.webp"]
        );
    }

    #[test]
    fn test_natural_sort_vol_chapter() {
        let mut paths: Vec<PathBuf> = vec![
            "Vol02_Ch01".into(),
            "Vol01_Ch02".into(),
            "Vol01_Ch01".into(),
        ];
        paths.sort_by(natural_sort);
        let names: Vec<&str> = paths.iter().filter_map(|p| p.to_str()).collect();
        assert_eq!(names, vec!["Vol01_Ch01", "Vol01_Ch02", "Vol02_Ch01"]);
    }

    #[test]
    fn test_natural_sort_mixed_prefix() {
        let mut paths: Vec<PathBuf> = vec![
            "007_p007.webp".into(),
            "008_p008.webp".into(),
            "006_p006.webp".into(),
        ];
        paths.sort_by(natural_sort);
        let names: Vec<&str> = paths.iter().filter_map(|p| p.to_str()).collect();
        assert_eq!(
            names,
            vec!["006_p006.webp", "007_p007.webp", "008_p008.webp"]
        );
    }

    #[test]
    fn test_extract_volume_chapter_vol_ch() {
        let path = PathBuf::from("Vol01_Ch03");
        assert_eq!(extract_volume_chapter(&path), (Some(1), Some(3)));
    }

    #[test]
    fn test_extract_volume_chapter_dash() {
        let path = PathBuf::from("1-15");
        assert_eq!(extract_volume_chapter(&path), (Some(1), Some(15)));
    }

    #[test]
    fn test_extract_volume_chapter_chapter_only() {
        let path = PathBuf::from("Chapter_5");
        assert_eq!(extract_volume_chapter(&path), (None, Some(5)));
    }

    #[test]
    fn test_extract_volume_chapter_ambiguous() {
        let path = PathBuf::from("003");
        assert_eq!(extract_volume_chapter(&path), (None, None));
    }
}
