//! Phase 5: Template engine, natural language, composition, and intelligence tests.

use std::collections::HashMap;
use agentic_workflow::template::*;
use agentic_workflow::intelligence::*;
use agentic_workflow::types::*;

// === Template Engine ===

#[test]
fn test_template_create_and_list() {
    let mut engine = TemplateEngine::new();
    let params = vec![TemplateParameter {
        name: "app".into(), description: "App name".into(),
        param_type: agentic_workflow::types::template::ParameterType::String,
        required: true, default: None, validation: None,
    }];
    engine.create_template("deploy", "Deploy app", params, serde_json::json!({"app": "{{app}}"}), vec!["ci".into()], "team").unwrap();
    assert_eq!(engine.list_templates().len(), 1);
}

#[test]
fn test_template_instantiate_substitution() {
    let mut engine = TemplateEngine::new();
    let params = vec![TemplateParameter {
        name: "app".into(), description: "".into(),
        param_type: agentic_workflow::types::template::ParameterType::String,
        required: true, default: None, validation: None,
    }];
    let tid = engine.create_template("deploy", "", params, serde_json::json!({"target": "{{app}}"}), vec![], "").unwrap();
    let mut p = HashMap::new();
    p.insert("app".to_string(), serde_json::json!("my-service"));
    let result = engine.instantiate(&tid, &p).unwrap();
    assert_eq!(result["target"], "my-service");
}

#[test]
fn test_template_missing_required_param() {
    let mut engine = TemplateEngine::new();
    let params = vec![TemplateParameter {
        name: "required_field".into(), description: "".into(),
        param_type: agentic_workflow::types::template::ParameterType::String,
        required: true, default: None, validation: None,
    }];
    let tid = engine.create_template("t", "", params, serde_json::json!({}), vec![], "").unwrap();
    let result = engine.instantiate(&tid, &HashMap::new());
    assert!(result.is_err());
}

#[test]
fn test_template_default_values() {
    let mut engine = TemplateEngine::new();
    let params = vec![TemplateParameter {
        name: "env".into(), description: "".into(),
        param_type: agentic_workflow::types::template::ParameterType::String,
        required: false, default: Some(serde_json::json!("production")), validation: None,
    }];
    let tid = engine.create_template("t", "", params, serde_json::json!({"env": "{{env}}"}), vec![], "").unwrap();
    let result = engine.instantiate(&tid, &HashMap::new()).unwrap();
    assert_eq!(result["env"], "production");
}

#[test]
fn test_template_search_by_tag() {
    let mut engine = TemplateEngine::new();
    engine.create_template("a", "", vec![], serde_json::json!({}), vec!["deploy".into()], "").unwrap();
    engine.create_template("b", "", vec![], serde_json::json!({}), vec!["test".into()], "").unwrap();
    assert_eq!(engine.search_by_tag("deploy").len(), 1);
    assert_eq!(engine.search_by_tag("missing").len(), 0);
}

#[test]
fn test_template_usage_count() {
    let mut engine = TemplateEngine::new();
    let tid = engine.create_template("t", "", vec![], serde_json::json!({}), vec![], "").unwrap();
    engine.instantiate(&tid, &HashMap::new()).unwrap();
    engine.instantiate(&tid, &HashMap::new()).unwrap();
    assert_eq!(engine.get_template(&tid).unwrap().usage_count, 2);
}

#[test]
fn test_template_share() {
    let mut engine = TemplateEngine::new();
    let tid = engine.create_template("t", "", vec![], serde_json::json!({}), vec![], "").unwrap();
    engine.share_template(&tid, "alice").unwrap();
    assert_eq!(engine.list_shared().len(), 1);
}

#[test]
fn test_template_instantiate_unknown() {
    let mut engine = TemplateEngine::new();
    assert!(engine.instantiate("nonexistent", &HashMap::new()).is_err());
}

// === Natural Language Engine ===

#[test]
fn test_nl_create_request() {
    let mut engine = NaturalLanguageEngine::new();
    let idx = engine.create_request("Every morning check inventory");
    assert_eq!(engine.get_request(idx).unwrap().description, "Every morning check inventory");
}

#[test]
fn test_nl_clarification_flow() {
    let mut engine = NaturalLanguageEngine::new();
    let idx = engine.create_request("Deploy to prod");
    engine.add_clarification(idx, "Which environment?", Some(vec!["staging".into(), "production".into()])).unwrap();
    engine.answer_clarification(idx, 0, "production").unwrap();
    assert_eq!(engine.get_request(idx).unwrap().clarifications[0].answer.as_deref(), Some("production"));
}

