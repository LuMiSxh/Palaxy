use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;
use tauri::async_runtime::{JoinHandle, spawn};
use tokio::fs::{create_dir};

use crate::collector::Collector;
use crate::generator::{cbz, epub, pdf};
use crate::prelude::*;

#[tauri::command(async)]
pub async fn analyze(
    source_directory: String,
    sensibility: usize,
) -> Result<AnalyzeResult, Error> {
    let now = std::time::Instant::now();

    let mut collector = Collector::new(&source_directory);
    let calculated_sensibility = sensibility as f64 / 100.0;

    let chapters: Vec<PathBuf> = collector.collect_chapters().await?;
    let chapter_total = chapters.len();
    let chapter_pages: Vec<Vec<PathBuf>> = collector.collect_pages(chapters).await?;
    let book_start_chapters: Vec<usize> = collector.collect_volume_start_chapters(chapter_pages, calculated_sensibility).await?;
    let calculated_book_chapters = collector.calculate_book_chapters(book_start_chapters, chapter_total)?;

    let elapsed = now.elapsed();

    Ok(AnalyzeResult {
        message: format!("Finished in: {:.2?}", elapsed),
        chapter_per_volume: calculated_book_chapters,
    })
}

#[tauri::command(async)]
pub async fn convert(
    source_directory: String,
    target_directory: String,
    chapter_per_volume: Vec<usize>,
    file_format: FileFormat,
    direction: Direction,
) -> Result<ConvertResult, Error> {
    let now = std::time::Instant::now();

    let mut collector = Collector::new(&source_directory);
    let page_directories: Vec<PathBuf> = collector.collect_chapters().await?;
    let chapter_pages: Vec<Vec<PathBuf>> = collector.collect_pages(page_directories).await?;

    let manga_name: String = Path::new(&source_directory)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    
    // Create target folder if it doesn't exist
    let target_directory_full_path = Path::new(&target_directory).join(&manga_name);
    if !target_directory_full_path.exists() {
        create_dir(&target_directory_full_path).await?;
    }
    
    let bc = chapter_per_volume.clone();

    // Arc is a thread-safe reference-counting pointer, used to share ownership between threads
    let book_chapters = Arc::new(chapter_per_volume);
    let images_per_chapter = Arc::new(chapter_pages);
    let manga_name: Arc<String> = Arc::new(manga_name);
    let manga_save: Arc<String> = Arc::new(target_directory_full_path.to_string_lossy().into_owned());

    let handles: Vec<JoinHandle<Result<(), Error>>> =
        bc.into_par_iter().enumerate()
            .map(|(i, book_chapter)| {
                let book_chapters = Arc::clone(&book_chapters);
                let images_per_chapter = Arc::clone(&images_per_chapter);
                let manga_name = Arc::clone(&manga_name);
                let manga_save = Arc::clone(&manga_save);

                spawn(async move {
                    let j: usize = book_chapters[0..i].par_iter().sum();

                    let volume_name = format!("{} | {}", manga_name, i + 1);

                    match file_format {
                        FileFormat::CBZ => {
                            let mut cbz = cbz::Cbz::new(&manga_save, &volume_name)?;

                            for k in j..(j + book_chapter) {
                                for image in &images_per_chapter[k] {
                                    cbz.add_page(image)?;
                                }
                            }

                            cbz.set_comicinfo(&volume_name, i + 1)?.save()?;
                        }
                        FileFormat::EPUB => {
                            let mut epub = epub::EPub::new()?;

                            epub
                                .set_cover(&images_per_chapter[j][0])?
                                .set_lang("en")?
                                .set_metadata("title", &volume_name)?
                                .set_metadata("author", "Manga Bundler")?
                                .set_metadata("direction", if direction == Direction::LTR { "ltr" } else { "rtl" })?;

                            for k in j..(j + book_chapter) {
                                epub.add_chapter(k + 1, &images_per_chapter[k]).await?;
                            }

                            epub.save(&manga_save, format!("{}", volume_name).as_str()).await?;
                        }
                        FileFormat::PDF => {
                            let mut pdf = pdf::Pdf::new(&volume_name, &images_per_chapter[j][0])?;

                            for k in (j + 1)..(j + book_chapter) {
                                for image in &images_per_chapter[k] {
                                    pdf.add_page(image)?;
                                }
                            }

                            pdf.save(&manga_save, &volume_name)?;
                        }
                    }

                    Ok(())
                })
            }).collect();

    for handle in handles {
        handle.await??;
    }

    let elapsed = now.elapsed();
    Ok(ConvertResult {
        message: format!("Finished in: {:.2?}", elapsed),
    })
}
