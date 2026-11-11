import sqlite3
import uuid
from pathlib import Path
from typing import Optional

BASE_DIR = Path(__file__).resolve().parent.parent
DB_DIR = BASE_DIR / "database"
DB_PATH = DB_DIR / "music.db"
SCHEMA_PATH = DB_DIR / "schema.sql"


def get_connection(db_path: Path = DB_PATH) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON;")
    conn.execute("PRAGMA journal_mode = WAL;")
    return conn


def init_db(db_path: Path = DB_PATH) -> None:
    schema_sql = SCHEMA_PATH.read_text(encoding="utf-8")
    with get_connection(db_path) as conn:
        conn.executescript(schema_sql)
        conn.commit()


def upsert_artist(
    conn: sqlite3.Connection,
    source_key: str,
    name: str,
    real_name: Optional[str] = None,
    description: Optional[str] = None,
    avatar_path: Optional[str] = None,
    cover_path: Optional[str] = None,
    country: Optional[str] = None,
    birthday: Optional[str] = None,
) -> str:
    row = conn.execute(
        "SELECT id FROM artists WHERE source_key = ?", (source_key,)
    ).fetchone()
    if row:
        artist_id = row["id"]
        conn.execute(
            """
            UPDATE artists
            SET name = COALESCE(?, name),
                real_name = COALESCE(?, real_name),
                description = COALESCE(?, description),
                avatar_path = COALESCE(?, avatar_path),
                cover_path = COALESCE(?, cover_path),
                country = COALESCE(?, country),
                birthday = COALESCE(?, birthday)
            WHERE id = ?
            """,
            (
                name,
                real_name,
                description,
                avatar_path,
                cover_path,
                country,
                birthday,
                artist_id,
            ),
        )
        return artist_id

    artist_id = str(uuid.uuid4())
    conn.execute(
        """
        INSERT INTO artists (
            id, source_key, name, real_name, description,
            avatar_path, cover_path, country, birthday
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            artist_id,
            source_key,
            name,
            real_name,
            description,
            avatar_path,
            cover_path,
            country,
            birthday,
        ),
    )
    return artist_id


def upsert_album(
    conn: sqlite3.Connection,
    source_key: str,
    title: str,
    album_type: str = "PLAYLIST",
    artist_id: Optional[str] = None,
    genre: Optional[str] = None,
    description: Optional[str] = None,
    cover_path: Optional[str] = None,
    release_date: Optional[str] = None,
) -> str:
    row = conn.execute(
        "SELECT id FROM albums WHERE source_key = ?", (source_key,)
    ).fetchone()
    if row:
        album_id = row["id"]
        conn.execute(
            """
            UPDATE albums
            SET title = COALESCE(?, title),
                type = COALESCE(?, type),
                genre = COALESCE(?, genre),
                description = COALESCE(?, description),
                cover_path = COALESCE(?, cover_path),
                release_date = COALESCE(?, release_date)
            WHERE id = ?
            """,
            (
                title,
                album_type,
                genre,
                description,
                cover_path,
                release_date,
                album_id,
            ),
        )
        return album_id

    album_id = str(uuid.uuid4())
    conn.execute(
        """
        INSERT INTO albums (
            id, source_key, title, type,
            genre, description, cover_path, release_date
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            album_id,
            source_key,
            title,
            album_type,
            genre,
            description,
            cover_path,
            release_date,
        ),
    )
    return album_id


def upsert_song(
    conn: sqlite3.Connection,
    source_key: str,
    title: str,
    artist_display: str,
    audio_path: str,
    duration_ms: int = 0,
    album_id: Optional[str] = None,
    cover_path: Optional[str] = None,
    lyrics_path: Optional[str] = None,
    genre: Optional[str] = None,
    bpm: Optional[float] = None,
    musical_key: Optional[str] = None,
) -> str:
    row = conn.execute(
        "SELECT id FROM songs WHERE source_key = ?", (source_key,)
    ).fetchone()
    if row:
        song_id = row["id"]
        conn.execute(
            """
            UPDATE songs
            SET title = COALESCE(?, title),
                artist_display = COALESCE(?, artist_display),
                duration_ms = CASE WHEN ? > 0 THEN ? ELSE duration_ms END,
                audio_path = COALESCE(?, audio_path),
                cover_path = COALESCE(?, cover_path),
                lyrics_path = COALESCE(?, lyrics_path),
                genre = COALESCE(?, genre),
                bpm = COALESCE(?, bpm),
                musical_key = COALESCE(?, musical_key)
            WHERE id = ?
            """,
            (
                title,
                artist_display,
                duration_ms,
                duration_ms,
                audio_path,
                cover_path,
                lyrics_path,
                genre,
                bpm,
                musical_key,
                song_id,
            ),
        )
        return song_id

    song_id = str(uuid.uuid4())
    conn.execute(
        """
        INSERT INTO songs (
            id, source_key, title, artist_display,
            duration_ms, audio_path, cover_path, lyrics_path,
            genre, bpm, musical_key
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            song_id,
            source_key,
            title,
            artist_display,
            duration_ms,
            audio_path,
            cover_path,
            lyrics_path,
            genre,
            bpm,
            musical_key,
        ),
    )
    # Khởi tạo trạng thái mặc định cho Drum Kit là NONE
    conn.execute(
        "INSERT OR IGNORE INTO drum_kits (song_id, status) VALUES (?, 'NONE')",
        (song_id,),
    )
    return song_id


def link_song_artist(
    conn: sqlite3.Connection, song_id: str, artist_id: str, is_main: bool = True
) -> None:
    conn.execute(
        """
        INSERT OR REPLACE INTO song_artists (song_id, artist_id, is_main)
        VALUES (?, ?, ?)
        """,
        (song_id, artist_id, 1 if is_main else 0),
    )


def link_album_song(
    conn: sqlite3.Connection, album_id: str, song_id: str, position: int = 0
) -> None:
    conn.execute(
        """
        INSERT OR REPLACE INTO album_songs (album_id, song_id, position)
        VALUES (?, ?, ?)
        """,
        (album_id, song_id, position),
    )


if __name__ == "__main__":
    init_db()
    print(f"Database initialized successfully at: {DB_PATH}")
