# WebP Conversion Feature

## Overview
This document describes the WebP conversion feature that was added to Palaxy. This feature allows users to automatically convert all images to WebP format during the conversion process for EPUB and CBZ files.

## Feature Details

### User-Facing Changes

#### Frontend (UI)
- **New Toggle in Step 3**: A "Convert to WebP" toggle has been added to the Output Settings card in Step 3 of the conversion wizard
- **Default State**: The toggle is enabled by default (images will be converted to WebP)
- **Review Display**: The WebP conversion setting is displayed in Step 6 (Review) for user confirmation before conversion
- **Localization**: Full translation support for English and German languages

### Backend Changes

#### State Management
1. **ConvState Structure** (`src-tauri/src/prelude.rs`)
   - Added `convert_to_webp: bool` field to track the conversion preference
   - Defaults to `true` (enabled)

2. **ConvStateKey Enum** (`src-tauri/src/types.rs`)
   - Added `ConvertToWebp(bool)` variant for state updates

3. **State Handler** (`src-tauri/src/commands/converter.rs`)
   - Added handler case for `ConvertToWebp` in `conv_state_set` function

#### Image Conversion Logic

##### Helper Function: `convert_image_to_webp`
Location: `src-tauri/src/commands/converter.rs`

**Purpose**: Converts images to WebP format if they aren't already in that format

**Behavior**:
- Checks if the image is already in WebP format (skips conversion if true)
- Loads the image using the `image` crate
- Encodes to WebP format
- Saves to a temporary directory
- Returns the path to the converted image

**Performance**:
- Uses `spawn_blocking` for CPU-intensive image operations to avoid blocking async runtime
- Processes images in parallel during volume creation

##### Temporary Directory Management
- Creates `.palaxy_temp` directory in the target output folder when WebP conversion is enabled
- Stores all converted images temporarily during processing
- Automatically cleans up the temporary directory after conversion completes
- Includes error handling for cleanup failures (logged as warnings)

#### Integration Points

##### CBZ Generation (`conv_convert` function)
- Before adding pages to CBZ archives, images are converted if the flag is enabled
- Conversion happens per-page basis within chapter processing
- Original files remain unchanged; only converted copies are added to the archive

##### EPUB Generation (`conv_convert` function)
- Before adding chapters to EPUB files, all chapter images are batch-converted
- Conversion happens per-chapter to maintain proper structure
- Converted images are passed to the `add_chapter` method

##### Concurrent Processing
- Maintains existing parallelization strategy (max 10 concurrent volumes)
- Each volume conversion task handles its own image conversions
- Temp directory is shared across all concurrent tasks

## Technical Implementation

### Dependencies
- **image crate**: Already present in `Cargo.toml` (v0.25)
  - Used for reading various image formats
  - Provides WebP encoding support via the `webp` feature (implicitly enabled)
- **tokio**: Used for async file operations and `spawn_blocking`

### File Structure Changes

```
Backend (Rust):
├── src-tauri/src/commands/converter.rs
│   ├── convert_image_to_webp() - New helper function
│   ├── conv_state_set() - Updated to handle ConvertToWebp
│   └── conv_convert() - Updated to perform conversions
├── src-tauri/src/prelude.rs
│   └── ConvState - Added convert_to_webp field
└── src-tauri/src/types.rs
    └── ConvStateKey - Added ConvertToWebp variant

Frontend (Svelte):
├── src/components/convert/Step3.svelte
│   └── Added WebP conversion toggle
├── src/components/convert/Step6.svelte
│   └── Added WebP conversion display in review
└── src/locales/
    ├── en.po - Added English translations
    └── de.po - Added German translations
```

## Translation Strings

### English
- `"Convert to WebP"` - Toggle label
- `"Images will be converted to WebP format"` - When enabled
- `"Images will keep their original format"` - When disabled

### German
- `"Zu WebP konvertieren"` - Toggle label
- `"Bilder werden in WebP-Format konvertiert"` - When enabled
- `"Bilder behalten ihr ursprüngliches Format"` - When disabled

## Benefits of WebP Conversion

1. **Smaller File Sizes**: WebP typically achieves 25-35% better compression than JPEG/PNG
2. **Quality**: Maintains high visual quality even with better compression
3. **Compatibility**: Widely supported in modern EPUB/CBZ readers
4. **User Control**: Optional feature that can be disabled if needed

## Error Handling

1. **Conversion Failures**: If an image fails to convert, the error is logged and the conversion process stops for that volume
2. **Temp Directory**: If cleanup fails, it's logged as a warning but doesn't fail the overall process
3. **Already WebP**: Images already in WebP format are detected and skipped, avoiding unnecessary processing

## Performance Considerations

- **Blocking Operations**: Image encoding runs in blocking threads to prevent async runtime blocking
- **Parallel Processing**: Multiple volumes are processed in parallel (up to 10 concurrent)
- **Memory**: Uses memory-mapped files for efficient large file handling
- **Cleanup**: Temporary files are cleaned up automatically after conversion

## Future Enhancements (Potential)

1. **Quality Settings**: Allow users to specify WebP quality/compression level
2. **Selective Conversion**: Convert only specific image types (e.g., only PNG files)
3. **Preview**: Show estimated space savings before conversion
4. **Progress**: Display conversion progress for individual images
5. **Lossy/Lossless**: Option to choose between lossy and lossless WebP encoding

## Testing Recommendations

1. Test with various image formats (JPEG, PNG, already WebP)
2. Test with large volumes (100+ images) to verify cleanup
3. Test with both EPUB and CBZ output formats
4. Test with concurrent volume processing
5. Verify translations display correctly in both languages
6. Test error scenarios (corrupted images, permission issues)

## Notes

- The feature is enabled by default to provide immediate benefits to users
- Original source files are never modified; only the output contains converted images
- The conversion happens during the final conversion step, not during bundling
- Temporary directory uses a hidden folder (`.palaxy_temp`) to avoid clutter
