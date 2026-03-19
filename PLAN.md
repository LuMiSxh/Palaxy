# Palaxy Backend Rewrite Plan

## Goals
1. ZIP file input support
2. Flat structure support (images at root, no nesting)
3. N-level nesting support (arbitrary depth)
4. Robust filename/path parsing for sorting
5. Flatten option (merge all volumes into one output)
6. Non-image file handling (skip silently, warn at analysis)
7. Performance optimizations
8. Code readability improvements

---

## Phase 1: Scanner Crate Rewrite

### 1A. New module: `scanner/src/extractor.rs` — ZIP extraction
- Detect if `source` is a ZIP file (by extension + magic bytes)
- Extract to a temp directory (`std::env::temp_dir().join("palaxy_<hash>")`)
- Return the extracted directory path for the rest of the pipeline
- Cleanup function to remove temp dir after conversion
- Add `zip` dependency to scanner's Cargo.toml

### 1B. Rewrite `scanner/src/collector.rs` — Recursive + flat support
Current problems:
- Hardcoded 2-level depth (base → chapter dirs → image files)
- Errors on mixed content (files AND dirs at same level)
- No image extension filtering at collection time

New design:
```
Collector::new(source: &PathBuf) -> Self
Collector::collect(&mut self) -> EResult<CollectedSource>

struct CollectedSource {
    /// Chapters: each is a list of image paths, in order
    chapters: Vec<Vec<PathBuf>>,
    /// Detected structure type for UI feedback
    structure: SourceStructure,
    /// Non-image files found (for warnings)
    skipped_files: Vec<PathBuf>,
}

enum SourceStructure {
    Flat,              // Images directly at root
    SingleLevel,       // base → chapter_dirs → images (current behavior)
    MultiLevel,        // base → ...nested... → leaf_dirs → images
}
```

Algorithm:
1. Scan root directory entries
2. Classify: does root contain image files? subdirectories? both?
3. **Flat**: root has only images (no subdirs with images) → single chapter
4. **Has subdirs**: recursively find all "leaf image directories" (dirs containing images)
   - A leaf image dir = contains image files
   - If a dir has BOTH images and subdirs containing images, treat the loose images as their own chapter
5. Sort leaf dirs by path (using new smart sort)
6. Within each leaf dir, sort images (using new smart sort)
7. Filter: only collect files with image extensions (jpg, jpeg, png, webp, avif, bmp, gif, tiff)
8. Track skipped non-image files for analysis warnings

### 1C. Rewrite `scanner/src/sorting.rs` — Robust multi-strategy parser
Current problems:
- `sort_by_stem_number`: only works if entire stem is numeric
- `sort_name_by_number`: takes last number only, loses context
- `sort_by_name_volume_chapter`: expects exactly "VOL-CHAP" format

New design — single smart sort function:
```rust
pub fn smart_sort(a: &PathBuf, b: &PathBuf) -> Ordering
```

Strategy:
1. Extract filename (or directory name for chapter sorting)
2. Parse into segments: split on common delimiters (`_`, `-`, `.`, ` `)
3. For each segment, classify as numeric or text
4. Compare segment-by-segment (natural sort):
   - Numeric vs numeric: compare as numbers
   - Text vs text: compare lexicographically (case-insensitive)
   - Numeric vs text: numeric comes first
5. Fallback: full string lexicographic comparison

This handles ALL patterns:
- `001.jpg` → [001] → sort by 1
- `007_p007.webp` → [007, p, 007] → sort by 7, then "p", then 7
- `Vol01_Ch03_p015.png` → [Vol, 01, Ch, 03, p, 015] → sort by text "vol", 1, text "ch", 3, text "p", 15
- `1-15.jpg` → [1, 15] → sort by 1, then 15
- `page_1.jpg` → [page, 1] → sort by text "page", then 1

Keep the old functions for backward compat if needed but make `smart_sort` the default everywhere.

### 1D. Update `scanner/src/analyzer.rs`
- No structural changes needed — it already works on `Vec<Vec<PathBuf>>`
- The new collector feeds it the same shape

---

## Phase 2: Common Crate Updates

### 2A. Update `common/src/types.rs`
- Add `Flatten` variant or field:
  ```rust
  // New field in ConvStateKey
  ConvStateKey::Flatten(bool),
  ```
- Add `SourceStructure` enum (or put it in scanner and re-export)

### 2B. Update `common/src/utils.rs`
- Add `is_image_extension(ext: &str) -> bool` helper
- Add `is_zip_file(path: &Path) -> bool` helper (extension check)
- Keep `get_file_info` as-is

### 2C. Update `common/src/error.rs`
- Add `ZipExtraction(String)` error variant for ZIP-specific failures

---

## Phase 3: Tauri Command Updates

### 3A. Update `state.rs`
- Add `flatten: bool` field to `ConvState`
- Add `source_is_zip: bool` field (tracks if source was a ZIP, for cleanup)
- Add `temp_dir: Option<PathBuf>` field (tracks extracted temp dir)

