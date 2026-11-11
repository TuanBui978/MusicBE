import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import html as html_lib
import json
import re
import ssl
import threading
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from db import (
    BASE_DIR,
    get_connection,
    init_db,
    link_album_song,
    link_song_artist,
    upsert_album,
    upsert_artist,
    upsert_song,
)

STORAGE_DIR = BASE_DIR / "storage"
AUDIO_DIR = STORAGE_DIR / "audio"
LYRICS_DIR = STORAGE_DIR / "lyrics"
SONG_COVER_DIR = STORAGE_DIR / "covers" / "songs"
ALBUM_COVER_DIR = STORAGE_DIR / "covers" / "albums"
ARTIST_COVER_DIR = STORAGE_DIR / "covers" / "artists"

USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
)

# Danh sách Playlist mẫu trên NhacCuaTui (Hợp cho cả UI lẫn AI Drum Pad)
DEFAULT_PLAYLISTS = [
    {
        "key": "m3liaiy6vVsF",
        "genre": "VPOP",
        "type": "CHART",
        "note": "Top 100 Nhạc Trẻ Hay Nhất",
    },
    {
        "key": "iY1AnIsXedqE",
        "genre": "RAP",
        "type": "CHART",
        "note": "Top 100 Rap Việt Hay Nhất",
    },
    {
        "key": "aY3KIEnpCywU",
        "genre": "EDM",
        "type": "CHART",
        "note": "Top 100 Remix Việt Hay Nhất",
    },
    {
        "key": "zE23R7bc8e9X",
        "genre": "USUK",
        "type": "CHART",
        "note": "Top 100 Pop USUK Hay Nhất",
    },
    {
        "key": "ippIsiqacmnE",
        "genre": "EDM",
        "type": "CHART",
        "note": "Top 100 Electronica/Dance Hay Nhất",
    },
    {
        "key": "bxkenI7MAoFv",
        "genre": "HIPHOP",
        "type": "CHART",
        "note": "Top 100 Rap/Hip Hop Hay Nhất",
    },
    {
        "key": "JThboxrhG2TV",
        "genre": "LOFI",
        "type": "PLAYLIST",
        "note": "Lofi Chill Cho Ngày Mưa",
    },
    {
        "key": "1706159682ioF3",
        "genre": "INDIE",
        "type": "PLAYLIST",
        "note": "Indie Việt",
    },
]

SSL_CONTEXT = ssl._create_unverified_context()


def ensure_dirs() -> None:
    for d in (
        AUDIO_DIR,
        LYRICS_DIR,
        SONG_COVER_DIR,
        ALBUM_COVER_DIR,
        ARTIST_COVER_DIR,
    ):
        d.mkdir(parents=True, exist_ok=True)


def http_get_bytes(url: str, timeout: int = 20) -> Optional[bytes]:
    if not url:
        return None
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(req, timeout=timeout, context=SSL_CONTEXT) as resp:
            return resp.read()
    except Exception as exc:
        print(f"  [WARN] Lỗi tải URL {url}: {exc}")
        return None


def download_file(url: str, dest_path: Path, overwrite: bool = False) -> Optional[str]:
    if not url:
        return None
    if dest_path.exists() and dest_path.stat().st_size > 0 and not overwrite:
        return str(dest_path.relative_to(BASE_DIR))

    data = http_get_bytes(url, timeout=30)
    if not data:
        return None

    dest_path.parent.mkdir(parents=True, exist_ok=True)
    dest_path.write_bytes(data)
    return str(dest_path.relative_to(BASE_DIR))


def upgrade_image_url(url: Optional[str]) -> Optional[str]:
    """Đổi ảnh _150 / _300 của NCT lên bản nét hơn (_500 hoặc gốc)."""
    if not url:
        return None
    return re.sub(r"_(150|300)\.(jpg|jpeg|png|webp)$", r"_500.\2", url, flags=re.I)


