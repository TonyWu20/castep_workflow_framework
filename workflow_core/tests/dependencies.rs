// Integration tests for dependency handling
// These tests verify the DAG execution model works correctly.

use std::collections::HashMap;
use tempfile::tempdir;
use workflow_core::{ExecutionMode, Task, Workflow};

#[test]
fn test_diamond_ancestry() {
    // Verify that a DAG with diamond ancestry (a->b, a->c, b->d, c->d)
    // executes in correct topological order.
    let _dir = tempdir().unwrap();

    // Create workflow with diamond: a -> b, c; b, c -> d
    let mut wf = Workflow::new("diamond_test");

    wf.add_task(Task::new(
        "a",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    ))
    .unwrap();

    wf.add_task(
        Task::new(
            "b",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .depends_on("a"),
    )
    .unwrap();

    wf.add_task(
        Task::new(
            "c",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .depends_on("a"),
    )
    .unwrap();

    wf.add_task(
        Task::new(
            "d",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .depends_on("b")
        .depends_on("c"),
    )
    .unwrap();

    // Verify dry_run returns valid topological order
    let order = wf.dry_run().unwrap();
    assert_eq!(order.len(), 4, "expected 4 tasks in topological order");
    let pa = order.iter().position(|x| x == "a").unwrap();
    let pb = order.iter().position(|x| x == "b").unwrap();
    let pc = order.iter().position(|x| x == "c").unwrap();
    let pd = order.iter().position(|x| x == "d").unwrap();
    assert!(pa < pb, "a must precede b");
    assert!(pa < pc, "a must precede c");
    assert!(pb < pd, "b must precede d");
    assert!(pc < pd, "c must precede d");
}