### 3B. Rewrite `commands/analyze.rs`
Current problems:
- 450-line god function
- 11 sequential validation checks that could be parallelized
- Hardcoded to expect subdirectories
- Errors when no subdirectories found (flat structure)

New design:
1. Split into named validator functions:
   ```rust
   fn validate_image_formats(pages: &[PathBuf]) -> ValidationResult
   fn validate_file_sizes(pages: &[PathBuf]) -> ValidationResult
   fn validate_naming_consistency(pages: &[PathBuf]) -> ValidationResult
   fn validate_permissions(chapters: &[PathBuf], pages: &[PathBuf]) -> ValidationResult
   fn validate_path_lengths(pages: &[PathBuf]) -> ValidationResult
   fn validate_special_characters(chapters: &[PathBuf]) -> ValidationResult
   fn validate_directory_naming(chapters: &[PathBuf]) -> ValidationResult
   fn validate_chapter_sizes(chapter_pages: &[Vec<PathBuf>]) -> ValidationResult
   fn check_non_image_files(skipped: &[PathBuf]) -> ValidationResult
   ```
2. Run all validators in parallel using rayon's `par_iter`
3. **ZIP support**: if source is ZIP, extract first, then analyze extracted dir
4. **Flat support**: if `CollectedSource.structure == Flat`, skip chapter-level validations, adjust messaging
5. **Non-image warning**: report skipped_files count and examples from CollectedSource
6. Detect structure and suggest appropriate bundling strategy

### 3C. Update `commands/bundle.rs`
- Use new `Collector::collect()` instead of separate `collect_chapters` + `collect_pages`
- **Flatten support**: if `state.flatten == true`, set `volume_sizes = [total_chapters]` (one volume with all chapters)
- For flat input: `volume_sizes = [1]`, `data = [all_images]`
- Use `smart_sort` as default sorter

### 3D. Update `commands/convert.rs`
- After conversion completes, clean up temp dir if `state.source_is_zip`
- No other structural changes needed — it already works with `Vec<Vec<PathBuf>>`

---

## Phase 4: Performance Optimizations

### 4A. Parallelize analysis validators
- Currently 11 checks run sequentially. Run all in parallel since they're independent reads.

### 4B. Cache grayscale detection results
- During conversion, `convert_to_webp`/`convert_to_avif` calls `is_grayscale` on every image
- If analysis already ran grayscale detection on cover images, cache those results
- Low priority — the distributed strategy is already fast (~10-20ms per image)

### 4C. Remove redundant concurrency control in collector
- Current code: rayon `par_iter` + tokio semaphore + tokio spawn — triple layer
- Simplify to just tokio tasks with semaphore (for async I/O) OR rayon (for CPU)
- Don't mix both for the same operation

### 4D. Eliminate double-collection in analyze
- `conv_analyze` calls `collect_pages` TWICE (line 87 and line 239)
- Fix: collect once, reuse

---

## Phase 5: Code Readability

### 5A. Rename sorting functions
- `sort_by_stem_number` → keep as legacy, mark deprecated
- `sort_name_by_number` → keep as legacy, mark deprecated
- `sort_by_name_volume_chapter` → keep as legacy, mark deprecated
- New: `smart_sort` is the primary function

### 5B. Split god functions
- `conv_analyze`: extract validators as described in 3B
- `conv_convert`: extract volume preparation into helper

### 5C. Remove deprecated field
- `convert_to_webp` in ConvState — it's redundant with `image_format`
- Remove the backward-compat sync logic in state.rs

### 5D. Consolidate constants
- Move magic numbers from analyze.rs into constants (outlier thresholds, path length limits, etc.)

---

## Implementation Order
1. Phase 1C (sorting) — no dependencies, foundation for everything
2. Phase 1B (collector rewrite) — depends on 1C
3. Phase 2 (common crate types/utils) — parallel with 1B
4. Phase 1A (ZIP extractor) — depends on 1B
5. Phase 3B (analyze rewrite) — depends on 1B, 2
6. Phase 3A + 3C (state + bundle) — depends on 1B, 2
7. Phase 3D (convert cleanup) — depends on 3A
8. Phase 4 (optimizations) — after everything works
9. Phase 5 (readability) — ongoing, interleaved

## Files Changed
- **New**: `crates/scanner/src/extractor.rs`
- **Rewrite**: `crates/scanner/src/collector.rs`
- **Rewrite**: `crates/scanner/src/sorting.rs`
- **Update**: `crates/scanner/src/lib.rs`
- **Update**: `crates/scanner/src/constants.rs`
- **Update**: `crates/scanner/Cargo.toml`
- **Update**: `crates/common/src/types.rs`
- **Update**: `crates/common/src/utils.rs`
- **Update**: `crates/common/src/error.rs`
- **Update**: `crates/common/src/lib.rs`
- **Update**: `src-tauri/src/state.rs`
- **Rewrite**: `src-tauri/src/commands/analyze.rs`
- **Update**: `src-tauri/src/commands/bundle.rs`
- **Update**: `src-tauri/src/commands/convert.rs`
- **Update**: `src-tauri/src/commands/state.rs`
