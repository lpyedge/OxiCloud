-- OxiCloud Authentication Database Schema

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(36) PRIMARY KEY,
    username VARCHAR(32) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(10) NOT NULL CHECK (role IN ('admin', 'user')),
    storage_quota_bytes BIGINT NOT NULL DEFAULT 10737418240, -- 10GB default
    storage_used_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at TIMESTAMP WITH TIME ZONE,
    active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Sessions table for refresh tokens
CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (user_id, refresh_token)
);

-- File ownership tracking
CREATE TABLE IF NOT EXISTS file_ownership (
    file_id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    path VARCHAR(1024) NOT NULL,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, path)
);

-- Folder ownership tracking
CREATE TABLE IF NOT EXISTS folder_ownership (
    folder_id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    path VARCHAR(1024) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, path)
);

-- Create admin user (password: Admin123!)
INSERT INTO users (
    id, 
    username, 
    email, 
    password_hash, 
    role, 
    storage_quota_bytes
) VALUES (
    '00000000-0000-0000-0000-000000000000',
    'admin',
    'admin@oxicloud.local',
    '$argon2id$v=19$m=65536,t=3,p=4$c2FsdHNhbHRzYWx0c2FsdA$H3VxE8LL2qPT31DM3loTg6D+O4MSc2sD7GjlQ5h7Jkw', -- Admin123!
    'admin',
    107374182400  -- 100GB for admin
) ON CONFLICT (id) DO NOTHING;