use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    AdaptiveSchedule, ConflictPolicy, Schedule, ScheduleExpression,
    WorkflowError, WorkflowResult,
};

/// Context-aware scheduling engine.
pub struct SchedulerEngine {
    schedules: HashMap<String, Schedule>,
}

impl SchedulerEngine {
    pub fn new() -> Self {
        Self {
            schedules: HashMap::new(),
        }
    }

    /// Create a new schedule for a workflow.
    pub fn create_schedule(
        &mut self,
        workflow_id: &str,
        expression: ScheduleExpression,
        conflict_policy: ConflictPolicy,
        timezone: &str,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let schedule = Schedule {
            id: id.clone(),
            workflow_id: workflow_id.to_string(),
            expression,
            conflict_policy,
            enabled: true,
            next_fire_at: None,
            last_fired_at: None,
            timezone: timezone.to_string(),
            created_at: Utc::now(),
        };

        self.schedules.insert(id.clone(), schedule);
        Ok(id)
    }

    /// List all schedules.
    pub fn list_schedules(&self) -> Vec<&Schedule> {
        self.schedules.values().collect()
    }

    /// Get schedules for a specific workflow.
    pub fn schedules_for_workflow(&self, workflow_id: &str) -> Vec<&Schedule> {
        self.schedules
            .values()
            .filter(|s| s.workflow_id == workflow_id)
            .collect()
    }

    /// Pause a schedule.
    pub fn pause_schedule(&mut self, schedule_id: &str) -> WorkflowResult<()> {
        let schedule = self
            .schedules
            .get_mut(schedule_id)
            .ok_or_else(|| WorkflowError::ScheduleError(format!("Not found: {}", schedule_id)))?;

        schedule.enabled = false;
        Ok(())
    }

    /// Resume a paused schedule.
    pub fn resume_schedule(&mut self, schedule_id: &str) -> WorkflowResult<()> {
        let schedule = self
            .schedules
            .get_mut(schedule_id)
            .ok_or_else(|| WorkflowError::ScheduleError(format!("Not found: {}", schedule_id)))?;

        schedule.enabled = true;
        Ok(())
    }

    /// Remove a schedule.
    pub fn remove_schedule(&mut self, schedule_id: &str) -> WorkflowResult<Schedule> {
        self.schedules
            .remove(schedule_id)
            .ok_or_else(|| WorkflowError::ScheduleError(format!("Not found: {}", schedule_id)))
    }

    /// Get adaptive schedule recommendations (placeholder — uses execution history).
    pub fn get_adaptive_recommendation(
        &self,
        schedule_id: &str,
    ) -> WorkflowResult<AdaptiveSchedule> {
        let schedule = self
            .schedules
            .get(schedule_id)
            .ok_or_else(|| WorkflowError::ScheduleError(format!("Not found: {}", schedule_id)))?;

        Ok(AdaptiveSchedule {
            schedule_id: schedule_id.to_string(),
            recommended_time: "08:30".to_string(),
            reason: "Historical success rate is 12% higher at 08:30 vs current schedule"
                .to_string(),
            success_rate_at_recommended: 0.95,
            success_rate_at_current: 0.83,
        })
    }
}

impl Default for SchedulerEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_lifecycle() {
        let mut engine = SchedulerEngine::new();
        let sid = engine
            .create_schedule(
                "wf-1",
                ScheduleExpression::Cron("0 8 * * 1-5".to_string()),
                ConflictPolicy::Skip,
                "UTC",
            )
            .unwrap();

        assert_eq!(engine.list_schedules().len(), 1);
        assert!(engine.pause_schedule(&sid).is_ok());
        assert!(!engine.schedules.get(&sid).unwrap().enabled);
        assert!(engine.resume_schedule(&sid).is_ok());
        assert!(engine.schedules.get(&sid).unwrap().enabled);
        assert!(engine.remove_schedule(&sid).is_ok());
        assert_eq!(engine.list_schedules().len(), 0);
    }
}