def download_image_smart(url: Optional[str], dest_path: Path) -> Optional[str]:
    if not url:
        return None
    hi_res = upgrade_image_url(url)
    if hi_res and hi_res != url:
        saved = download_file(hi_res, dest_path)
        if saved:
            return saved
    return download_file(url, dest_path)


def extract_nuxt_data(html_text: str) -> Dict[str, Any]:
    """Giải mã mảng Nuxt 3 ShallowReactive JSON được nhúng trong trang NCT."""
    scripts = re.findall(r"<script[^>]*>(.*?)</script>", html_text, re.S)
    raw: Optional[List[Any]] = None
    for s in scripts:
        s_strip = s.strip()
        if s_strip.startswith('[["ShallowReactive"') or s_strip.startswith('[["Reactive"'):
            raw = json.loads(s_strip)
            break

    if not raw:
        return {}

    memo: Dict[int, Any] = {}

    def resolve(idx: Any, depth: int = 0) -> Any:
        if depth > 12 or not isinstance(idx, int) or idx < 0 or idx >= len(raw):
            return idx
        if idx in memo:
            return memo[idx]

        val = raw[idx]
        if isinstance(val, list):
            if (
                len(val) == 2
                and isinstance(val[0], str)
                and val[0]
                in (
                    "ShallowReactive",
                    "Reactive",
                    "Ref",
                    "EmptyRef",
                    "EmptyShallowRef",
                )
            ):
                res = resolve(val[1], depth)
                memo[idx] = res
                return res
            res = [resolve(x, depth + 1) for x in val]
            memo[idx] = res
            return res

        if isinstance(val, dict):
            res = {k: resolve(v, depth + 1) for k, v in val.items()}
            memo[idx] = res
            return res

        return val

    root = resolve(0)
    if isinstance(root, dict) and isinstance(root.get("data"), dict):
        return root["data"]
    return {}


def rc4_decrypt_hex(hex_str: str, key_str: str) -> str:
    """Giải mã file .lrc mã hóa RC4 hex của NhacCuaTui."""
    data = bytes.fromhex(hex_str.strip())
    key = key_str.encode("utf-8")
    s = list(range(256))
    j = 0
    for i in range(256):
        j = (j + s[i] + key[i % len(key)]) % 256
        s[i], s[j] = s[j], s[i]

    i = j = 0
    out = bytearray()
    for byte in data:
        i = (i + 1) % 256
        j = (j + s[i]) % 256
        s[i], s[j] = s[j], s[i]
        out.append(byte ^ s[(s[i] + s[j]) % 256])

    return out.decode("utf-8", errors="ignore")


def clean_html_text(raw_html: Optional[str]) -> Optional[str]:
    if not raw_html:
        return None
    text = re.sub(r"<br\s*/?>", "\n", raw_html, flags=re.I)
    text = re.sub(r"</p>", "\n\n", text, flags=re.I)
    text = re.sub(r"<[^>]+>", "", text)
    text = html_lib.unescape(text)
    lines = [line.strip() for line in text.splitlines()]
    cleaned = "\n".join(line for line in lines if line)
    return cleaned or None


def fetch_and_save_Lyrics(song_key: str, lyric_detail: Optional[Dict[str, Any]]) -> Optional[str]:
    if not isinstance(lyric_detail, dict):
        return None

    lrc_dest = LYRICS_DIR / f"{song_key}.lrc"
    if lrc_dest.exists() and lrc_dest.stat().st_size > 0:
        return str(lrc_dest.relative_to(BASE_DIR))

    timed_url = lyric_detail.get("timedLyric")
    decrypt_key = lyric_detail.get("keyDecryptLyric") or "Lyr1cjust4nct"

    if isinstance(timed_url, str) and timed_url.startswith("http"):
        raw_bytes = http_get_bytes(timed_url)
        if raw_bytes:
            raw_text = raw_bytes.decode("utf-8", errors="ignore").strip()
            try:
                if re.fullmatch(r"[0-9A-Fa-f]+", raw_text):
                    lrc_content = rc4_decrypt_hex(raw_text, decrypt_key)
                else:
                    lrc_content = raw_text
                if lrc_content.strip():
                    lrc_dest.write_text(lrc_content, encoding="utf-8")
                    return str(lrc_dest.relative_to(BASE_DIR))
            except Exception as exc:
                print(f"  [WARN] Giải mã LRC thất bại cho {song_key}: {exc}")

    plain_content = lyric_detail.get("content")
    if isinstance(plain_content, str) and plain_content.strip():
        lrc_dest.write_text(plain_content.strip(), encoding="utf-8")
        return str(lrc_dest.relative_to(BASE_DIR))

    return None


