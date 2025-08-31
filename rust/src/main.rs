#![windows_subsystem = "windows"]

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;
use reqwest::blocking::get;
use serde::Deserialize;
use eframe::egui;
use lofty::{read_from_path, ItemKey, TaggedFileExt};
use eframe::IconData;
use image;


#[derive(Debug, Deserialize)]
struct LyricsResult {
    #[serde(rename = "syncedLyrics")]
    syncedLyrics: Option<String>,
}

/// Reads metadata from a file.
///
/// This function will first attempt to read the "TrackTitle" and "TrackArtist" tags from the file using the `lofty` crate.
/// If this fails, it will then attempt to split the file name into an artist and title by splitting on " - ".
/// If this fails (for example, if the file name does not contain " - "), the function will return (None, None).
fn get_metadata(path: &PathBuf) -> (Option<String>, Option<String>) {
    if let Ok(tagged_file) = read_from_path(path) {
        let tag = tagged_file.primary_tag();
        let title = tag.and_then(|t| t.get_string(&ItemKey::TrackTitle).map(|s| s.to_string()));
        let artist = tag.and_then(|t| t.get_string(&ItemKey::TrackArtist).map(|s| s.to_string()));
        return (title, artist);
    }

    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
        if let Some((artist, title)) = name.split_once(" - ") {
            return (Some(title.trim().to_string()), Some(artist.trim().to_string()));
        } else {
            return (Some(name.to_string()), None);
        }
    }

    (None, None)
}

/// Fetches the lyrics for a given song from lrclib.net.
///
/// Will return None if the API request fails, or if the response does not
/// contain a LyricsResult with syncedLyrics.
fn fetch_lyrics(title: &str, artist: &str) -> Option<String> {
    let url = format!(
        "https://lrclib.net/api/search?track_name={}&artist_name={}",
        urlencoding::encode(title),
        urlencoding::encode(artist)
    );

    if let Ok(resp) = get(&url) {
        if let Ok(json) = resp.json::<Vec<LyricsResult>>() {
            if let Some(result) = json.first() {
                return result.syncedLyrics.clone();
            }
        }
    }

    None
}

fn write_lrc(path: &PathBuf, lyrics: &str) {
    let lrc_path = path.with_extension("lrc"); // removed mut
    if let Ok(mut file) = File::create(&lrc_path) {
        let _ = file.write_all(lyrics.as_bytes());
    }
}

/// Main entry point of the program.
///
/// This function will create an egui-native window with the given title,
/// and will set up the icon for that window. It will then create a
/// `LyricsApp` instance and pass it to `eframe::run_native` to start
/// the event loop.
fn main() -> eframe::Result<()> {
    let icon = {
        let icon_bytes = include_bytes!("../icon.png");
        let image = image::load_from_memory(icon_bytes).expect("Failed to load icon").into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        IconData { rgba, width, height }
    };

    let options = eframe::NativeOptions {
        icon_data: Some(icon),
        ..Default::default()
    };
    eframe::run_native("Lyrics Downloader", options, Box::new(|_cc| Box::<LyricsApp>::default()))
}
struct LyricsApp {
    folder: Option<PathBuf>,
    scanned: Arc<Mutex<usize>>,
    written: Arc<Mutex<usize>>,
    processing: Arc<Mutex<bool>>,
    logs: Arc<Mutex<Vec<String>>>, // Add this field
}

impl Default for LyricsApp {
    fn default() -> Self {
        Self {
            folder: None,
            scanned: Arc::new(Mutex::new(0)),
            written: Arc::new(Mutex::new(0)),
            processing: Arc::new(Mutex::new(false)),
            logs: Arc::new(Mutex::new(Vec::new())), // Initialize logs
        }
    }
}

