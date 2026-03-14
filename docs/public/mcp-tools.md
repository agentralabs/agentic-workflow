# MCP Tools Reference

AgenticWorkflow exposes 124 MCP tools across 24 capability categories.

## Workflow Lifecycle (12 tools)

| Tool | Description |
|------|-------------|
| `workflow_create` | Create a new workflow definition |
| `workflow_run` | Execute a workflow |
| `workflow_pause` | Pause a running workflow |
| `workflow_resume` | Resume a paused workflow |
| `workflow_cancel` | Cancel a running workflow |
| `workflow_status` | Get workflow execution status |
| `workflow_list` | List all workflows |
| `workflow_delete` | Delete a workflow definition |
| `workflow_clone` | Clone an existing workflow |
| `workflow_export` | Export workflow to .awf format |
| `workflow_import` | Import workflow from .awf file |
| `workflow_validate` | Validate workflow correctness |

## Step Management (10 tools)

| Tool | Description |
|------|-------------|
| `step_add` | Add a step to a workflow |
| `step_remove` | Remove a step from a workflow |
| `step_update` | Update step configuration |
| `step_reorder` | Change step execution order |
| `step_enable` | Enable a disabled step |
| `step_disable` | Disable a step without removing |
| `step_status` | Get step execution status |
| `step_output` | Retrieve step output data |
| `step_retry` | Retry a failed step |
| `step_skip` | Skip a pending step |

## Dependency Management (6 tools)

| Tool | Description |
|------|-------------|
| `dep_add` | Add dependency between steps |
| `dep_remove` | Remove a dependency |
| `dep_list` | List dependencies for a step |
| `dep_validate` | Validate dependency graph |
| `dep_cycle_check` | Check for circular dependencies |
| `dep_topological_sort` | Get execution order |

## Template Operations (8 tools)

| Tool | Description |
|------|-------------|
| `template_list` | List available templates |
| `template_create` | Create a template from workflow |
| `template_apply` | Apply template to create workflow |
| `template_export` | Export template to file |
| `template_import` | Import template from file |
| `template_validate` | Validate template correctness |
| `template_compose` | Compose multiple templates |
| `template_params` | List template parameters |

## Execution Control (10 tools)

| Tool | Description |
|------|-------------|
| `exec_start` | Start workflow execution |
| `exec_stop` | Stop workflow execution |
| `exec_checkpoint` | Create execution checkpoint |
| `exec_restore` | Restore from checkpoint |
| `exec_dry_run` | Validate without executing |
| `exec_parallel_limit` | Set parallel execution limit |
| `exec_timeout_set` | Set step or global timeout |
| `exec_retry_config` | Configure retry behavior |
| `exec_resource_bind` | Bind resource to execution |
| `exec_metrics` | Get execution metrics |

## AWF File Operations (8 tools)

| Tool | Description |
|------|-------------|
| `awf_parse` | Parse .awf file content |
| `awf_serialize` | Serialize workflow to .awf |
| `awf_validate` | Validate .awf syntax |
| `awf_migrate` | Migrate to latest schema |
| `awf_diff` | Diff two .awf files |
| `awf_merge` | Merge workflow definitions |
| `awf_schema` | Get .awf JSON schema |
| `awf_format` | Format .awf file |

## Variable Management (8 tools)

| Tool | Description |
|------|-------------|
| `var_set` | Set a workflow variable |
| `var_get` | Get a variable value |
| `var_list` | List all variables |
| `var_delete` | Delete a variable |
| `var_scope_push` | Push variable scope |
| `var_scope_pop` | Pop variable scope |
| `var_interpolate` | Interpolate variables in text |
| `var_validate` | Validate variable references |

## Conditional Routing (8 tools)

| Tool | Description |
|------|-------------|
| `cond_branch` | Add conditional branch |
| `cond_merge` | Add merge point |
| `cond_evaluate` | Evaluate condition |
| `cond_list` | List conditions in workflow |
| `cond_update` | Update condition expression |
| `cond_remove` | Remove condition |
| `cond_default` | Set default branch |
| `cond_validate` | Validate condition expressions |

## Hook System (8 tools)

| Tool | Description |
|------|-------------|
| `hook_pre_step` | Register pre-step hook |
| `hook_post_step` | Register post-step hook |
| `hook_on_error` | Register error hook |
| `hook_on_complete` | Register completion hook |
| `hook_list` | List registered hooks |
| `hook_remove` | Remove a hook |
| `hook_enable` | Enable a hook |
| `hook_disable` | Disable a hook |

## Error Handling (6 tools)

| Tool | Description |
|------|-------------|
| `error_classify` | Classify an error type |
| `error_history` | Get error history |
| `error_retry_policy` | Set error retry policy |
| `error_escalate` | Escalate error to parent |
| `error_suppress` | Suppress error class |
| `error_stats` | Get error statistics |

## Monitoring and Audit (10 tools)

| Tool | Description |
|------|-------------|
| `monitor_events` | Stream workflow events |
| `monitor_metrics` | Get execution metrics |
| `monitor_audit_log` | Get audit trail |
| `monitor_errors` | Get error summary |
| `monitor_throughput` | Get throughput stats |
| `monitor_duration` | Get duration breakdown |
| `monitor_resource_usage` | Get resource usage |
| `monitor_step_timeline` | Get step timeline |
| `monitor_health` | Get engine health |
| `monitor_export` | Export monitoring data |

## Composition (6 tools)

| Tool | Description |
|------|-------------|
| `compose_nest` | Nest workflow as sub-step |
| `compose_chain` | Chain workflows sequentially |
| `compose_parallel` | Run workflows in parallel |
| `compose_merge` | Merge workflow outputs |
| `compose_validate` | Validate composition |
| `compose_flatten` | Flatten nested workflows |

## Resource Pool (6 tools)

| Tool | Description |
|------|-------------|
| `resource_create` | Create a shared resource |
| `resource_acquire` | Acquire resource lock |
| `resource_release` | Release resource lock |
| `resource_list` | List resources |
| `resource_status` | Get resource status |
| `resource_delete` | Delete a resource |

## Schema Validation (6 tools)

| Tool | Description |
|------|-------------|
| `schema_validate_step` | Validate step params |
| `schema_validate_workflow` | Validate full workflow |
| `schema_get` | Get schema definition |
| `schema_list` | List known schemas |
| `schema_register` | Register custom schema |
| `schema_migrate` | Migrate schema version |

## Migration (6 tools)

| Tool | Description |
|------|-------------|
| `migrate_check` | Check if migration needed |
| `migrate_preview` | Preview migration changes |
| `migrate_apply` | Apply migration |
| `migrate_rollback` | Rollback migration |
| `migrate_history` | Get migration history |
| `migrate_validate` | Validate post-migration |

## Dry Run (6 tools)

| Tool | Description |
|------|-------------|
| `dry_run_full` | Full workflow dry run |
| `dry_run_step` | Single step dry run |
| `dry_run_deps` | Validate dependencies only |
| `dry_run_resources` | Check resource availability |
| `dry_run_report` | Generate dry run report |
| `dry_run_compare` | Compare dry run vs actual |
