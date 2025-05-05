import os
import requests
from urllib.parse import quote
from mutagen.mp3 import MP3
from mutagen.id3 import ID3
from mutagen.flac import FLAC
from mutagen.mp4 import MP4
from mutagen.oggvorbis import OggVorbis
from mutagen.aiff import AIFF
from mutagen.wave import WAVE
from mutagen.dsf import DSF

# Directory to scan for songs
MUSIC_DIR = "X:\Music\Loseless_musics\Maneskin"

# File extensions supported
SUPPORTED_EXTENSIONS = [".mp3", ".flac"]

def get_song_metadata(file_path):
    title, artist, album, duration = None, None, None, None
    ext = os.path.splitext(file_path)[1].lower()

    try:
        if ext == ".mp3":
            audio = MP3(file_path, ID3=ID3)
            title = audio.get("TIT2", [None])[0]
            artist = audio.get("TPE1", [None])[0]
            album = audio.get("TALB", [None])[0]
            duration = audio.info.length

        elif ext == ".flac":
            audio = FLAC(file_path)
            title = audio.get("title", [None])[0]
            artist = audio.get("artist", [None])[0]
            album = audio.get("album", [None])[0]
            duration = audio.info.length

        elif ext == ".m4a":
            audio = MP4(file_path)
            title = audio.tags.get("\xa9nam", [None])[0]
            artist = audio.tags.get("\xa9ART", [None])[0]
            album = audio.tags.get("\xa9alb", [None])[0]
            duration = audio.info.length

        elif ext == ".ogg":
            audio = OggVorbis(file_path)
            title = audio.get("title", [None])[0]
            artist = audio.get("artist", [None])[0]
            album = audio.get("album", [None])[0]
            duration = audio.info.length

        elif ext == ".aiff":
            audio = AIFF(file_path)
            title = audio.get("TIT2", [None])[0]
            artist = audio.get("TPE1", [None])[0]
            album = audio.get("TALB", [None])[0]
            duration = audio.info.length

        elif ext == ".wav":
            audio = WAVE(file_path)
            duration = audio.info.length
            # WAV files often lack tags

        elif ext == ".dsf":
            audio = DSF(file_path)
            title = audio.get("title", [None])[0]
            artist = audio.get("artist", [None])[0]
            album = audio.get("album", [None])[0]
            duration = audio.info.length

        elif ext == ".dsd":
            # mutagen doesn't support .dsd — use filename fallback
            pass

    except Exception as e:
        print(f"[WARNING] Could not parse tags from {file_path}: {e}")

    # Fallback: try to parse from filename
    if not title or not artist:
        base = os.path.basename(file_path)
        name, _ = os.path.splitext(base)
        if " - " in name:
            parts = name.split(" - ", 1)
            artist = artist or parts[0].strip()
            title = title or parts[1].strip()
        else:
            title = title or name

    return title, artist, album, duration


def fetch_synced_lyrics(title, artist, album=None, duration=None):
    search_url = f"https://lrclib.net/api/search?track_name={quote(title)}&artist_name={quote(artist)}"
    resp = requests.get(search_url)
    
    if resp.status_code != 200:
        return None

    results = resp.json()

    if not results:
        # Try fallback query
        fallback_url = f"https://lrclib.net/api/search?q={quote(title + ' ' + artist)}"
        resp = requests.get(fallback_url)
        if resp.status_code != 200:
            return None
        results = resp.json()

    if not results:
        return None

    lyrics_text = results[0].get("syncedLyrics")
    if not lyrics_text:
        return None

    return lyrics_text


def write_lrc_file(song_path, lyrics_text):
    base_name = os.path.splitext(song_path)[0]
    lrc_path = base_name + ".lrc"
    with open(lrc_path, "w", encoding="utf-8") as f:
        f.write(lyrics_text)
    print(f"✔ Saved lyrics to {os.path.basename(lrc_path)}")


def main():
    lyrics_written_files_count = 0  # Initialize counter for files with lyrics
    scanned_files_count = 0  # Initialize counter for all scanned files (with or without lyrics)

    for root, dirs, files in os.walk(MUSIC_DIR):
        for file in files:
            if not any(file.endswith(ext) for ext in SUPPORTED_EXTENSIONS):
                continue

            scanned_files_count += 1  # Increment scanned files count for every file lyrics_written

            full_path = os.path.join(root, file)
            title, artist, album, duration = get_song_metadata(full_path)

            print(f"[DEBUG] File: {file}")
            print(f"[DEBUG] Title: {title}")
            print(f"[DEBUG] Artist: {artist}")
            print(f"[DEBUG] Album: {album}")
            print(f"[DEBUG] Duration: {duration}")

            if not title or not artist:
                print(f"❌ Skipping {file}: missing metadata")
                continue

            print(f"Fetching lyrics for {title} by {artist}")
            lyrics = fetch_synced_lyrics(title, artist, album, duration)
            if lyrics:
                write_lrc_file(full_path, lyrics)
                lyrics_written_files_count += 1  # Increment lyrics_written files count (files with lyrics)
            else:
                print(f"✘ No lyrics found for {title} by {artist}")
            print(f"🔍 File number: {scanned_files_count}")
            print(f"✅ Files with lyrics: {lyrics_written_files_count}")

    # Final output with both counts
    print(f"\n[INFO] Lyrics written {lyrics_written_files_count} files with lyrics.")
    print(f"[INFO] Scanned {scanned_files_count} files in total.")  # Total files processed




if __name__ == "__main__":
    main()
