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
    if file_path.endswith(".mp3"):
        audio = MP3(file_path)
        title_frame = audio.get("TIT2")
        artist_frame = audio.get("TPE1")
        title = title_frame.text[0] if title_frame and title_frame.text else None
        artist = artist_frame.text[0] if artist_frame and artist_frame.text else None
    elif file_path.endswith(".flac"):
        audio = FLAC(file_path)
        title = audio.get("title", [None])[0]
        artist = audio.get("artist", [None])[0]
    else:
        return None, None
    return title, artist


def fetch_synced_lyrics(title, artist):
    search_url = f"https://lrclib.net/api/search?track_name={quote(title)}&artist_name={quote(artist)}"
    resp = requests.get(search_url)
    if resp.status_code != 200:
        return None

    results = resp.json()
    if not results:
        return None

    # Get the first result's syncedLyrics URL
    synced_lyrics = results[0].get("syncedLyrics")
    if not synced_lyrics or "url" not in synced_lyrics:
        return None

    lyrics_url = synced_lyrics["url"]
    lyrics_resp = requests.get(lyrics_url)
    if lyrics_resp.status_code != 200:
        return None

    return lyrics_resp.text


def embed_lyrics(file_path, lyrics):
    if file_path.endswith(".mp3"):
        audio = MP3(file_path, ID3=ID3)
        try:
            audio.add_tags()
        except Exception:
            pass  # tags already exist

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
            title, artist = get_song_metadata(full_path)
            if not title or not artist:
                print(f"Skipping {file}: missing metadata")
                continue

            print(f"Fetching lyrics for {title} by {artist}")
            lyrics = fetch_synced_lyrics(title, artist)
            if lyrics:
                embed_lyrics(full_path, lyrics)
                print(f"✔ Embedded lyrics into {file}")
            else:
                print(f"✘ No lyrics found for {title} by {artist}")

if __name__ == "__main__":
    main()
