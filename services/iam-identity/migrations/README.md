# IAM-Identity Database Migrations

This directory contains SQL migration files for the IAM-Identity service database schema.

## Migration Naming Convention

Migrations follow the format: `YYYYMMDDHHMMSS_description.sql`

Example: `20260127000001_fix_oauth_schema_issues.sql`

## Migration Order

Migrations are executed in chronological order based on the timestamp prefix:

1. `20250101000001_create_users_table.sql` - Initial users table
2. `20250101000002_create_sessions_table.sql` - Sessions for authentication
3. `20260126011629_add_2fa_support.sql` - Two-factor authentication
4. `20260126021500_create_password_reset_tokens_table.sql` - Password reset
5. `20260126030000_create_webauthn_credentials_table.sql` - WebAuthn/Passkeys
6. `20260126050000_add_account_lock_fields.sql` - Account security
7. `20260126052917_create_tenants.sql` - Multi-tenancy support
8. `20260126052918_add_tenant_id_to_tables.sql` - Tenant isolation
9. `20260126060000_create_login_logs_table.sql` - Login audit logs
10. `20260126070000_create_email_verifications_table.sql` - Email verification
11. `20260126070001_create_phone_verifications_table.sql` - Phone verification
12. `20260126080000_create_oauth_clients_table.sql` - OAuth2 clients
13. `20260126080001_create_authorization_codes_table.sql` - OAuth2 authorization codes
14. `20260126080002_create_access_tokens_table.sql` - OAuth2 access tokens
15. `20260126080003_create_refresh_tokens_table.sql` - OAuth2 refresh tokens
16. `20260126085000_add_tenant_id_to_new_tables.sql` - Tenant support for OAuth
17. `20260126090000_enable_rls_with_tenant.sql` - Row-Level Security
18. `20260126100000_fix_missing_columns.sql` - Schema fixes
19. `20260127000001_fix_oauth_schema_issues.sql` - OAuth schema type fixes

## Running Migrations

### Using sqlx-cli (Recommended)

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Run all pending migrations
sqlx migrate run --database-url "postgresql://cuba:cuba@localhost:5432/cuba"

# Revert the last migration
sqlx migrate revert --database-url "postgresql://cuba:cuba@localhost:5432/cuba"

# Check migration status
sqlx migrate info --database-url "postgresql://cuba:cuba@localhost:5432/cuba"
```

### Manual Execution

```bash
# Run a specific migration
psql -U postgres -d cuba -f migrations/20260127000001_fix_oauth_schema_issues.sql

# Run all migrations in order
for file in migrations/*.sql; do
    echo "Running $file..."
    psql -U postgres -d cuba -f "$file"
done
```

## Creating New Migrations

1. Create a new file with timestamp and description:
   ```bash
   touch migrations/$(date +%Y%m%d%H%M%S)_your_description.sql
   ```

2. Write your SQL migration:
   ```sql
   -- Description of what this migration does

   ALTER TABLE your_table ADD COLUMN new_column TEXT;

   COMMENT ON COLUMN your_table.new_column IS 'Description';
   ```

3. Test the migration:
   ```bash
   sqlx migrate run --database-url "postgresql://cuba:cuba@localhost:5432/cuba"
   ```

4. Commit the migration file with your code changes

## Migration Best Practices

1. **Idempotent Operations**: Use `IF EXISTS` / `IF NOT EXISTS` where possible
   ```sql
   ALTER TABLE users ADD COLUMN IF NOT EXISTS new_field TEXT;
   ```

2. **Data Preservation**: When changing column types, use `USING` clause
   ```sql
   ALTER TABLE table_name
       ALTER COLUMN column_name TYPE new_type
       USING column_name::new_type;
   ```

3. **Comments**: Always add comments to explain schema changes
   ```sql
   COMMENT ON COLUMN table.column IS 'Explanation';
   ```

4. **Indexes**: Create indexes for foreign keys and frequently queried columns
   ```sql
   CREATE INDEX idx_table_column ON table(column);
   ```

5. **Row-Level Security**: Enable RLS for multi-tenant tables
   ```sql
   ALTER TABLE table_name ENABLE ROW LEVEL SECURITY;
   CREATE POLICY policy_name ON table_name
       USING (tenant_id::text = current_setting('app.current_tenant_id', TRUE));
   ```

## Troubleshooting

### Migration Failed

If a migration fails:

1. Check the error message
2. Fix the SQL in the migration file
3. Manually revert any partial changes
4. Re-run the migration

### Type Conversion Issues

The OAuth schema migrations (20260127000001) fix a common issue where:
- Original migrations used `TEXT[]` (array type)
- Code implementation expects `TEXT` (space-separated string)

If you encounter type errors, ensure you've run migration `20260127000001_fix_oauth_schema_issues.sql`.

### Column Name Mismatches

- `access_tokens.scopes` was renamed to `access_tokens.scope` to match code expectations
- Migration `20260127000001` handles this automatically

## Schema Documentation

For detailed schema documentation, see:
- [Database Schema](../docs/database-schema.md)
- [OAuth2 Implementation](../docs/oauth2-implementation.md)
