CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    job_reference VARCHAR(255) NOT NULL,
    client_name VARCHAR(255),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Input (stored as JSONB for flexibility)
    request JSONB NOT NULL,

    -- Output (populated when completed)
    result JSONB,
    pdf_bytes BYTEA,
    error_message TEXT,

    -- Webhook (optional)
    webhook_url TEXT,
    webhook_delivered BOOLEAN NOT NULL DEFAULT FALSE,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);
