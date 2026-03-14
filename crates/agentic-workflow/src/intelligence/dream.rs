use serde::Serialize;

/// Dream state engine — idle-time workflow maintenance.
pub struct DreamEngine {
    insights: Vec<DreamInsight>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DreamInsight {
    pub workflow_id: String,
    pub insight_type: InsightType,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum InsightType {
    DependencyHealth,
    ConfigurationDrift,
    ScheduleOptimization,
    UnusedWorkflow,
    SecurityConcern,
}

impl DreamEngine {
    pub fn new() -> Self {
        Self {
            insights: Vec::new(),
        }
    }

    /// Record a proactive insight.
    pub fn add_insight(
        &mut self,
        workflow_id: &str,
        insight_type: InsightType,
        message: &str,
        severity: &str,
    ) {
        self.insights.push(DreamInsight {
            workflow_id: workflow_id.to_string(),
            insight_type,
            message: message.to_string(),
            severity: severity.to_string(),
        });
    }

    /// Get all insights.
    pub fn get_insights(&self) -> &[DreamInsight] {
        &self.insights
    }

    /// Get insights for a specific workflow.
    pub fn insights_for_workflow(&self, workflow_id: &str) -> Vec<&DreamInsight> {
        self.insights
            .iter()
            .filter(|i| i.workflow_id == workflow_id)
            .collect()
    }

    /// Clear consumed insights.
    pub fn clear(&mut self) {
        self.insights.clear();
    }
}

impl Default for DreamEngine {
    fn default() -> Self {
        Self::new()
    }
}
