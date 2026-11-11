PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

-- ============================================================
-- 1. BẢNG ARTISTS (Nghệ sĩ)
-- ============================================================
CREATE TABLE IF NOT EXISTS artists (
    id          TEXT PRIMARY KEY,               -- UUID
    source_key  TEXT NOT NULL UNIQUE,           -- Slug/ID trên NCT (VD: 'son-tung-m-tp')
    name        TEXT NOT NULL,                  -- Nghệ danh (VD: 'Sơn Tùng M-TP')
    real_name   TEXT,                           -- Tên thật (VD: 'Nguyễn Thanh Tùng')
    description TEXT,                           -- Tiểu sử / giới thiệu nghệ sĩ
    avatar_path TEXT,                           -- Đường dẫn ảnh đại diện vuông/tròn
    cover_path  TEXT,                           -- Đường dẫn ảnh bìa lớn (banner)
    country     TEXT,                           -- Quốc gia (VD: 'Việt Nam', 'US-UK')
    birthday    TEXT,                           -- Ngày sinh
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- 2. BẢNG ALBUMS (Album / Playlist / Bảng xếp hạng Top 100)
-- ============================================================
CREATE TABLE IF NOT EXISTS albums (
    id          TEXT PRIMARY KEY,               -- UUID
    source_key  TEXT NOT NULL UNIQUE,           -- Key playlist/album trên NCT
    title       TEXT NOT NULL,                  -- Tên Album / Playlist
    type        TEXT NOT NULL DEFAULT 'PLAYLIST' CHECK (type IN ('ALBUM', 'PLAYLIST', 'CHART')),
    genre       TEXT,                           -- Thể loại chính (VD: 'RAP', 'EDM', 'LOFI', 'VPOP')
    description TEXT,                           -- Mô tả Album / Playlist
    cover_path  TEXT,                           -- Đường dẫn ảnh bìa 500x500
    release_date TEXT,                          -- Ngày phát hành / cập nhật
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- 3. BẢNG SONGS (Bài hát & Metadata phục vụ AI)
-- ============================================================
CREATE TABLE IF NOT EXISTS songs (
    id              TEXT PRIMARY KEY,           -- UUID
    source_key      TEXT NOT NULL UNIQUE,       -- Key bài hát trên NCT (chống tải trùng MP3)
    title           TEXT NOT NULL,              -- Tên bài hát
    artist_display  TEXT NOT NULL,              -- Chuỗi tên ca sĩ hiển thị nhanh (VD: 'HIEUTHUHAI, Hurrykng')
    duration_ms     INTEGER NOT NULL DEFAULT 0, -- Thời lượng (mili-giây)
    audio_path      TEXT NOT NULL,              -- Đường dẫn file .mp3 trên ổ cứng
    cover_path      TEXT,                       -- Đường dẫn ảnh bìa 500x500
    lyrics_path     TEXT,                       -- Đường dẫn file lời bài hát .lrc (có timestamp)
    genre           TEXT,                       -- Nhãn thể loại phục vụ lọc & fine-tune AI
    bpm             REAL,                       -- Nhịp độ bài hát (do AI phân tích)
    musical_key     TEXT,                       -- Tông nhạc (VD: 'C minor')
    play_count      INTEGER NOT NULL DEFAULT 0, -- Lượt nghe
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- 4. BẢNG NỐI SONG_ARTISTS (1 bài hát có thể có nhiều ca sĩ)
-- ============================================================
CREATE TABLE IF NOT EXISTS song_artists (
    song_id     TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    artist_id   TEXT NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    is_main     INTEGER NOT NULL DEFAULT 1,     -- 1: Ca sĩ chính, 0: Featuring
    PRIMARY KEY (song_id, artist_id)
);

-- ============================================================
-- 5. BẢNG NỐI ALBUM_SONGS (1 bài hát nằm trong nhiều Playlist)
-- ============================================================
CREATE TABLE IF NOT EXISTS album_songs (
    album_id    TEXT NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    song_id     TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL DEFAULT 0,     -- Thứ tự bài hát trong Playlist/Album
    PRIMARY KEY (album_id, song_id)
);

-- ============================================================
-- 6. BẢNG DRUM_KITS (Quản lý trạng thái Lazy AI tách Drum Pad)
-- ============================================================
CREATE TABLE IF NOT EXISTS drum_kits (
    song_id         TEXT PRIMARY KEY REFERENCES songs(id) ON DELETE CASCADE,
    status          TEXT NOT NULL DEFAULT 'NONE' CHECK (status IN ('NONE', 'PROCESSING', 'READY', 'FAILED')),
    stems_dir       TEXT,                       -- Thư mục chứa 4 file stems (drums, bass, vocals, other)
    error_message   TEXT,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- 7. BẢNG DRUM_PADS (16 ô mẫu âm thanh của 1 bộ Drum Kit)
-- ============================================================
CREATE TABLE IF NOT EXISTS drum_pads (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    song_id         TEXT NOT NULL REFERENCES drum_kits(song_id) ON DELETE CASCADE,
    pad_index       INTEGER NOT NULL CHECK (pad_index BETWEEN 0 AND 15),
    label           TEXT NOT NULL,              -- VD: 'Kick', 'Snare', 'Hi-Hat', 'Bass 1', 'Vocal Chop'
    category        TEXT NOT NULL CHECK (category IN ('DRUM', 'BASS', 'MELODY', 'VOCAL')),
    color_hex       TEXT,                       -- Màu hiển thị ô Pad trên UI
    sample_path     TEXT NOT NULL,              -- Đường dẫn file .wav ngắn nạp vào SoundPool
    source_start_ms INTEGER,                    -- Vị trí cắt bắt đầu trong bài gốc (ms) - dùng làm nhãn AI
    source_end_ms   INTEGER,                    -- Vị trí cắt kết thúc trong bài gốc (ms)
    UNIQUE (song_id, pad_index)
);

-- ============================================================
-- INDEXES TỐI ƯU TRUY VẤN
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_songs_genre ON songs(genre);
CREATE INDEX IF NOT EXISTS idx_albums_genre ON albums(genre);
CREATE INDEX IF NOT EXISTS idx_albums_type ON albums(type);
CREATE INDEX IF NOT EXISTS idx_song_artists_artist_id ON song_artists(artist_id);
CREATE INDEX IF NOT EXISTS idx_album_songs_song_id ON album_songs(song_id);
CREATE INDEX IF NOT EXISTS idx_drum_pads_song_id ON drum_pads(song_id);
