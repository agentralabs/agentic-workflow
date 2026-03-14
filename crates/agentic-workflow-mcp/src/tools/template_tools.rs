use serde_json::json;
use std::collections::HashMap;

use crate::types::{ToolDefinition, ToolResult};
use super::registry::EngineState;

fn def(name: &str, desc: &str, props: serde_json::Value) -> ToolDefinition {
    ToolDefinition {
        name: name.to_string(),
        description: desc.to_string(),
        input_schema: json!({ "type": "object", "properties": props }),
    }
}

fn s(d: &str) -> serde_json::Value { json!({ "type": "string", "description": d }) }

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Template (5)
        def("workflow_template_list", "List available workflow templates",
            json!({ "tag": s("Filter by tag") })),
        def("workflow_template_use", "Instantiate a workflow from a template with parameters",
            json!({ "template_id": s("Template ID"), "params": json!({ "type": "object", "description": "Template parameters" }) })),
        def("workflow_template_create", "Create a reusable workflow template",
            json!({ "name": s("Template name"), "description": s("Description"), "workflow_definition": json!({ "type": "object", "description": "Workflow JSON" }), "tags": json!({ "type": "array", "items": { "type": "string" } }), "author": s("Author") })),
        def("workflow_template_share", "Share a template with the community",
            json!({ "template_id": s("Template ID"), "shared_by": s("Sharer identity") })),
        def("workflow_template_compose", "Compose multiple templates into a pipeline",
            json!({ "template_ids": json!({ "type": "array", "items": { "type": "string" }, "description": "Template IDs to compose" }), "name": s("Composed workflow name") })),
        // Natural language (5)
        def("workflow_natural_create", "Create a workflow from a natural language description",
            json!({ "description": s("Plain-English workflow description") })),
        def("workflow_natural_preview", "Preview the synthesized workflow from a natural language request",
            json!({ "request_index": json!({ "type": "integer", "description": "Request index" }) })),
        def("workflow_natural_clarify", "Add a clarification question to a natural language request",
            json!({ "request_index": json!({ "type": "integer", "description": "Request index" }), "question": s("Clarification question"), "options": json!({ "type": "array", "items": { "type": "string" } }) })),
        def("workflow_natural_refine", "Answer a clarification to refine the workflow",
            json!({ "request_index": json!({ "type": "integer", "description": "Request index" }), "clarification_index": json!({ "type": "integer", "description": "Clarification index" }), "answer": s("Answer to clarification") })),
        // Composition algebra (5)
        def("workflow_compose_sequence", "Compose workflows in sequence: A then B then C",
            json!({ "name": s("Composition name"), "workflow_ids": json!({ "type": "array", "items": { "type": "string" }, "description": "Workflow IDs in order" }) })),
        def("workflow_compose_parallel", "Compose workflows in parallel: A and B and C simultaneously",
            json!({ "name": s("Composition name"), "workflow_ids": json!({ "type": "array", "items": { "type": "string" } }) })),
        def("workflow_compose_conditional", "Compose workflows conditionally: if P then A else B",
            json!({ "name": s("Composition name"), "predicate": s("Condition expression"), "if_true": s("Workflow if true"), "if_false": s("Workflow if false") })),
        def("workflow_compose_validate", "Validate a composed meta-workflow",
            json!({ "meta_id": s("Meta-workflow ID") })),
        def("workflow_compose_run", "Execute a composed meta-workflow",
            json!({ "meta_id": s("Meta-workflow ID") })),
        // Collective (4)
        def("workflow_collective_share", "Share a workflow with the collective community",
            json!({ "name": s("Workflow name"), "description": s("Description"), "workflow_definition": json!({ "type": "object", "description": "Workflow JSON" }), "author": s("Author"), "tags": json!({ "type": "array", "items": { "type": "string" } }) })),
        def("workflow_collective_search", "Search community-shared workflows",
            json!({ "query": s("Search query") })),
        def("workflow_collective_apply", "Download and apply a community workflow",
            json!({ "id": s("Collective item ID") })),
        def("workflow_collective_rate", "Rate a community workflow",
            json!({ "id": s("Collective item ID"), "rating": json!({ "type": "number", "description": "Rating 0-5" }) })),
        def("workflow_collective_private", "Verify no private data in a shared workflow",
            json!({ "id": s("Collective item ID") })),
    ]
}