def fetch_artist_payload(
    artist_key: str,
    fallback_name: str,
    fallback_avatar_url: Optional[str],
) -> Dict[str, Optional[str]]:
    url = f"https://www.nhaccuatui.com/artist/{artist_key}"
    html_bytes = http_get_bytes(url)
    detail: Dict[str, Any] = {}
    if html_bytes:
        data = extract_nuxt_data(html_bytes.decode("utf-8", errors="ignore"))
        detail = data.get("artistDataDetail") or {}

    name = detail.get("name") or fallback_name
    real_name = detail.get("fullName")
    if real_name == name:
        real_name = None
    description = clean_html_text(detail.get("description"))
    country = detail.get("national") or None
    birthday = detail.get("birthDay") or None

    avatar_url = detail.get("image") or fallback_avatar_url
    cover_url = detail.get("cover") or None

    avatar_path = download_image_smart(
        avatar_url, ARTIST_COVER_DIR / f"{artist_key}_avatar.jpg"
    )
    cover_path = download_image_smart(
        cover_url, ARTIST_COVER_DIR / f"{artist_key}_cover.jpg"
    )
    return {
        "source_key": str(artist_key),
        "name": name,
        "real_name": real_name,
        "description": description,
        "avatar_path": avatar_path,
        "cover_path": cover_path,
        "country": country,
        "birthday": birthday,
    }


def pick_best_stream_url(stream_urls: Any) -> Optional[str]:
    """Ưu tiên chọn link MP3 320kbps, nếu không có thì lấy 128kbps."""
    if not isinstance(stream_urls, list):
        return None

    mp3_320 = None
    mp3_128 = None
    for item in stream_urls:
        if not isinstance(item, dict):
            continue
        stream = item.get("stream") or item.get("download")
        if not isinstance(stream, str) or not stream.startswith("http"):
            continue
        t = str(item.get("type") or "")
        if t == "320":
            mp3_320 = stream
        elif t == "128":
            mp3_128 = stream

    return mp3_320 or mp3_128