#[test]
fn test_nl_set_synthesized() {
    let mut engine = NaturalLanguageEngine::new();
    let idx = engine.create_request("Build and test");
    engine.set_synthesized(idx, serde_json::json!({"steps": ["build", "test"]})).unwrap();
    assert!(engine.get_request(idx).unwrap().synthesized_workflow.is_some());
}

// === Composition Engine ===

#[test]
fn test_compose_sequence() {
    let mut engine = CompositionEngine::new();
    let mid = engine.sequence("pipeline", vec!["build".into(), "test".into(), "deploy".into()]).unwrap();
    let meta = engine.get_meta(&mid).unwrap();
    assert_eq!(meta.name, "pipeline");
}

#[test]
fn test_compose_parallel() {
    let mut engine = CompositionEngine::new();
    let mid = engine.parallel("fan-out", vec!["a".into(), "b".into(), "c".into()]).unwrap();
    assert!(engine.get_meta(&mid).is_ok());
}

#[test]
fn test_compose_conditional() {
    let mut engine = CompositionEngine::new();
    let mid = engine.conditional("branch", "is_prod", "deploy-prod", "deploy-staging").unwrap();
    assert!(engine.get_meta(&mid).is_ok());
}

#[test]
fn test_compose_data_bridge() {
    let mut engine = CompositionEngine::new();
    let mid = engine.sequence("p", vec!["a".into(), "b".into()]).unwrap();
    engine.add_bridge(&mid, "a", "output", "b", "input", None).unwrap();
    assert_eq!(engine.get_meta(&mid).unwrap().data_bridges.len(), 1);
}

#[test]
fn test_compose_validate() {
    let mut engine = CompositionEngine::new();
    let mid = engine.sequence("p", vec!["a".into()]).unwrap();
    let warnings = engine.validate(&mid).unwrap();
    assert!(warnings.is_empty());
}

// === Archaeology Engine ===

#[test]
fn test_archaeology_compare_executions() {
    let mut engine = ArchaeologyEngine::new();
    let mut d1 = HashMap::new(); d1.insert("s1".into(), 100u64);
    let mut d2 = HashMap::new(); d2.insert("s1".into(), 500u64);
    engine.record_fingerprint(ExecutionFingerprint {
        execution_id: "e1".into(), workflow_id: "wf".into(), total_duration_ms: 100,
        step_durations: d1, step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
    });
    engine.record_fingerprint(ExecutionFingerprint {
        execution_id: "e2".into(), workflow_id: "wf".into(), total_duration_ms: 500,
        step_durations: d2, step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
    });
    let cmp = engine.compare("e1", "e2").unwrap();
    assert_eq!(cmp.duration_ratio, 5.0);
    assert!(!cmp.significant_step_diffs.is_empty());
}

#[test]
fn test_archaeology_anomaly_detection() {
    let mut engine = ArchaeologyEngine::new();
    for i in 0..5 {
        engine.record_fingerprint(ExecutionFingerprint {
            execution_id: format!("e{}", i), workflow_id: "wf".into(), total_duration_ms: 1000,
            step_durations: HashMap::new(), step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
        });
    }
    engine.record_fingerprint(ExecutionFingerprint {
        execution_id: "outlier".into(), workflow_id: "wf".into(), total_duration_ms: 100_000,
        step_durations: HashMap::new(), step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
    });
    let anomalies = engine.detect_anomalies("wf");
    assert!(!anomalies.is_empty());
}

#[test]
fn test_archaeology_bottleneck_identification() {
    let mut engine = ArchaeologyEngine::new();
    let mut durations = HashMap::new();
    durations.insert("fast".into(), 10u64);
    durations.insert("slow".into(), 990u64);
    engine.record_fingerprint(ExecutionFingerprint {
        execution_id: "e1".into(), workflow_id: "wf".into(), total_duration_ms: 1000,
        step_durations: durations, step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
    });
    let bottlenecks = engine.bottlenecks("wf");
    assert_eq!(bottlenecks[0].step_id, "slow");
}

// === Prediction Engine ===

