# Task Backend API

A RESTful API for task management built with Rust, Axum, and PostgreSQL.

[![CI/CD Pipeline](https://github.com/Higashi-Kota/rust-backend-api-first-lesson/actions/workflows/ci.yml/badge.svg)](https://github.com/Higashi-Kota/rust-backend-api-first-lesson/actions/workflows/ci.yml)

## 🚀 Features

- **RESTful API** for task management (CRUD operations)
- **Batch operations** for creating, updating, and deleting multiple tasks
- **Advanced filtering** and pagination
- **Database migrations** with Sea-ORM
- **Comprehensive testing** with integration and unit tests
- **Docker support** for easy deployment
- **Multi-schema support** for tenant isolation

## 🛠 Technology Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: PostgreSQL
- **ORM**: Sea-ORM
- **Testing**: Testcontainers
- **Containerization**: Docker

## 📋 Prerequisites

- Rust 1.75+
- PostgreSQL 15+
- Docker (optional)

## 🚀 Quick Start

### Using Docker (Recommended)

1. **Clone the repository**

   ```bash
   git clone https://github.com/Higashi-Kota/rust-backend-api-first-lesson.git
   cd rust-backend-api-first-lesson/task-backend
   ```

2. **Start the application**

   ```bash
   docker-compose up
   ```

3. **Access the API**
   ```bash
   curl http://localhost:3000/health
   ```

### Local Development

1. **Set up environment**

   ```bash
   make dev-setup
   # Edit .env file with your database credentials
   ```

2. **Run database migrations**

   ```bash
   make migrate
   ```

3. **Start the application**
   ```bash
   make run
   ```

## 🧪 Testing

```bash
# Run all tests
make test

# Run CI checks locally
make ci-check
```

## 🐳 Docker イメージの使用

### GitHub Container Registry からイメージを取得

```bash
# 公開リポジトリの場合（認証不要）
docker pull ghcr.io/Higashi-Kota/rust-backend-api-first-lesson:latest

# プライベートリポジトリの場合（GitHub Personal Access Token が必要）
echo $GITHUB_TOKEN | docker login ghcr.io -u Higashi-Kota --password-stdin
docker pull ghcr.io/Higashi-Kota/rust-backend-api-first-lesson:latest

# 実行
docker run -p 3000:3000 \
  -e DATABASE_URL=postgres://user:pass@localhost:5432/db \
  ghcr.io/Higashi-Kota/rust-backend-api-first-lesson:latest
```

### ローカルでのビルドと実行

```bash
# Docker Compose でローカルビルド（推奨）
docker-compose up --build

# 手動ビルド
docker build -t task-backend .
docker run -p 3000:3000 task-backend
```

## 📡 API Endpoints

### Health Check

- `GET /health` - Health check endpoint

### Task Management

- `GET /tasks` - List all tasks
- `GET /tasks/paginated?page=1&page_size=10` - Get paginated tasks
- `GET /tasks/filter?status=todo&title_contains=urgent` - Filter tasks
- `GET /tasks/{id}` - Get task by ID
- `POST /tasks` - Create new task
- `PATCH /tasks/{id}` - Update task
- `DELETE /tasks/{id}` - Delete task

### Batch Operations

- `POST /tasks/batch/create` - Create multiple tasks
- `PATCH /tasks/batch/update` - Update multiple tasks
- `POST /tasks/batch/delete` - Delete multiple tasks

## 📊 Request/Response Examples

### Create Task

```bash
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Complete project",
    "description": "Finish the task management API",
    "status": "todo",
    "due_date": "2024-12-31T23:59:59Z"
  }'
```

### Filter Tasks

```bash
curl "http://localhost:3000/tasks/filter?status=todo&limit=5&sort_by=created_at&sort_order=desc"
```

## 🔧 Configuration

Environment variables can be set in `.env` file:

```bash
# Database
DATABASE_URL=postgres://username:password@localhost:5432/database_name

# Server
SERVER_ADDR=0.0.0.0:3000

# Optional: Multi-schema support
DB_SCHEMA=your_schema_name

# Logging
RUST_LOG=info
```

## 🚢 Deployment

### Using Docker

1. **Build image**

   ```bash
   make docker-build
   ```

2. **Deploy with Docker Compose**
   ```bash
   docker-compose -f docker-compose.prod.yml up
   ```

### Manual Deployment

1. **Build release**

   ```bash
   make build
   ```

2. **Run migrations**

   ```bash
   ./target/release/migration up
   ```

3. **Start application**
   ```bash
   ./target/release/task-backend
   ```

## 🔄 CI/CD

The project uses GitHub Actions for continuous integration and deployment:

- **Testing**: Runs on every push and pull request
- **Security**: Automated security audits with cargo-audit
- **Docker**: Builds and pushes images to Docker Hub
- **Deployment**: Automated deployment to staging and production

## 📈 Monitoring

### Health Check

```bash
curl http://localhost:3000/health
```

### Logs

```bash
# View application logs
docker-compose logs -f app

# Set log level
export RUST_LOG=debug
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Troubleshooting

### Common Issues

1. **Database connection errors**

   - Ensure PostgreSQL is running
   - Check DATABASE_URL in .env file
   - Verify database credentials

2. **Migration errors**

   - Run `make migrate` to apply latest migrations
   - Check database permissions

3. **Test failures**
   - Ensure Docker is running for integration tests
   - Check that ports 5432 is available

### Getting Help

- Open an issue on GitHub
- Check the [API documentation](docs/api.md)
- Review the [troubleshooting guide](docs/troubleshooting.md)
