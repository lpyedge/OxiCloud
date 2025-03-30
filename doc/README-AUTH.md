# OxiCloud Authentication System

This document describes the authentication system for OxiCloud, a file storage system built with Rust and PostgreSQL.

## Overview

OxiCloud uses a standard JWT (JSON Web Token) authentication system with the following features:

- User registration and login
- Role-based access control (Admin/User)
- JWT token with refresh capabilities
- Secure password hashing with Argon2id
- User storage quotas
- File and folder ownership

## API Endpoints

The authentication API is available at the `/api/auth` endpoint:

- **POST /api/auth/register** - Register a new user
- **POST /api/auth/login** - Login and get tokens
- **POST /api/auth/refresh** - Refresh access token
- **GET /api/auth/me** - Get current user information
- **PUT /api/auth/change-password** - Change user password
- **POST /api/auth/logout** - Logout and invalidate refresh token

## Request/Response Examples

### Register

**Request:**
```json
POST /api/auth/register
{
  "username": "testuser",
  "email": "test@example.com",
  "password": "SecurePassword123"
}
```

**Response:**
```json
201 Created
{
  "userId": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "username": "testuser",
  "email": "test@example.com"
}
```

### Login

**Request:**
```json
POST /api/auth/login
{
  "username": "testuser",
  "password": "SecurePassword123"
}
```

**Response:**
```json
200 OK
{
  "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

### Refresh Token

**Request:**
```json
POST /api/auth/refresh
{
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response:**
```json
200 OK
{
  "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

### Get Current User

**Request:**
```
GET /api/auth/me
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response:**
```json
200 OK
{
  "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "username": "testuser",
  "email": "test@example.com",
  "role": "user",
  "storageQuota": 10737418240,
  "storageUsed": 1048576,
  "createdAt": "2023-01-01T12:00:00Z"
}
```

### Change Password

**Request:**
```json
PUT /api/auth/change-password
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
{
  "oldPassword": "SecurePassword123",
  "newPassword": "NewSecurePassword456"
}
```

**Response:**
```
200 OK
```

### Logout

**Request:**
```
POST /api/auth/logout
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response:**
```
200 OK
```

## Testing the Authentication System

1. Start PostgreSQL and create the database:
   ```bash
   createdb oxicloud
   psql -d oxicloud -f db/schema.sql
   ```

2. Set environment variables for authentication:
   ```bash
   source test-auth-env.sh
   ```

3. Start the OxiCloud server:
   ```bash
   cargo run
   ```

4. Run the authentication test script:
   ```bash
   ./test-auth-api.sh
   ```

## Database Schema

The authentication system uses the following tables:

- `users` - Store user information
- `sessions` - Store refresh token sessions
- `file_ownership` - Track file ownership
- `folder_ownership` - Track folder ownership

## Implementation Details

- **Password Hashing**: Argon2id with memory cost of 65536 (64MB), time cost of 3, and 4 parallelism
- **JWT Secret**: Configured via environment variable `OXICLOUD_JWT_SECRET`
- **Token Expiry**: Access token expires in 1 hour, refresh token in 30 days (configurable)
- **Database Connection**: PostgreSQL with connection pooling
- **Middleware**: Auth middleware for protected routes

## Security Considerations

- Passwords are never stored in plain text, only as Argon2id hashes
- JWT tokens are signed with a secret key
- Refresh tokens can be revoked to force logout
- Rate limiting should be implemented for login attempts
- Password policy requires at least 8 characters
- Regular security audits recommended

## Future Improvements

- Email verification for new registrations
- Password reset functionality
- Enhanced password policy
- Two-factor authentication
- OAuth integration for social logins
- Session management UI