#[test]
fn test_prediction_duration() {
    let mut engine = PredictionEngine::new();
    for i in 0..10 {
        engine.ingest_fingerprint(ExecutionFingerprint {
            execution_id: format!("e{}", i), workflow_id: "wf".into(), total_duration_ms: 1000 + i * 100,
            step_durations: HashMap::new(), step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
        });
    }
    let pred = engine.predict_duration("wf").unwrap();
    assert!(pred.predicted_ms > 0);
    assert!(pred.confidence > 0.0);
    assert_eq!(pred.based_on_executions, 10);
}

#[test]
fn test_prediction_no_history() {
    let engine = PredictionEngine::new();
    let pred = engine.predict_duration("unknown").unwrap();
    assert_eq!(pred.based_on_executions, 0);
    assert_eq!(pred.confidence, 0.0);
}

#[test]
fn test_prediction_cost() {
    let engine = PredictionEngine::new();
    let cost = engine.predict_cost("wf").unwrap();
    assert!(cost.estimated_cost_usd >= 0.0);
}

// === Evolution Engine ===

#[test]
fn test_evolution_health_score() {
    let mut engine = EvolutionEngine::new();
    for i in 0..10 {
        engine.ingest(ExecutionFingerprint {
            execution_id: format!("e{}", i), workflow_id: "wf".into(), total_duration_ms: 1000,
            step_durations: HashMap::new(), step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
        });
    }
    let health = engine.health("wf").unwrap();
    assert!(health.score > 0.0);
    assert!(health.success_rate > 0.0);
}

#[test]
fn test_evolution_drift_detection() {
    let mut engine = EvolutionEngine::new();
    for i in 0..4 { engine.ingest(ExecutionFingerprint {
        execution_id: format!("old{}", i), workflow_id: "wf".into(), total_duration_ms: 100,
        step_durations: HashMap::new(), step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
    }); }
    for i in 0..4 { engine.ingest(ExecutionFingerprint {
        execution_id: format!("new{}", i), workflow_id: "wf".into(), total_duration_ms: 10_000,
        step_durations: HashMap::new(), step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
    }); }
    assert!(engine.detect_drift("wf"));
}

#[test]
fn test_evolution_no_data_health() {
    let engine = EvolutionEngine::new();
    let health = engine.health("unknown").unwrap();
    assert_eq!(health.score, 1.0);
}

// === Dream Engine ===

#[test]
fn test_dream_insights() {
    let mut engine = DreamEngine::new();
    engine.add_insight("wf-1", dream::InsightType::DependencyHealth, "API key expiring", "warning");
    assert_eq!(engine.get_insights().len(), 1);
    assert_eq!(engine.insights_for_workflow("wf-1").len(), 1);
    assert_eq!(engine.insights_for_workflow("wf-2").len(), 0);
    engine.clear();
    assert_eq!(engine.get_insights().len(), 0);
}

// === Collective Engine ===

#[test]
fn test_collective_share_and_search() {
    let mut engine = CollectiveEngine::new();
    engine.share("CI Pipeline", "Standard CI", serde_json::json!({"steps": ["build"]}), "alice", vec!["ci".into()]);
    let results = engine.search("pipeline");
    assert_eq!(results.len(), 1);
    assert_eq!(engine.search("nonexistent").len(), 0);
}

#[test]
fn test_collective_rate_average() {
    let mut engine = CollectiveEngine::new();
    let id = engine.share("test", "", serde_json::json!({}), "", vec![]);
    engine.rate(&id, 4.0);
    engine.rate(&id, 5.0);
    let item = engine.get(&id).unwrap();
    assert!((item.rating - 4.5).abs() < 0.01);
}

#[test]
fn test_collective_privacy_detect_secrets() {
    let mut engine = CollectiveEngine::new();
    let id = engine.share("leak", "", serde_json::json!({"password": "secret123"}), "", vec![]);
    assert!(!engine.verify_privacy(&id));
}

#[test]
fn test_collective_privacy_clean() {
    let mut engine = CollectiveEngine::new();
    let id = engine.share("clean", "", serde_json::json!({"steps": ["build"]}), "", vec![]);
    assert!(engine.verify_privacy(&id));
}

#[test]
fn test_collective_apply_increments_downloads() {
    let mut engine = CollectiveEngine::new();
    let id = engine.share("t", "", serde_json::json!({}), "", vec![]);
    engine.apply(&id);
    engine.apply(&id);
    assert_eq!(engine.get(&id).unwrap().download_count, 2);
}
