-- Add indexes for optimization

-- Index for list_by_tenant sorting
CREATE INDEX IF NOT EXISTS idx_roles_tenant_list ON roles(tenant_id, is_system DESC, created_at DESC);

-- Index for searching (basic B-Tree, supports prefix search if pattern allows, 
-- but full wildcard ILIKE needs pg_trgm which we might not have enabled yet. 
-- Adding B-Tree on name helps exact match or prefix if needed later)
CREATE INDEX IF NOT EXISTS idx_roles_name ON roles(name);