// Update process_folder to accept logs
fn process_folder(folder: &PathBuf, logs: Arc<Mutex<Vec<String>>>) -> (usize, usize) {
    let mut scanned = 0;
    let mut written = 0;

    for entry in WalkDir::new(folder).into_iter().filter_map(Result::ok) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext == "mp3" || ext == "flac" {
                scanned += 1;
                let (title_opt, artist_opt) = get_metadata(&path);

                logs.lock().unwrap().push(format!("[DEBUG] File: {}", path.display()));
                logs.lock().unwrap().push(format!("[DEBUG] Title: {:?}", title_opt));
                logs.lock().unwrap().push(format!("[DEBUG] Artist: {:?}", artist_opt));

                if title_opt.is_none() || artist_opt.is_none() {
                    logs.lock().unwrap().push(format!("❌ Skipping {}: missing metadata", path.display()));
                    continue;
                }

                let title = title_opt.unwrap();
                let artist = artist_opt.unwrap();

                logs.lock().unwrap().push(format!("Fetching lyrics for {} by {}", title, artist));
                if let Some(lyrics) = fetch_lyrics(&title, &artist) {
                    write_lrc(&path, &lyrics);
                    written += 1;
                    logs.lock().unwrap().push(format!("✔ Saved lyrics to {}.lrc", path.with_extension("lrc").file_name().unwrap().to_string_lossy()));
                } else {
                    logs.lock().unwrap().push(format!("✘ No lyrics found for {} by {}", title, artist));
                }
                logs.lock().unwrap().push(format!("🔍 File number: {}", scanned));
                logs.lock().unwrap().push(format!("✅ Files with lyrics: {}", written));
            }
        }
    }

    logs.lock().unwrap().push(format!("\n[INFO] Lyrics written for {} files.", written));
    logs.lock().unwrap().push(format!("[INFO] Scanned {} files in total.", scanned));

    (scanned, written)
}

/// Processes a folder to embed lyrics into audio files.
///
/// This function scans the specified `folder` for audio files with `.mp3` or `.flac` extensions,
/// attempts to fetch lyrics for each file based on its metadata, and embeds the lyrics into the
/// file if found. The process is logged using the provided `logs` Arc<Mutex<Vec<String>>>.
///
/// # Arguments
///
/// * `folder` - A reference to the folder path to be scanned for audio files.
/// * `logs` - A thread-safe vector for logging messages during the processing.
///
/// # Returns
///
/// A tuple containing:
/// * `usize` - The total number of files scanned.
/// * `usize` - The number of files into which lyrics were successfully embedded.

fn process_folder_embed(folder: &PathBuf, logs: Arc<Mutex<Vec<String>>>) -> (usize, usize) {
    let mut scanned = 0;
    let mut embedded = 0;

    for entry in WalkDir::new(folder).into_iter().filter_map(Result::ok) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext == "mp3" || ext == "flac" {
                scanned += 1;
                let (title_opt, artist_opt) = get_metadata(&path);

                logs.lock().unwrap().push(format!("[DEBUG] File: {}", path.display()));
                logs.lock().unwrap().push(format!("[DEBUG] Title: {:?}", title_opt));
                logs.lock().unwrap().push(format!("[DEBUG] Artist: {:?}", artist_opt));

                if title_opt.is_none() || artist_opt.is_none() {
                    logs.lock().unwrap().push(format!("❌ Skipping {}: missing metadata", path.display()));
                    continue;
                }

                let title = title_opt.unwrap();
                let artist = artist_opt.unwrap();

                logs.lock().unwrap().push(format!("Fetching lyrics for {} by {}", title, artist));
                if let Some(lyrics) = fetch_lyrics(&title, &artist) {
                    if embed_lyrics(&path, &lyrics, &ext, &logs) {
                        embedded += 1;
                        logs.lock().unwrap().push(format!("💾 Embedded lyrics into {}", path.file_name().unwrap().to_string_lossy()));
                    } else {
                        logs.lock().unwrap().push(format!("❌ Failed to embed lyrics into {}", path.display()));
                    }
                } else {
                    logs.lock().unwrap().push(format!("✘ No lyrics found for {} by {}", title, artist));
                }
                logs.lock().unwrap().push(format!("🔍 File number: {}", scanned));
                logs.lock().unwrap().push(format!("✅ Files with lyrics embedded: {}", embedded));
            }
        }
    }

    logs.lock().unwrap().push(format!("\n[INFO] Lyrics embedded in {} files.", embedded));
    logs.lock().unwrap().push(format!("[INFO] Scanned {} files in total.", scanned));

    (scanned, embedded)
}

    /// Embed lyrics in a file.
    ///
    /// This function takes a file path, some lyrics, the file extension, and a reference to a vector of log messages.
    /// It uses the `lofty` crate to read the file as a `TaggedFile`, and then attempts to embed the lyrics in a tag.
    /// If the tag does not exist, it is created.
    /// If the file cannot be opened or saved, an error is logged and the function returns `false`.
    /// If the tag cannot be read or written, an error is logged and the function returns `false`.
    ///
    /// The function returns `true` if the lyrics were successfully embedded, and `false` otherwise.
