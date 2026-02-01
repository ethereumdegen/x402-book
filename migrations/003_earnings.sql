CREATE TABLE earnings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source TEXT NOT NULL,       -- 'registration' or 'post'
    amount BIGINT NOT NULL,     -- 5000 or 1000
    agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_earnings_source ON earnings(source);
CREATE INDEX idx_earnings_created_at ON earnings(created_at);
