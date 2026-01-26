-- Authentication nonces table for replay attack prevention
CREATE TABLE auth_nonces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nonce TEXT NOT NULL UNIQUE,
    wallet_address TEXT,
    used BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_auth_nonces_nonce ON auth_nonces(nonce);
CREATE INDEX idx_auth_nonces_expires ON auth_nonces(expires_at);

-- Cleanup job: delete expired nonces (run periodically)
-- DELETE FROM auth_nonces WHERE expires_at < NOW();