fn embed_lyrics(path: &PathBuf, lyrics: &str, ext: &str, logs: &Arc<Mutex<Vec<String>>>) -> bool {
    use lofty::{TagType, ItemKey, TaggedFileExt, AudioFile, Tag};

    match lofty::read_from_path(path) {
        Ok(mut tagged_file) => {
            let tag_type = if ext == "mp3" { TagType::Id3v2 } else { TagType::VorbisComments };
            // Ensure the tag exists
            if tagged_file.tag_mut(tag_type).is_none() {
                // Create a new tag of the correct type and insert it
                let new_tag = Tag::new(tag_type);
                tagged_file.insert_tag(new_tag);
            }
            // Now get a mutable reference
            if let Some(tag) = tagged_file.tag_mut(tag_type) {
                tag.insert_text(ItemKey::Lyrics, lyrics.to_string());
            } else {
                logs.lock().unwrap().push(format!("❌ Could not get or create tag for embedding lyrics."));
                return false;
            }

            // Save the tags back to the file
            if let Err(e) = tagged_file.save_to_path(path) {
                logs.lock().unwrap().push(format!("❌ Failed to embed lyrics: {}", e));
                return false;
            }
            true
        }
        Err(e) => {
            logs.lock().unwrap().push(format!("❌ Failed to open file for embedding: {}", e));
            false
        }
    }
}


impl eframe::App for LyricsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set background to white (light theme)
        ctx.set_visuals(egui::Visuals::light());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Lyrics Downloader");
            // Select folder button
            if ui.button("Select Folder").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.folder = Some(path);
                }
            }
            // Show the selected folder path
            if let Some(folder) = &self.folder {
                ui.label(format!("Selected folder: {}", folder.display()));
            }
            let processing = *self.processing.lock().unwrap();
            // Add buttons for processing
            if ui.button("Add .lrc files").clicked() && !processing {
                if let Some(folder) = self.folder.clone() {
                    let scanned = Arc::clone(&self.scanned);
                    let written = Arc::clone(&self.written);
                    let processing = Arc::clone(&self.processing);
                    let ctx = ctx.clone();
                    let logs = Arc::clone(&self.logs);

                    *processing.lock().unwrap() = true;
                    logs.lock().unwrap().clear(); // Clear logs before new run
                    thread::spawn(move || {
                        let result = process_folder(&folder, logs);
                        *scanned.lock().unwrap() = result.0;
                        *written.lock().unwrap() = result.1;
                        *processing.lock().unwrap() = false;
                        ctx.request_repaint();
                    });
                }
            }
            // Add button for embedding lyrics
            if ui.button("Embed Lyrics").clicked() && !processing {
                if let Some(folder) = self.folder.clone() {
                    let scanned = Arc::clone(&self.scanned);
                    let written = Arc::clone(&self.written);
                    let processing = Arc::clone(&self.processing);
                    let ctx = ctx.clone();
                    let logs = Arc::clone(&self.logs);
            
                    *processing.lock().unwrap() = true;
                    logs.lock().unwrap().clear();
                    thread::spawn(move || {
                        let result = process_folder_embed(&folder, logs);
                        *scanned.lock().unwrap() = result.0;
                        *written.lock().unwrap() = result.1;
                        *processing.lock().unwrap() = false;
                        ctx.request_repaint();
                    });
                }
            }
            // Show processing status
            if processing {
                ui.label("Processing...");
            } else if *self.scanned.lock().unwrap() > 0 {
                ui.label(format!("Scanned: {}", *self.scanned.lock().unwrap()));
                ui.label(format!("Lyrics written: {}", *self.written.lock().unwrap()));
            }

            // Show logs in a scrollable area
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                let logs = self.logs.lock().unwrap();
                for log in logs.iter() {
                    ui.label(log);
                }
                // Add an invisible widget and scroll to it
                ui.add_space(0.0); // Ensures the cursor is at the end
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            });
        });
    }
}