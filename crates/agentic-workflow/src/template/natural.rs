use crate::types::{
    Clarification, NaturalLanguageRequest, WorkflowResult,
};

/// Natural language workflow creation engine.
/// Note: actual NL parsing delegates to LLM (per CLAUDE.md no-hardcoded-intelligence rule).
pub struct NaturalLanguageEngine {
    requests: Vec<NaturalLanguageRequest>,
}

impl NaturalLanguageEngine {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    /// Start a natural language workflow request.
    pub fn create_request(&mut self, description: &str) -> usize {
        let request = NaturalLanguageRequest {
            description: description.to_string(),
            clarifications: Vec::new(),
            synthesized_workflow: None,
        };

        self.requests.push(request);
        self.requests.len() - 1
    }

    /// Add a clarification question.
    pub fn add_clarification(
        &mut self,
        request_idx: usize,
        question: &str,
        options: Option<Vec<String>>,
    ) -> WorkflowResult<()> {
        let request = self.requests.get_mut(request_idx).ok_or_else(|| {
            crate::types::WorkflowError::Internal("Request not found".to_string())
        })?;

        request.clarifications.push(Clarification {
            question: question.to_string(),
            answer: None,
            options,
        });

        Ok(())
    }

    /// Answer a clarification question.
    pub fn answer_clarification(
        &mut self,
        request_idx: usize,
        clarification_idx: usize,
        answer: &str,
    ) -> WorkflowResult<()> {
        let request = self.requests.get_mut(request_idx).ok_or_else(|| {
            crate::types::WorkflowError::Internal("Request not found".to_string())
        })?;

        let clarification = request
            .clarifications
            .get_mut(clarification_idx)
            .ok_or_else(|| {
                crate::types::WorkflowError::Internal("Clarification not found".to_string())
            })?;

        clarification.answer = Some(answer.to_string());
        Ok(())
    }

    /// Set the synthesized workflow (produced by LLM).
    pub fn set_synthesized(
        &mut self,
        request_idx: usize,
        workflow: serde_json::Value,
    ) -> WorkflowResult<()> {
        let request = self.requests.get_mut(request_idx).ok_or_else(|| {
            crate::types::WorkflowError::Internal("Request not found".to_string())
        })?;

        request.synthesized_workflow = Some(workflow);
        Ok(())
    }

    /// Get a request.
    pub fn get_request(&self, request_idx: usize) -> Option<&NaturalLanguageRequest> {
        self.requests.get(request_idx)
    }

    /// Get all requests.
    pub fn list_requests(&self) -> &[NaturalLanguageRequest] {
        &self.requests
    }
}

impl Default for NaturalLanguageEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nl_request_flow() {
        let mut engine = NaturalLanguageEngine::new();
        let idx = engine.create_request(
            "Every morning, check inventory and reorder items below threshold",
        );

        engine
            .add_clarification(idx, "What threshold should trigger reorder?", None)
            .unwrap();

        engine
            .answer_clarification(idx, 0, "When stock is below 50 units")
            .unwrap();

        let req = engine.get_request(idx).unwrap();
        assert_eq!(req.clarifications.len(), 1);
        assert_eq!(
            req.clarifications[0].answer.as_deref(),
            Some("When stock is below 50 units")
        );
    }
}
