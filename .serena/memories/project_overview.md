# Project Overview

## Project Purpose
This is a **Rust-based Task Management API** built with modern backend technologies. It features a sophisticated **dynamic permission system** that adapts API responses based on user roles and subscription tiers.

## Core Features
- Task management with CRUD operations and batch processing
- JWT-based authentication with access/refresh tokens
- Role-based access control (RBAC) with dynamic permissions
- Multi-tenant support via database schemas
- Subscription tiers (Free/Pro/Enterprise) with different feature sets
- Team and organization hierarchy management
- File attachments with MinIO/S3 storage
- Payment processing with Stripe integration
- GDPR compliance features (data export/deletion)
- Comprehensive audit logging

## Architecture Pattern
**Layered Architecture**:
- **API Layer**: Axum handlers (`task-backend/src/api/handlers/`)
- **Service Layer**: Business logic (`task-backend/src/service/`)
- **Repository Layer**: Data access (`task-backend/src/repository/`)
- **Domain Layer**: Core models (`task-backend/src/domain/`)

## Dynamic Permission System
The system's core innovation is that **the same endpoint returns different responses** based on:
- User role (Admin, Member, Viewer)
- Subscription tier (Free, Pro, Enterprise)
- Permission scope (Own, Team, Organization, Global)

Example:
```
GET /tasks/dynamic
- Free user: Max 100 items, basic features
- Pro user: Max 10,000 items, advanced filters, export
- Enterprise: Unlimited, all features
```

## Database
- PostgreSQL with SeaORM
- Multi-tenant support via schemas
- Comprehensive migrations system
- All timestamps use TIMESTAMPTZ
- Table names are plural (users, tasks, teams)