def download_song_bundle(
    song_key: str,
    position: int,
    default_genre: str,
    known_artist_keys: set,
    artist_lock: threading.Lock,
) -> Optional[Dict[str, Any]]:
    url = f"https://www.nhaccuatui.com/song/{song_key}"
    html_bytes = http_get_bytes(url)
    if not html_bytes:
        return None

    data = extract_nuxt_data(html_bytes.decode("utf-8", errors="ignore"))
    song_detail = data.get(f"dataDetail:{song_key}")
    if not isinstance(song_detail, dict):
        return None

    title = song_detail.get("name") or "Unknown"
    artist_display = song_detail.get("artistName") or "Unknown"
    duration_sec = int(song_detail.get("duration") or 0)
    duration_ms = duration_sec * 1000
    genre = song_detail.get("genreName") or default_genre

    mp3_file = AUDIO_DIR / f"{song_key}.mp3"
    if mp3_file.exists() and mp3_file.stat().st_size > 0:
        audio_path = str(mp3_file.relative_to(BASE_DIR))
    else:
        stream_url = pick_best_stream_url(song_detail.get("streamURL"))
        if not stream_url:
            print(f"  [SKIP] Bài '{title}' ({song_key}) không có link stream MP3.")
            return None
        audio_path = download_file(stream_url, mp3_file)
        if not audio_path:
            print(f"  [SKIP] Tải MP3 thất bại cho bài '{title}'")
            return None

    cover_path = download_image_smart(
        song_detail.get("image"), SONG_COVER_DIR / f"{song_key}.jpg"
    )
    lyrics_path = fetch_and_save_Lyrics(song_key, song_detail.get("LyricDetail"))

    artist_refs: List[Tuple[str, str, Optional[str], bool]] = []
    artists = song_detail.get("artist")
    if isinstance(artists, list) and artists:
        for idx, art in enumerate(artists):
            if not isinstance(art, dict):
                continue
            art_key = str(art.get("key") or "").strip()
            art_name = str(art.get("name") or "").strip()
            if art_key and art_name:
                artist_refs.append((art_key, art_name, art.get("image"), idx == 0))
    else:
        fallback_art_key = str(song_detail.get("artistId") or "").strip()
        if fallback_art_key:
            artist_refs.append(
                (
                    fallback_art_key,
                    artist_display,
                    song_detail.get("artistImage"),
                    True,
                )
            )

    fetched_artists: List[Tuple[Dict[str, Optional[str]], bool]] = []
    for art_key, art_name, art_img, is_main in artist_refs:
        need_fetch = False
        with artist_lock:
            if art_key not in known_artist_keys:
                known_artist_keys.add(art_key)
                need_fetch = True
        if need_fetch:
            payload = fetch_artist_payload(art_key, art_name, art_img)
        else:
            payload = {
                "source_key": art_key,
                "name": art_name,
                "real_name": None,
                "description": None,
                "avatar_path": None,
                "cover_path": None,
                "country": None,
                "birthday": None,
            }
        fetched_artists.append((payload, is_main))

    print(f"  [OK #{position + 1}] {title} - {artist_display} ({song_key})", flush=True)
    return {
        "source_key": song_key,
        "position": position,
        "title": title,
        "artist_display": artist_display,
        "duration_ms": duration_ms,
        "audio_path": audio_path,
        "cover_path": cover_path,
        "lyrics_path": lyrics_path,
        "genre": genre,
        "artists": fetched_artists,
    }


