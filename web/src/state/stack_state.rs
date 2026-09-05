use leptos::*;
use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct StackItem {
    pub name: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Stack {
    pub id: String,
    pub name: String,
    pub items: Vec<StackItem>,
}

#[derive(Clone)]
pub struct StackState {
    pub stacks: RwSignal<Vec<Stack>>,
}

impl Default for StackState {
    fn default() -> Self {
        Self {
            stacks: RwSignal::new(Vec::new()),
        }
    }
}

impl StackState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_stack(&mut self, name: &str, items: Vec<StackItem>) {
        let stack = Stack {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            items,
        };
        let mut stacks = self.stacks.get_untracked();
        stacks.insert(0, stack);
        self.stacks.set(stacks);
    }

    pub fn delete_stack(&mut self, id: &str) {
        let mut stacks = self.stacks.get_untracked();
        stacks.retain(|s| s.id != id);
        self.stacks.set(stacks);
    }

    pub fn log_stack(&self, id: &str) -> Option<Vec<String>> {
        let stacks = self.stacks.read();
        stacks.iter()
            .find(|s| s.id == id)
            .map(|stack| {
                stack.items.iter()
                    .map(|item| format!("Logged: {} {}", item.name, item.unit.as_deref().unwrap_or("")))
                    .collect()
            })
    }
}
