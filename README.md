
# Lyrics Downloader

This project is a simple tool for downloading synced lyrics for music files in your collection. The program scans through your music folder, identifies supported formats (e.g., `.mp3` and `.flac`), retrieves metadata (title and artist), and fetches synced lyrics from an API. The lyrics are then saved as `.lrc` files in the same folder as the original music files.
![image](https://github.com/user-attachments/assets/22ba6b82-189f-4ef2-8aa8-a5391b9fc458)

## Features

- Scans a selected folder for music files (`.mp3` and `.flac` supported).
- Fetches synced lyrics using an API.
- Saves lyrics as `.lrc` files alongside the music files.
- User-friendly graphical interface built with `eframe`.

## Requirements

- Rust 2021 edition
- Dependencies: `walkdir`, `reqwest`, `serde`, `lofty`, `eframe`, `rfd`

## Setup
## Prerequisites

Before you can run this project, you need to install the following:

### 1. **Install Cargo (Rust Package Manager)**

To get started with Rust and Cargo (the Rust package manager), follow these steps:

- **Install Rust:**

  - Go to the official Rust installation page: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
  - Follow the instructions for your operating system.
  - After installation, verify it by running the following commands in your terminal:

    ```bash
    rustc --version
    cargo --version
    ```

  - This will install both `rustc` (Rust Compiler) and `cargo` (Rust Package Manager).

- **Using Cargo**:
  Cargo is the Rust package manager, and it will help you download dependencies and build the project.
### 2. Clone the project 
1. Clone the repository:
   
   ```bash
   git clone https://github.com/caberfan/LyricsDownloader.git
   ```

2. Change directory to the project folder:

   ```bash
   cd LyricsDownloader
   ```
### 3. Build the project
Install the dependencies:

   ```bash
   cargo build
   ```

### 4. Run the program:

   ```bash
   cargo run
   ```

## Usage

* Click the "Select Folder" button to choose your music folder.
* Click "Start Processing" to scan for music files and download the lyrics.

## Disclaimer

In the moment I am still too lazy to add a digital signature, but there's no virus, go ahead and run it.

Feel free to modify the content as necessary!