def crawl_playlist(
    conn,
    playlist_key: str,
    genre: str = "VPOP",
    album_type: str = "PLAYLIST",
    limit: int = 100,
    workers: int = 8,
    visited_artists: Optional[Dict[str, str]] = None,
) -> None:
    if visited_artists is None:
        visited_artists = {}

    url = f"https://www.nhaccuatui.com/playlist/{playlist_key}"
    print(f"\n=== Đang crawl Playlist: {url} (limit={limit}, workers={workers}) ===", flush=True)
    html_bytes = http_get_bytes(url)
    if not html_bytes:
        print(f"[ERROR] Không tải được trang Playlist {playlist_key}", flush=True)
        return

    data = extract_nuxt_data(html_bytes.decode("utf-8", errors="ignore"))
    pl_detail = data.get(f"dataDetail:{playlist_key}")
    if not isinstance(pl_detail, dict):
        print(f"[ERROR] Không bóc tách được Nuxt dataDetail cho Playlist {playlist_key}", flush=True)
        return

    title = pl_detail.get("name") or f"Playlist {playlist_key}"
    description = clean_html_text(pl_detail.get("description"))
    cover_path = download_image_smart(
        pl_detail.get("image"), ALBUM_COVER_DIR / f"{playlist_key}.jpg"
    )

    album_id = upsert_album(
        conn=conn,
        source_key=playlist_key,
        title=title,
        album_type=album_type,
        genre=genre,
        description=description,
        cover_path=cover_path,
    )
    conn.commit()

    songs = (pl_detail.get("listSong") or [])[:limit]
    print(f"Playlist '{title}' có {len(songs)} bài cần xử lý...", flush=True)

    # Kiểm tra các bài đã có sẵn trong DB & ổ cứng để bỏ qua tải lại
    existing_rows = conn.execute("SELECT source_key, id FROM songs").fetchall()
    existing_songs = {r["source_key"]: r["id"] for r in existing_rows}

    existing_artist_rows = conn.execute("SELECT source_key, id FROM artists").fetchall()
    for r in existing_artist_rows:
        visited_artists[r["source_key"]] = r["id"]

    known_artist_keys = set(visited_artists.keys())
    artist_lock = threading.Lock()

    to_download: List[Tuple[int, str]] = []
    skipped_count = 0
    for idx, item in enumerate(songs):
        if not isinstance(item, dict):
            continue
        song_key = str(item.get("key") or "").strip()
        if not song_key:
            continue
        mp3_path = AUDIO_DIR / f"{song_key}.mp3"
        if song_key in existing_songs and mp3_path.exists() and mp3_path.stat().st_size > 0:
            link_album_song(conn, album_id=album_id, song_id=existing_songs[song_key], position=idx)
            skipped_count += 1
        else:
            to_download.append((idx, song_key))

    if skipped_count > 0:
        conn.commit()
        print(f"  -> Đã có sẵn {skipped_count} bài trong kho, cần tải mới {len(to_download)} bài.", flush=True)

    saved_count = skipped_count
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {
            executor.submit(
                download_song_bundle,
                song_key,
                idx,
                genre,
                known_artist_keys,
                artist_lock,
            ): (idx, song_key)
            for idx, song_key in to_download
        }
        for fut in as_completed(futures):
            bundle = fut.result()
            if not bundle:
                continue
            song_id = upsert_song(
                conn=conn,
                source_key=bundle["source_key"],
                title=bundle["title"],
                artist_display=bundle["artist_display"],
                audio_path=bundle["audio_path"],
                duration_ms=bundle["duration_ms"],
                album_id=album_id,
                cover_path=bundle["cover_path"],
                lyrics_path=bundle["lyrics_path"],
                genre=bundle["genre"],
            )
            link_album_song(conn, album_id=album_id, song_id=song_id, position=bundle["position"])
            for art_payload, is_main in bundle["artists"]:
                art_id = upsert_artist(conn=conn, **art_payload)
                visited_artists[art_payload["source_key"]] = art_id
                link_song_artist(conn, song_id=song_id, artist_id=art_id, is_main=is_main)
            conn.commit()
            saved_count += 1

    print(f"=== Hoàn tất Playlist '{title}': Tổng cộng {saved_count}/{len(songs)} bài! ===", flush=True)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="NhacCuaTui Crawler (MP3 320kbps + LRC Lyrics + Artist Bio + Covers)"
    )
    parser.add_argument(
        "--playlist",
        type=str,
        default=None,
        help="Key hoặc URL của Playlist trên NCT (VD: m3liaiy6vVsF). Để trống sẽ chạy danh sách mặc định.",
    )
    parser.add_argument(
        "--genre",
        type=str,
        default="VPOP",
        help="Nhãn thể loại gán cho Playlist (VD: RAP, EDM, LOFI, VPOP)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=3,
        help="Số bài hát tối đa cần tải mỗi Playlist (mặc định: 3 để test nhanh)",
    )
    args = parser.parse_args()

    ensure_dirs()
    init_db()

    visited_artists: Dict[str, str] = {}
    with get_connection() as conn:
        if args.playlist:
            m = re.search(r"([A-Za-z0-9]{10,14})(?:\.html)?$", args.playlist)
            pl_key = m.group(1) if m else args.playlist
            crawl_playlist(
                conn=conn,
                playlist_key=pl_key,
                genre=args.genre,
                album_type="PLAYLIST",
                limit=args.limit,
                visited_artists=visited_artists,
            )
        else:
            for preset in DEFAULT_PLAYLISTS:
                crawl_playlist(
                    conn=conn,
                    playlist_key=preset["key"],
                    genre=preset["genre"],
                    album_type=preset["type"],
                    limit=args.limit,
                    visited_artists=visited_artists,
                )


if __name__ == "__main__":
    main()
