use super::axis::Axis;
use super::button::Button;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// Represents a collection of bindings mapping inputs to actions and axes for various input
/// devices.
#[derive(Default, Serialize, Deserialize)]
pub struct Bindings<ActionId, AxisId>
where
    ActionId: Clone + Eq + Hash + Send + Sync,
    AxisId: Clone + Eq + Hash + Send + Sync,
{
    /// A mapping from the action ID to an array of button combinations.
    pub(super) actions: HashMap<ActionId, Vec<Vec<Button>>>,
    /// A mapping from the axis ID to an array of axes.
    pub(super) axes: HashMap<AxisId, Vec<Axis>>,
}

impl<'de, ActionId, AxisId> Bindings<ActionId, AxisId>
where
    ActionId: Clone + Eq + Hash + Serialize + Deserialize<'de> + Send + Sync,
    AxisId: Clone + Eq + Hash + Serialize + Deserialize<'de> + Send + Sync,
{
    pub fn insert_action<B: IntoIterator<Item = Button>>(
        &mut self,
        id: ActionId,
        buttons: B,
    ) {
        // Collect the button combination.
        let action = buttons.into_iter().collect::<Vec<Button>>();

        // Add the button combination to the bindings for the given action ID.
        if let Some(actions) = self.actions.get_mut(&id) {
            actions.push(action);
            return;
        }

        // Create the bindings for the action ID.
        let bindings = vec![action];
        self.actions.insert(id, bindings);
    }

    pub fn insert_axis(&mut self, id: AxisId, axis: Axis) {
        // Add the axis to the bindings for the given axis ID.
        if let Some(bindings) = self.axes.get_mut(&id) {
            bindings.push(axis);
            return;
        }

        // Create the bindings for the axis ID.
        let bindings = vec![axis];
        self.axes.insert(id, bindings);
    }

    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            axes: HashMap::new(),
        }
    }
}
