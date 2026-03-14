# feedhook

YouTube RSS → Discord webhook 알림 서비스.

## Build & Run

```bash
cargo build
cargo run
# or
DATABASE_URL=sqlite://feedhook.db cargo run
```

## Architecture

- `main.rs`: 엔트리포인트, axum 라우터, rust-embed 정적 파일 서빙
- `config.rs`: 환경변수 (PORT, DATABASE_URL)
- `db.rs`: SQLite pool + 마이그레이션
- `models.rs`: DB 모델 (Channel, NotificationLog, Settings)
- `error.rs`: AppError → axum IntoResponse
- `handlers.rs`: REST API 핸들러
- `poller.rs`: 폴링 루프 (tokio::time::sleep + watch 채널로 간격 동적 변경)
- `notifier.rs`: Discord webhook 전송 (429 rate limit 재시도)

## Key Patterns

- `AppState`를 `Arc`로 감싸서 axum State로 공유
- `watch::channel`로 폴링 간격 실시간 변경
- 채널 최초 등록 시 기존 영상 seed (알림 폭탄 방지)
- `web/` 디렉토리가 rust-embed로 바이너리에 임베드됨

## DB

SQLite WAL 모드. 마이그레이션은 `migrations/` 디렉토리에 SQL 파일로 관리 (sqlx::migrate! 매크로).

## Frontend

Alpine.js + Pico CSS. 빌드 스텝 없음. `web/index.html`, `web/app.js`.