pub fn dispatch(
    name: &str,
    params: serde_json::Value,
    state: &mut EngineState,
) -> Result<ToolResult, (i32, String)> {
    match name {
        // --- Template ---
        "workflow_template_list" => {
            let templates = if let Some(tag) = params["tag"].as_str() {
                state.template.search_by_tag(tag)
            } else {
                state.template.list_templates()
            };
            let items: Vec<_> = templates.iter().map(|t| json!({
                "id": t.id, "name": t.name, "tags": t.tags, "usage_count": t.usage_count
            })).collect();
            Ok(ToolResult::text(json!({ "templates": items }).to_string()))
        }
        "workflow_template_use" => {
            let tid = params["template_id"].as_str().unwrap_or("");
            let p: HashMap<String, serde_json::Value> = params["params"]
                .as_object()
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();
            match state.template.instantiate(tid, &p) {
                Ok(wf) => Ok(ToolResult::text(json!({
                    "template_id": tid, "workflow": wf, "status": "instantiated"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_template_create" => {
            let tname = params["name"].as_str().unwrap_or("");
            let desc = params["description"].as_str().unwrap_or("");
            let wf_def = params["workflow_definition"].clone();
            let tags: Vec<String> = params["tags"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let author = params["author"].as_str().unwrap_or("anonymous");
            match state.template.create_template(tname, desc, Vec::new(), wf_def, tags, author) {
                Ok(tid) => Ok(ToolResult::text(json!({ "template_id": tid, "status": "created" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_template_share" => {
            let tid = params["template_id"].as_str().unwrap_or("");
            let by = params["shared_by"].as_str().unwrap_or("anonymous");
            match state.template.share_template(tid, by) {
                Ok(sid) => Ok(ToolResult::text(json!({ "shared_id": sid, "status": "shared" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_template_compose" => {
            let ids: Vec<String> = params["template_ids"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let cname = params["name"].as_str().unwrap_or("composed");
            match state.composer.sequence(cname, ids) {
                Ok(mid) => Ok(ToolResult::text(json!({ "meta_id": mid, "status": "composed" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        // --- Natural language ---
        "workflow_natural_create" => {
            let desc = params["description"].as_str().unwrap_or("");
            let idx = state.natural.create_request(desc);
            Ok(ToolResult::text(json!({ "request_index": idx, "status": "created" }).to_string()))
        }
        "workflow_natural_preview" => {
            let idx = params["request_index"].as_u64().unwrap_or(0) as usize;
            match state.natural.get_request(idx) {
                Some(req) => Ok(ToolResult::text(json!({
                    "description": req.description,
                    "clarifications": req.clarifications.len(),
                    "synthesized": req.synthesized_workflow.is_some()
                }).to_string())),
                None => Ok(ToolResult::error("Request not found")),
            }
        }
        "workflow_natural_clarify" => {
            let idx = params["request_index"].as_u64().unwrap_or(0) as usize;
            let question = params["question"].as_str().unwrap_or("");
            let options = params["options"].as_array().map(|a| {
                a.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            });
            match state.natural.add_clarification(idx, question, options) {
                Ok(()) => Ok(ToolResult::text(json!({ "status": "clarification_added" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_natural_refine" => {
            let idx = params["request_index"].as_u64().unwrap_or(0) as usize;
            let cidx = params["clarification_index"].as_u64().unwrap_or(0) as usize;
            let answer = params["answer"].as_str().unwrap_or("");
            match state.natural.answer_clarification(idx, cidx, answer) {
                Ok(()) => Ok(ToolResult::text(json!({ "status": "refined" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        // --- Composition ---
        "workflow_compose_sequence" => {
            let cname = params["name"].as_str().unwrap_or("sequence");
            let ids: Vec<String> = params["workflow_ids"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            match state.composer.sequence(cname, ids) {
                Ok(mid) => Ok(ToolResult::text(json!({ "meta_id": mid, "status": "created" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_compose_parallel" => {
            let cname = params["name"].as_str().unwrap_or("parallel");
            let ids: Vec<String> = params["workflow_ids"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            match state.composer.parallel(cname, ids) {
                Ok(mid) => Ok(ToolResult::text(json!({ "meta_id": mid, "status": "created" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_compose_conditional" => {
            let cname = params["name"].as_str().unwrap_or("conditional");
            let pred = params["predicate"].as_str().unwrap_or("true");
            let if_t = params["if_true"].as_str().unwrap_or("");
            let if_f = params["if_false"].as_str().unwrap_or("");
            match state.composer.conditional(cname, pred, if_t, if_f) {
                Ok(mid) => Ok(ToolResult::text(json!({ "meta_id": mid, "status": "created" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_compose_validate" => {
            let mid = params["meta_id"].as_str().unwrap_or("");
            match state.composer.validate(mid) {
                Ok(warnings) => Ok(ToolResult::text(json!({
                    "meta_id": mid, "valid": warnings.is_empty(), "warnings": warnings
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_compose_run" => {
            let mid = params["meta_id"].as_str().unwrap_or("");
            match state.composer.get_meta(mid) {
                Ok(meta) => Ok(ToolResult::text(json!({
                    "meta_id": meta.id, "name": meta.name, "status": "execution_started"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        // --- Collective ---
        "workflow_collective_share" => {
            let cname = params["name"].as_str().unwrap_or("");
            let desc = params["description"].as_str().unwrap_or("");
            let wf_def = params["workflow_definition"].clone();
            let author = params["author"].as_str().unwrap_or("anonymous");
            let tags: Vec<String> = params["tags"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let id = state.collective.share(cname, desc, wf_def, author, tags);
            Ok(ToolResult::text(json!({ "id": id, "status": "shared" }).to_string()))
        }
        "workflow_collective_search" => {
            let query = params["query"].as_str().unwrap_or("");
            let results = state.collective.search(query);
            let items: Vec<_> = results.iter().map(|r| json!({
                "id": r.id, "name": r.name, "rating": r.rating, "downloads": r.download_count
            })).collect();
            Ok(ToolResult::text(json!({ "results": items }).to_string()))
        }
        "workflow_collective_apply" => {
            let id = params["id"].as_str().unwrap_or("");
            match state.collective.apply(id) {
                Some(wf) => Ok(ToolResult::text(json!({ "id": id, "workflow": wf, "status": "applied" }).to_string())),
                None => Ok(ToolResult::error(format!("Collective item not found: {}", id))),
            }
        }
        "workflow_collective_rate" => {
            let id = params["id"].as_str().unwrap_or("");
            let rating = params["rating"].as_f64().unwrap_or(3.0);
            if state.collective.rate(id, rating) {
                Ok(ToolResult::text(json!({ "id": id, "rating": rating, "status": "rated" }).to_string()))
            } else {
                Ok(ToolResult::error(format!("Collective item not found: {}", id)))
            }
        }
        "workflow_collective_private" => {
            let id = params["id"].as_str().unwrap_or("");
            let clean = state.collective.verify_privacy(id);
            Ok(ToolResult::text(json!({ "id": id, "privacy_verified": clean }).to_string()))
        }
        _ => Ok(ToolResult::error(format!("Unknown template tool: {}", name))),
    }
}
