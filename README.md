# feedhook

YouTube RSS 피드를 폴링하여 Discord 웹훅으로 새 영상 알림을 보내는 셀프호스팅 서비스.

## Features

- YouTube 채널 RSS 자동 폴링
- Discord webhook embed 알림
- 채널별 개별 webhook URL 설정
- 웹 UI (채널 관리, 알림 이력, 설정)
- 단일 바이너리 (정적 파일 임베드)
- SQLite (무설정 스토리지)

## Quick Start

### Docker (권장)

```bash
docker compose up -d
```

`http://localhost:8080`으로 접속.

### 직접 빌드

```bash
cargo build --release
DATABASE_URL=sqlite://feedhook.db ./target/release/feedhook
```

## Configuration

| 환경변수 | 기본값 | 설명 |
|---|---|---|
| `PORT` | `8080` | 서버 포트 |
| `DATABASE_URL` | `sqlite://feedhook.db` | SQLite DB 경로 |

폴링 간격은 웹 UI 설정 탭에서 변경 (기본 300초).

## API

```
GET    /api/channels     채널 목록
POST   /api/channels     채널 추가
DELETE /api/channels/:id  채널 삭제
GET    /api/settings     설정 조회
PUT    /api/settings     설정 변경
GET    /api/logs         알림 이력 (최근 50건)
POST   /api/poll         수동 폴링
```

## Tech Stack

Rust, axum, sqlx + SQLite, feed-rs, reqwest, Alpine.js + Pico CSS, rust-embed
