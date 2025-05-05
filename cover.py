import os
from mutagen.mp3 import MP3
from mutagen.id3 import ID3, APIC
from mutagen.flac import FLAC
from mutagen.mp4 import MP4, MP4Cover
from mutagen.oggvorbis import OggVorbis
from mutagen.aiff import AIFF
MUSIC_DIR = "X:\Music\Loseless_musics"
def extract_cover_image(file_path):
    ext = os.path.splitext(file_path)[1].lower()
    output_path = os.path.join(os.path.dirname(file_path), "cover.jpg")

    try:
        if ext == ".mp3":
            audio = MP3(file_path, ID3=ID3)
            for tag in audio.tags.values():
                if isinstance(tag, APIC):
                    with open(output_path, "wb") as img:
                        img.write(tag.data)
                    print(f"✅ Extracted cover from {file_path}")
                    return

        elif ext == ".flac":
            audio = FLAC(file_path)
            if audio.pictures:
                with open(output_path, "wb") as img:
                    img.write(audio.pictures[0].data)
                print(f"✅ Extracted cover from {file_path}")
                return

        elif ext == ".m4a":
            audio = MP4(file_path)
            if 'covr' in audio:
                cover = audio['covr'][0]
                if isinstance(cover, MP4Cover):
                    with open(output_path, "wb") as img:
                        img.write(cover)
                    print(f"✅ Extracted cover from {file_path}")
                    return

        elif ext == ".ogg":
            # OGG usually doesn't store covers in a standard way
            print(f"⚠️ Cover extraction for OGG is not supported.")

        elif ext == ".aiff":
            audio = AIFF(file_path)
            for tag in audio.tags.values():
                if isinstance(tag, APIC):
                    with open(output_path, "wb") as img:
                        img.write(tag.data)
                    print(f"✅ Extracted cover from {file_path}")
                    return

        else:
            print(f"❌ Unsupported format: {file_path}")
            return

        print(f"🚫 No cover found in {file_path}")

    except Exception as e:
        print(f"❌ Error processing {file_path}: {e}")


# Example usage:
if __name__ == "__main__":
    for root, dirs, files in os.walk(MUSIC_DIR):
        for file in files:
            if file.lower().endswith((".mp3", ".flac", ".m4a", ".aiff")):
                extract_cover_image(os.path.join(root, file))