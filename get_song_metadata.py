import os
import requests
from mutagen.mp3 import MP3
from mutagen.id3 import ID3, USLT
from mutagen.flac import FLAC
from urllib.parse import quote

# Directory to scan for songs
MUSIC_DIR = "./music"

# File extensions supported
SUPPORTED_EXTENSIONS = [".mp3", ".flac"]

def get_song_metadata(file_path):
    title, artist, album, duration = None, None, None, None

    if file_path.endswith(".mp3"):
        audio = MP3(file_path, ID3=ID3)
        title_frame = audio.get("TIT2")
        artist_frame = audio.get("TPE1")
        album_frame = audio.get("TALB")

        title = title_frame.text[0] if title_frame else None
        artist = artist_frame.text[0] if artist_frame else None
        album = album_frame.text[0] if album_frame else None
        duration = audio.info.length if audio.info else None

    elif file_path.endswith(".flac"):
        audio = FLAC(file_path)
        title = audio.get("title", [None])[0]
        artist = audio.get("artist", [None])[0]
        album = audio.get("album", [None])[0]
        duration = audio.info.length if audio.info else None

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

    # syncedLyrics is the actual lyrics string, not a URL
    lyrics_text = results[0].get("syncedLyrics")
    if not lyrics_text:
        return None

    return lyrics_text



def embed_lyrics(file_path, lyrics):
    if file_path.endswith(".mp3"):
        audio = MP3(file_path, ID3=ID3)
        try:
            audio.add_tags()
        except Exception:
            pass  # Tags already exist

        audio.tags.setall("USLT", [USLT(encoding=3, lang='eng', desc='', text=lyrics)])
        audio.save()

    elif file_path.endswith(".flac"):
        audio = FLAC(file_path)
        audio["LYRICS"] = lyrics
        audio.save()


def main():
    for root, dirs, files in os.walk(MUSIC_DIR):
        for file in files:
            if not any(file.endswith(ext) for ext in SUPPORTED_EXTENSIONS):
                continue

            full_path = os.path.join(root, file)
            title, artist, album, duration = get_song_metadata(full_path)

            print(f"[DEBUG] File: {file}")
            print(f"[DEBUG] Title: {title}")
            print(f"[DEBUG] Artist: {artist}")
            print(f"[DEBUG] Album: {album}")
            print(f"[DEBUG] Duration: {duration}")

            if not title or not artist:
                print(f"Skipping {file}: missing metadata")
                continue

            print(f"Fetching lyrics for {title} by {artist}")
            lyrics = fetch_synced_lyrics(title, artist, album, duration)
            if lyrics:
                embed_lyrics(full_path, lyrics)
                print(f"✔ Embedded lyrics into {file}")
            else:
                print(f"✘ No lyrics found for {title} by {artist}")


if __name__ == "__main__":
    main()
