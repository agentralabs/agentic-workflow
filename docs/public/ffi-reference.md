# FFI Reference

AgenticWorkflow provides a C-compatible FFI surface for embedding in non-Rust applications.

## Header

```c
#include "agentic_workflow.h"
```

## Opaque Types

```c
typedef struct AwfEngine AwfEngine;
typedef struct AwfWorkflow AwfWorkflow;
typedef struct AwfExecution AwfExecution;
typedef struct AwfResult AwfResult;
```

## Lifecycle Functions

```c
// Create a new engine instance
AwfEngine* awf_engine_new(void);

// Destroy an engine instance
void awf_engine_free(AwfEngine* engine);
```

## Workflow Functions

```c
// Parse a .awf file into a workflow
AwfWorkflow* awf_workflow_parse(const AwfEngine* engine, const char* path);

// Parse .awf content from a string
AwfWorkflow* awf_workflow_parse_str(const AwfEngine* engine, const char* content);

// Validate a workflow (returns 0 on success, error code otherwise)
int awf_workflow_validate(const AwfWorkflow* workflow);

// Free a workflow
void awf_workflow_free(AwfWorkflow* workflow);
```

## Execution Functions

```c
// Execute a workflow (blocking)
AwfExecution* awf_workflow_run(AwfEngine* engine, const AwfWorkflow* workflow);

// Get execution status (0=pending, 1=running, 2=completed, 3=failed, 4=cancelled)
int awf_execution_status(const AwfExecution* execution);

// Get execution result as JSON string (caller must free with awf_string_free)
char* awf_execution_result_json(const AwfExecution* execution);

// Cancel a running execution
int awf_execution_cancel(AwfExecution* execution);

// Free an execution
void awf_execution_free(AwfExecution* execution);
```

## Utility Functions

```c
// Free a string returned by the library
void awf_string_free(char* s);

// Get the last error message (NULL if no error)
const char* awf_last_error(void);

// Get library version string
const char* awf_version(void);
```

## Language Examples

### Python (ctypes)

```python
import ctypes

lib = ctypes.CDLL("libagentic_workflow_ffi.dylib")
engine = lib.awf_engine_new()
workflow = lib.awf_workflow_parse(engine, b"pipeline.awf")
execution = lib.awf_workflow_run(engine, workflow)
lib.awf_execution_free(execution)
lib.awf_workflow_free(workflow)
lib.awf_engine_free(engine)
```

### Swift

```swift
let engine = awf_engine_new()
let workflow = awf_workflow_parse(engine, "pipeline.awf")
let execution = awf_workflow_run(engine, workflow)
awf_execution_free(execution)
awf_workflow_free(workflow)
awf_engine_free(engine)
```
