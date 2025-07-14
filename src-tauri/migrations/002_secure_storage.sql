-- Secure Storage Migration
-- This migration adds a secure storage table for persistent encrypted data storage
-- Replaces the in-memory HashMap implementation with database persistence

-- Secure storage table for encrypted data persistence
CREATE TABLE secure_storage (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL,
    data_type TEXT NOT NULL,
    storage_key TEXT NOT NULL UNIQUE, -- Generated key: secure_{data_type}_{user_id}
    encrypted_data TEXT NOT NULL, -- Base64 encoded encrypted data
    nonce TEXT NOT NULL, -- Base64 encoded nonce
    algorithm TEXT NOT NULL, -- Encryption algorithm used
    key_id TEXT NOT NULL, -- Key ID for encryption key management
    stored_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME, -- Optional expiration for automatic cleanup
    access_count INTEGER NOT NULL DEFAULT 0, -- Track access frequency
    last_accessed_at DATETIME, -- Track last access time
    
    -- Foreign key constraint to users table
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    
    -- Ensure unique combination of user_id and data_type
    UNIQUE(user_id, data_type)
);

-- Indexes for performance optimization
CREATE INDEX idx_secure_storage_user_id ON secure_storage(user_id);
CREATE INDEX idx_secure_storage_data_type ON secure_storage(data_type);
CREATE INDEX idx_secure_storage_storage_key ON secure_storage(storage_key);
CREATE INDEX idx_secure_storage_expires_at ON secure_storage(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_secure_storage_last_accessed ON secure_storage(last_accessed_at);

-- Trigger to update the updated_at timestamp
CREATE TRIGGER secure_storage_updated_at
    AFTER UPDATE ON secure_storage
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE secure_storage
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- Trigger to update access tracking
CREATE TRIGGER secure_storage_access_tracking
    AFTER UPDATE OF access_count ON secure_storage
    FOR EACH ROW
    WHEN NEW.access_count != OLD.access_count
BEGIN
    UPDATE secure_storage
    SET last_accessed_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- View for secure storage statistics (useful for monitoring)
CREATE VIEW secure_storage_stats AS
SELECT 
    user_id,
    data_type,
    COUNT(*) as total_entries,
    MAX(stored_at) as latest_storage,
    MAX(last_accessed_at) as latest_access,
    AVG(access_count) as avg_access_count,
    COUNT(CASE WHEN expires_at IS NOT NULL AND expires_at < CURRENT_TIMESTAMP THEN 1 END) as expired_entries
FROM secure_storage
GROUP BY user_id, data_type;
