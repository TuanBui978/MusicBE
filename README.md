# MusicBE - High Performance Music Streaming Server & Crawler

Backend server hiệu năng cao cho ứng dụng nghe nhạc trực tuyến, được xây dựng bằng **Rust (Axum + SQLx + Tokio)** và crawler dữ liệu đa nguồn bằng **Python**.

---

## 🚀 Tính năng nổi bật

- **Kiến trúc Layered Clean Architecture**: Tách bạch hoàn toàn giữa `entities` (Database Model) và `dtos` (Mạng/Client).
- **Parallel Query Execution**: Dùng `tokio::try_join!` bắn đồng thời nhiều query song song không chặn luồng cho các API Detail và Unified Search.
- **HTTP Range Streaming (RFC 7233)**: Hỗ trợ mã `206 Partial Content` phục vụ ExoPlayer / Media3 tua nhạc siêu mượt, không tốn RAM.
- **Anti-Spam Play Count**: Tự động tăng lượt nghe (`play_count`) khi bắt đầu nghe, không bị đếm ảo khi người dùng tua nhạc.
- **Unified Search 3-trong-1**: Tìm kiếm đồng thời Bài hát (Songs), Nghệ sĩ (Artists), và Album/Playlist (Albums).

---

## 📁 Cấu trúc thư mục

```
MusicBE/
├── server/                 # Mã nguồn Rust Backend (Axum 0.8 + SQLx 0.8)
│   ├── src/
│   │   ├── api/            # Axum Handlers & Sub-routers
│   │   ├── services/       # Nghiệp vụ & Xử lý song song
│   │   ├── repositories/   # SQL queries thuần túy (Free functions)
│   │   ├── models/
│   │   │   ├── entities/   # Ánh xạ bảng SQLite 1-1
│   │   │   └── dtos/       # DTOs trả về cho Client
│   │   ├── error.rs        # Centralized AppError
│   │   ├── state.rs        # AppState (Arc<Config> + SqlitePool)
│   │   └── main.rs         # Server entry point & Graceful shutdown
│   └── Cargo.toml
├── crawler/                # Python crawler & Stems separator
├── database/               # SQLite Schema (schema.sql)
└── storage/                # Media Storage (MP3, Covers, Lyrics)
```

---

## 📡 API Endpoints

| Method | Endpoint | Mô tả |
| :--- | :--- | :--- |
| `GET` | `/health` | Kiểm tra sức khỏe hệ thống & kết nối DB |
| `GET` | `/api/v1/search?q={query}&limit=10` | Tìm kiếm tổng hợp 3-trong-1 (Song, Artist, Album) |
| `GET` | `/api/v1/songs?page=1&page_size=20` | Danh sách bài hát phân trang |
| `GET` | `/api/v1/songs/{id}` | Chi tiết bài hát (`SongDetail`: Song + Artists + Albums) |
| `GET` | `/api/v1/songs/{id}/stream` | **Stream nhạc trực tiếp (Range 206 + Auto increment play count)** |
| `POST`| `/api/v1/songs/{id}/play` | Tăng lượt nghe thủ công |
| `GET` | `/api/v1/artists?page=1&page_size=20` | Danh sách nghệ sĩ phân trang |
| `GET` | `/api/v1/artists/{id}` | Chi tiết nghệ sĩ (`ArtistDetail`: Bio + Top 10 + Albums) |
| `GET` | `/api/v1/artists/{id}/songs?page=1` | Danh sách toàn bộ bài hát của nghệ sĩ |
| `GET` | `/api/v1/albums?type=CHART&page=1` | Danh sách Album/Playlist/BXH (có lọc `type`) |
| `GET` | `/api/v1/albums/{id}` | Chi tiết Album (`AlbumDetail`: Tracks theo vị trí + Artists) |
| `GET` | `/api/v1/albums/{id}/songs` | Danh sách bài hát trong Album |

---

## ⚙️ Cài đặt & Chạy Local

### 1. Khởi tạo Database
```bash
sqlite3 database/music.db < database/schema.sql
```

### 2. Cấu hình biến môi trường
```bash
cp server/.env.example server/.env
```

### 3. Khởi động Server
```bash
cd server
cargo run
```
Server sẽ lắng nghe tại `http://0.0.0.0:8080`.
