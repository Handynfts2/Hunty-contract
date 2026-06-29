// ... at the top with other imports
use crate::migration::{CURRENT_SCHEMA_VERSION, MigrationFramework, UpgradeAuthorization};

// In the HuntyCore impl, ensure initialization calls migration setup:

pub fn initialize(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
    // Existing admin init...
    if Storage::get_admin(&env).is_some() {
        return Err(HuntErrorCode::Unauthorized);
    }
    Storage::set_admin(&env, &admin);

    // NEW: Initialize schema version
    MigrationFramework::init_version_on_deploy(&env);
    UpgradeAuthorization::set_upgrade_admin(&env, &admin);

    Ok(())
}

// Add these public migration methods (near the end of the contractimpl):

    /// Get current schema version
    pub fn get_schema_version(env: Env) -> u32 {
        MigrationFramework::detect_version(&env)
    }

    /// Initialize or bump schema version (admin only)
    pub fn initialize_schema(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;
        MigrationFramework::init_version_on_deploy(&env);
        Ok(())
    }

    /// Run migration steps to bring storage to target version
    pub fn run_migration(
        env: Env,
        admin: Address,
        target_version: u32,
        dry_run: bool,
    ) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
        Self::require_admin(&env, &admin)?;

        let current = MigrationFramework::detect_version(&env);
        if current == target_version && !dry_run {
            return Ok(MigrationFramework::build_report(
                &env, current, target_version, 0, dry_run, true, "Already at target version"
            ));
        }

        // Example migration logic - extend this for future schema changes
        let mut steps = 0u32;

        if current < 2 && target_version >= 2 {
            // Example: Migrate old player progress structure
            Self::migrate_v1_to_v2(&env, dry_run)?;
            steps += 1;
        }

        if !dry_run {
            MigrationFramework::set_version(&env, target_version);
        }

        Ok(MigrationFramework::build_report(
            &env,
            current,
            target_version,
            steps,
            dry_run,
            true,
            "Migration completed successfully",
        ))
    }

    // Example migration step (expand as needed for new schema changes)
    fn migrate_v1_to_v2(env: &Env, dry_run: bool) -> Result<(), HuntErrorCode> {
        if dry_run {
            return Ok(());
        }
        // Add data transformation logic here, e.g.:
        // - Update existing Hunt structs
        // - Re-key old storage entries
        // - Add new fields with defaults
        Ok(())
    }

    /// Rollback to previous schema version if needed
    pub fn rollback_migration(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;
        if let Some(prev) = MigrationFramework::rollback_version(&env) {
            MigrationFramework::set_version(&env, prev);
            MigrationFramework::clear_rollback(&env);
        }
        Ok(())
    }