use std::sync::Arc;
use vrcx_0_application_core::{DatabaseMaintenancePort, RuntimeOperationStatus};
use vrcx_0_persistence::DatabaseService;

use super::RuntimeHostState;

struct RuntimeDatabaseMaintenanceAdapter {
    db: Arc<DatabaseService>,
}

impl DatabaseMaintenancePort for RuntimeDatabaseMaintenanceAdapter {
    fn optimize(&self) -> vrcx_0_application_core::Result<()> {
        vrcx_0_persistence::optimize_database(self.db.as_ref())
            .map(|_| ())
            .map_err(Into::into)
    }

    fn checkpoint_wal(&self) -> vrcx_0_application_core::Result<()> {
        self.db.checkpoint_wal().map(|_| ()).map_err(Into::into)
    }
}

impl RuntimeHostState {
    pub fn release_profile_lock(&self) {
        self._profile_lock.release();
    }

    pub fn start_data_services(&self) {
        self.runtime_context
            .runtime
            .set_host_services_started(true, "Runtime host services installed.");
        self.runtime_context
            .background_jobs
            .register_frontend_job_catalog();
        self.runtime_context.background_jobs.register_job(
            "startupRecovery",
            "rust-host",
            None,
            RuntimeOperationStatus::Checkpoint,
            "Rust runtime startup recovery checkpoint recorded; no durable recovery queue is configured.",
        );
        self.runtime_context.runtime.record_phase(
            "startupRecovery",
            RuntimeOperationStatus::Checkpoint,
            "Rust runtime startup recovery checkpoint recorded; no durable recovery queue is configured.",
        );
        self.runtime_context.sync.record(
            "startupRecovery",
            RuntimeOperationStatus::Observed,
            "Rust runtime startup recovery checkpoint recorded; no durable recovery queue is configured.",
            0,
        );
        let database_maintenance: Arc<dyn DatabaseMaintenancePort> =
            Arc::new(RuntimeDatabaseMaintenanceAdapter {
                db: Arc::clone(&self.db),
            });
        self.runtime_context
            .background_jobs
            .start_database_optimize_loop(
                Arc::clone(&database_maintenance),
                self.runtime_context.tasks.clone(),
            );
        self.runtime_context
            .background_jobs
            .start_database_checkpoint_loop(
                database_maintenance,
                self.runtime_context.tasks.clone(),
            );
        self.runtime_context
            .vrc_status
            .start_loop(self.runtime_context.tasks.clone());
        self.profile_backup.start_scheduler();
    }
}
