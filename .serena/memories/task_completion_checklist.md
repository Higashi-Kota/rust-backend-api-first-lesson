# Task Completion Checklist

## Before Marking Any Task as Complete

### 1. Code Quality Checks ✅
```bash
# MUST pass all these commands with ZERO warnings/errors:
make fmt                    # Format all code
make clippy                # Linter check - MUST have zero warnings
make ci-check-fast         # Full CI check
```

### 2. Testing Requirements ✅
- [ ] All existing tests still pass: `make test`
- [ ] New features have integration tests with 3 cases:
  - Success case (200/201)
  - Forbidden case (403)
  - Unauthorized case (401)
- [ ] Tests use real data, not hardcoded values
- [ ] Tests verify actual values, not just structure

### 3. API Implementation Standards ✅
- [ ] Uses `ApiResponse<T>` wrapper for all responses
- [ ] Datetime fields return Unix timestamps (seconds)
- [ ] Error handling uses `error_helper` functions
- [ ] Logging uses `log_with_context!` macro
- [ ] UUID validation uses `ValidatedUuid` extractor
- [ ] Pagination uses unified `PaginationQuery` type

### 4. Security & Permissions ✅
- [ ] Admin endpoints use `/admin` prefix
- [ ] Permissions use middleware, not direct checks
- [ ] Sensitive endpoints have proper authentication
- [ ] No secrets/keys in code or logs

### 5. Database Changes ✅
- [ ] Migration files follow naming: `m{YYYYMMDD}_{6-digits}_{description}.rs`
- [ ] Table names are plural
- [ ] Foreign keys use `{table}_id` format
- [ ] All tables have id, created_at, updated_at
- [ ] Timestamps use TIMESTAMPTZ type

### 6. Documentation & Clean Code ✅
- [ ] No new `#[allow(dead_code)]` annotations
- [ ] Unused code removed (YAGNI principle)
- [ ] No commented-out code
- [ ] API changes reflected in handlers

### 7. Final Verification ✅
```bash
# One final check before committing:
cargo clippy --all-targets --all-features -- -D warnings
# MUST show: "0 warnings"

# Verify no dead_code annotations added:
rg "#\[allow\(dead_code\)\]" --type rust | grep -v "tests/"
# Should only show test helpers
```

## Quick Validation Command
Run this single command to verify everything:
```bash
make ci-check-fast && echo "✅ Ready to commit!" || echo "❌ Fix issues first!"
```

## If Task Involves New API Endpoints
Additionally ensure:
- [ ] Endpoint follows routing conventions (no `/api/` prefix)
- [ ] Path parameters use `{param}` format
- [ ] Integration tests cover the endpoint
- [ ] Proper error responses for invalid inputs