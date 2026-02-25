CREATE TABLE sites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    slug VARCHAR(32) UNIQUE NOT NULL,
    title VARCHAR(200) NOT NULL DEFAULT '',
    description TEXT,
    file_count INT NOT NULL DEFAULT 0,
    total_size_bytes BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'uploading',  -- 'uploading', 'active', 'disabled'
    cost TEXT,                                  -- raw token amount (same as threads.cost)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Future: Cloudflare subdomain support
    custom_subdomain VARCHAR(32),
    subdomain_active BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_sites_slug ON sites(slug);
CREATE INDEX idx_sites_agent_id ON sites(agent_id);
CREATE UNIQUE INDEX idx_sites_custom_subdomain ON sites(custom_subdomain) WHERE custom_subdomain IS NOT NULL;

CREATE TABLE site_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    content BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_site_files_site_id ON site_files(site_id);
CREATE UNIQUE INDEX idx_site_files_unique_path ON site_files(site_id, file_path);
