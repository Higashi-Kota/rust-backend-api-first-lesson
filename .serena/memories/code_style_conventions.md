# Code Style and Conventions

## Rust Conventions
- **Rust Version**: 1.86.0 (pinned via rust-toolchain.toml)
- **Edition**: 2021
- **Naming**: 
  - snake_case for functions, variables, modules
  - PascalCase for types, traits
  - SCREAMING_SNAKE_CASE for constants

## Project-Specific Guidelines (from CLAUDE.md)

### Core Principles
- **No API duplication**: Avoid similar endpoints with different names
- **Path parameters**: Use `{param}` format (Axum 0.8 style)
- **No `/api/` prefix**: Use functional prefixes like `/admin/*`, `/auth/*`, `/tasks/*`

### Error Handling
- **Always use error_helper functions** in service layer
- **Context format**: `"module_name::function_name[:detail]"`
- Example: `internal_server_error(e, "task_service::get_stats", "Failed to count tasks")`

### Database Conventions
- **Table names**: Always plural (users, tasks, teams)
- **Column names**: snake_case
- **Foreign keys**: `{table_singular}_id` format (user_id, team_id)
- **Standard columns**: All tables have `id` (UUID), `created_at`, `updated_at`
- **Timestamps**: Always use TIMESTAMPTZ type
- **Migration naming**: `m{YYYYMMDD}_{6-digit-sequence}_{description}.rs`

### API Response Format
- **Unified response**: Use `ApiResponse<T>` wrapper
- **Timestamps**: Unix timestamp (seconds) for all datetime fields
- **Success/error structure**: Consistent JSON format with `success`, `data`, `error`, `meta` fields

### Dead Code Policy
- **No new `#[allow(dead_code)]`** annotations
- **Remove unused code** unless it's test helpers
- **Exception**: Test utilities like `AppConfig::for_testing`

### Security & Permissions
- **Admin APIs**: Must use `/admin` prefix
- **Use unified permission middleware**: `require_permission!` macro
- **No direct permission checks**: Use middleware instead
- **Three test patterns required**: Success, Forbidden (403), Unauthorized (401)

### Logging
- **Use `log_with_context!` macro**: Never use tracing directly
- **Three-stage pattern**: DEBUG (start), INFO (success), ERROR (failure)
- **Structured logs**: Include request_id, user_id, operation details

### Testing Requirements
- **AAA Pattern**: Arrange-Act-Assert for all tests
- **Real data**: No hardcoded test values, create actual test data
- **Verify values**: Not just structure, check actual values
- **Integration tests**: E2E flow with DB verification
- **Error coverage**: Test all error paths

### Pagination & Query Patterns
- **Use unified types**: `PaginationQuery`, `SortQuery`, `SearchQuery`
- **Default page size**: 20, max 100
- **Date filters**: Use `created_after`, `created_before` naming

### UUID Validation
- **Use `ValidatedUuid` extractor** for path parameters
- **Custom deserializers** for multiple UUID paths
- **Error format**: `"Invalid UUID format: 'xxx'"`

## Clippy Lints
- Enforced warnings: `cloned_instead_of_copied`, `inefficient_to_string`, `map_unwrap_or`
- All targets and features checked with `-D warnings`

## Testing Style
- Unit tests in `src/*/mod.rs`
- Integration tests in `tests/integration/`
- Test helpers in `tests/common/`
- Independent test schemas for parallel execution