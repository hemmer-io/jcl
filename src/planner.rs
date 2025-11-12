//! Planner - creates execution plans and manages dependencies

use anyhow::Result;
use petgraph::graph::DiGraph;
use std::fmt;

/// A plan of changes to be applied
pub struct Plan {
    pub actions: Vec<Action>,
    pub dependency_graph: DiGraph<String, ()>,
}

/// An action to be performed
#[derive(Debug, Clone)]
pub enum Action {
    Create {
        resource_type: String,
        name: String,
    },
    Update {
        resource_type: String,
        name: String,
        changes: Vec<Change>,
    },
    Delete {
        resource_type: String,
        name: String,
    },
    NoOp {
        resource_type: String,
        name: String,
    },
}

/// A change to a resource attribute
#[derive(Debug, Clone)]
pub struct Change {
    pub attribute: String,
    pub old_value: String,
    pub new_value: String,
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Planned actions:")?;
        writeln!(f)?;

        for action in &self.actions {
            match action {
                Action::Create { resource_type, name } => {
                    writeln!(f, "  + {} {}", resource_type, name)?;
                }
                Action::Update { resource_type, name, changes } => {
                    writeln!(f, "  ~ {} {}", resource_type, name)?;
                    for change in changes {
                        writeln!(
                            f,
                            "      {} {} -> {}",
                            change.attribute, change.old_value, change.new_value
                        )?;
                    }
                }
                Action::Delete { resource_type, name } => {
                    writeln!(f, "  - {} {}", resource_type, name)?;
                }
                Action::NoOp { resource_type, name } => {
                    writeln!(f, "    {} {} (no changes)", resource_type, name)?;
                }
            }
        }

        Ok(())
    }
}

/// Planner creates execution plans
pub struct Planner {
    // TODO: Add state backend
}

impl Planner {
    /// Create a new planner
    pub fn new() -> Self {
        Self {}
    }

    /// Create a plan for a stack
    pub fn plan(&self, _stack_name: &str) -> Result<Plan> {
        // TODO: Implement planning
        // 1. Load current state
        // 2. Compare with desired state
        // 3. Build dependency graph
        // 4. Determine actions
        // 5. Order actions based on dependencies

        Ok(Plan {
            actions: vec![],
            dependency_graph: DiGraph::new(),
        })
    }

    /// Apply a plan
    pub fn apply(&self, _plan: Plan) -> Result<()> {
        // TODO: Implement apply
        // 1. Validate plan
        // 2. Acquire state lock
        // 3. Execute actions in order
        // 4. Update state
        // 5. Release lock
        Ok(())
    }

    /// Destroy a stack
    pub fn destroy(&self, _stack_name: &str) -> Result<()> {
        // TODO: Implement destroy
        Ok(())
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}
