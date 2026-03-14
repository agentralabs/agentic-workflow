//! AgenticWorkflow FFI bindings
//!
//! Provides C-compatible foreign function interface for workflow operations.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use agentic_workflow::{Workflow, StepNode, StepType, Edge, EdgeType};
use agentic_workflow::engine::DagEngine;

static mut ENGINE: Option<DagEngine> = None;

fn get_engine() -> &'static mut DagEngine {
    unsafe {
        if ENGINE.is_none() {
            ENGINE = Some(DagEngine::new());
        }
        ENGINE.as_mut().unwrap()
    }
}

/// Initialize the workflow engine. Call once at startup.
#[no_mangle]
pub extern "C" fn awf_init() -> i32 {
    get_engine();
    0
}

/// Create a new workflow. Returns the workflow ID as a C string (caller must free).
#[no_mangle]
pub extern "C" fn awf_workflow_create(
    name: *const c_char,
    description: *const c_char,
) -> *mut c_char {
    let name = unsafe {
        if name.is_null() { return ptr::null_mut(); }
        CStr::from_ptr(name).to_string_lossy().to_string()
    };

    let description = unsafe {
        if description.is_null() { "".to_string() }
        else { CStr::from_ptr(description).to_string_lossy().to_string() }
    };

    let wf = Workflow::new(&name, &description);
    let id = wf.id.clone();

    match get_engine().register_workflow(wf) {
        Ok(()) => CString::new(id).unwrap().into_raw(),
        Err(e) => {
            eprintln!("awf_workflow_create error: {}", e);
            ptr::null_mut()
        }
    }
}

/// Validate a workflow DAG. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn awf_workflow_validate(workflow_id: *const c_char) -> i32 {
    let wf_id = unsafe {
        if workflow_id.is_null() { return -1; }
        CStr::from_ptr(workflow_id).to_string_lossy().to_string()
    };

    match get_engine().get_workflow(&wf_id) {
        Ok(wf) => match get_engine().validate_dag(wf) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("Validation failed: {}", e);
                -1
            }
        },
        Err(e) => {
            eprintln!("Workflow not found: {}", e);
            -1
        }
    }
}

/// Start executing a workflow. Returns execution ID (caller must free).
#[no_mangle]
pub extern "C" fn awf_workflow_run(workflow_id: *const c_char) -> *mut c_char {
    let wf_id = unsafe {
        if workflow_id.is_null() { return ptr::null_mut(); }
        CStr::from_ptr(workflow_id).to_string_lossy().to_string()
    };

    match get_engine().start_execution(&wf_id) {
        Ok(exec_id) => CString::new(exec_id).unwrap().into_raw(),
        Err(e) => {
            eprintln!("awf_workflow_run error: {}", e);
            ptr::null_mut()
        }
    }
}

/// Pause a running execution. Returns 0 on success.
#[no_mangle]
pub extern "C" fn awf_execution_pause(execution_id: *const c_char) -> i32 {
    let exec_id = unsafe {
        if execution_id.is_null() { return -1; }
        CStr::from_ptr(execution_id).to_string_lossy().to_string()
    };

    match get_engine().pause_execution(&exec_id) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Pause error: {}", e);
            -1
        }
    }
}

/// Resume a paused execution. Returns 0 on success.
#[no_mangle]
pub extern "C" fn awf_execution_resume(execution_id: *const c_char) -> i32 {
    let exec_id = unsafe {
        if execution_id.is_null() { return -1; }
        CStr::from_ptr(execution_id).to_string_lossy().to_string()
    };

    match get_engine().resume_execution(&exec_id) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Resume error: {}", e);
            -1
        }
    }
}

/// Cancel a running execution. Returns 0 on success.
#[no_mangle]
pub extern "C" fn awf_execution_cancel(execution_id: *const c_char) -> i32 {
    let exec_id = unsafe {
        if execution_id.is_null() { return -1; }
        CStr::from_ptr(execution_id).to_string_lossy().to_string()
    };

    match get_engine().cancel_execution(&exec_id) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Cancel error: {}", e);
            -1
        }
    }
}

/// Free a C string returned by this library.
#[no_mangle]
pub extern "C" fn awf_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

/// Get version string. Returns a static string (do NOT free).
#[no_mangle]
pub extern "C" fn awf_version() -> *const c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
}
