use super::{
    Key, Location, ModifiersState, MouseButton, PhysicalPosition,
    axis::{Axis, MouseAxis},
    bindings::Bindings,
    button::Button,
};
use ahash::{AHashMap, AHashSet};
use std::{
    collections::VecDeque,
    hash::Hash,
    time::{Duration, Instant},
};
use winit::{
    event::{
        DeviceEvent, ElementState, KeyEvent, MouseScrollDelta, WindowEvent,
    },
    keyboard::{self, ModifiersKeyState, NamedKey},
};

#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub enum MouseButtonAction {
    #[default]
    None,
    Single(MouseButton),
    Double(MouseButton),
    Triple(MouseButton),
}

impl MouseButtonAction {
    pub fn contains(&self, button: MouseButton) -> bool {
        match self {
            MouseButtonAction::None => false,
            MouseButtonAction::Single(btn) => button == *btn,
            MouseButtonAction::Double(btn) => button == *btn,
            MouseButtonAction::Triple(btn) => button == *btn,
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            MouseButtonAction::None => false,
            MouseButtonAction::Single(_)
            | MouseButtonAction::Double(_)
            | MouseButtonAction::Triple(_) => true,
        }
    }

    pub fn get_button(&self) -> Option<MouseButton> {
        match self {
            MouseButtonAction::None => None,
            MouseButtonAction::Single(button) => Some(*button),
            MouseButtonAction::Double(button) => Some(*button),
            MouseButtonAction::Triple(button) => Some(*button),
        }
    }

    pub fn next(&mut self, button: MouseButton) {
        *self = match self {
            MouseButtonAction::None => MouseButtonAction::Single(button),
            MouseButtonAction::Single(_) => MouseButtonAction::Double(button),
            MouseButtonAction::Double(_) => MouseButtonAction::Triple(button),
            MouseButtonAction::Triple(_) => MouseButtonAction::None,
        };
    }

    pub fn set_single(&mut self, button: MouseButton) {
        *self = MouseButtonAction::Single(button);
    }

    pub fn clear(&mut self) {
        *self = MouseButtonAction::None;
    }
}

#[derive(Default, PartialEq, Copy, Clone)]
pub enum InputEvent {
    #[default]
    None,
    MouseButtonAction(MouseButtonAction),
    MouseButton {
        button: MouseButton,
        pressed: bool,
    },
    KeyInput {
        key: Key,
        location: Location,
        pressed: bool,
    },
    /// Returns Pysical Mouse Position.
    MousePosition {
        x: f64,
        y: f64,
    },
    MouseWheel {
        amount: f32,
        axis: MouseAxis,
    },
    WindowFocused(bool),
    Modifier {
        modifier: Modifier,
        pressed: bool,
    },
}

impl InputEvent {
    pub fn mouse_button_action(action: MouseButtonAction) -> Self {
        Self::MouseButtonAction(action)
    }

    pub fn mouse_button(button: MouseButton, pressed: bool) -> Self {
        Self::MouseButton { button, pressed }
    }

    pub fn key_input(key: Key, location: Location, pressed: bool) -> Self {
        Self::KeyInput {
            key,
            location,
            pressed,
        }
    }

    pub fn mouse_position(x: f64, y: f64) -> Self {
        Self::MousePosition { x, y }
    }

    pub fn mouse_wheel(amount: f32, axis: MouseAxis) -> Self {
        Self::MouseWheel { amount, axis }
    }

    pub fn window_focused(focused: bool) -> Self {
        Self::WindowFocused(focused)
    }

    pub fn modifier(modifier: Modifier, pressed: bool) -> Self {
        Self::Modifier { modifier, pressed }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Modifier {
    LShift,
    RShift,
    LAlt,
    RAlt,
    LControl,
    RControl,
    LSuper,
    RSuper,
}

/// Handler Contains all the Possible Key presses, Modifier Presses and Mouse locations.
pub struct InputHandler<ActionId, AxisId>
where
    ActionId: Clone + Eq + Hash + Send + Sync,
    AxisId: Clone + Eq + Hash + Send + Sync,
{
    /// The bindings.
    bindings: Bindings<ActionId, AxisId>,
    /// The set of keys that are currently pressed down by their virtual key code.
    pub keys: AHashMap<Key, Location>,
    /// The set of mouse buttons that are currently pressed down.
    pub mouse_buttons: AHashSet<winit::event::MouseButton>,
    /// The current pysical mouse position.
    pub mouse_position: Option<(f64, f64)>,
    /// The last recorded pysical mouse position.
    pub last_mouse_position: Option<(f64, f64)>,
    /// The mouse delta, i.e. the relative mouse motion.
    pub mouse_delta: (f64, f64),
    ///key modifiers can tell if left or right keys.
    pub modifiers: AHashSet<Modifier>,
    ///key modifiers state can not tell if left or right keys.
    pub modifiers_state: ModifiersState,
    ///Current Mouse button presses in action. Please refer to the input_events
    ///To get a finished MouseButtonAvtion Return.
    pub mouse_button_action: MouseButtonAction,
    mouse_action_timer: Instant,
    /// If the window is focused or not.
    pub window_focused: bool,
    ///Input events gathered per the last Click. Will contain multiple events.
    pub input_events: VecDeque<InputEvent>,
    ///Duration allowed between clicks.
    click_duration: Duration,
}

impl<ActionId, AxisId> InputHandler<ActionId, AxisId>
where
    ActionId: Clone + Eq + Hash + Send + Sync,
    AxisId: Clone + Eq + Hash + Send + Sync,
{
    pub fn axis_value<A>(&self, id: &A) -> f64
    where
        AxisId: std::borrow::Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        let axes = match self.bindings.axes.get(id) {
            Some(axes) => axes,
            _ => return 0.0,
        };

        axes.iter()
            .map(|axis| self.map_axis_value(axis))
            .max_by(|x, y| x.abs().partial_cmp(&y.abs()).unwrap())
            .unwrap_or(0.0)
    }

    /// Looks up the set of bindings for the action, and then checks if there is any binding for
    /// which all buttons are currently down.
    pub fn is_action_down<A>(&self, action: &A) -> bool
    where
        ActionId: std::borrow::Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        self.bindings
            .actions
            .get(action)
            .map(|bindings| {
                bindings.iter().any(|buttons| {
                    buttons
                        .iter()
                        .all(|button| self.is_button_down(*button, None))
                })
            })
            .unwrap_or(false)
    }

    ///Checks if a mouse button or key button is down.
    pub fn is_button_down(
        &self,
        button: Button,
        location: Option<Location>,
    ) -> bool {
        match button {
            Button::Key(key) => self.is_key_down(key, location),
            Button::Mouse(button) => self.is_mouse_button_down(button),
        }
    }

    ///Checks if a key is down.
    pub fn is_key_down(&self, key: Key, location: Option<Location>) -> bool {
        if let Some(k) = self.keys.get(&key) {
            if let Some(loc) = location {
                *k == loc
            } else {
                true
            }
        } else {
            false
        }
    }

    ///Checks if a modifier is down.
    pub fn is_modifier_down(&self, modifier: Modifier) -> bool {
        self.modifiers.contains(&modifier)
    }

    ///Checks if the window is focused.
    pub fn is_focused(&self) -> bool {
        self.window_focused
    }

    ///Checks if a mouse button is down.
    pub fn is_mouse_button_down(
        &self,
        button: winit::event::MouseButton,
    ) -> bool {
        self.mouse_buttons.contains(&button)
    }

    ///returns a specific Axis Value based on buttons and Axis.
    fn map_axis_value(&self, axis: &Axis) -> f64 {
        match axis {
            Axis::Emulated { pos, neg, .. } => {
                match (
                    self.is_button_down(*pos, None),
                    self.is_button_down(*neg, None),
                ) {
                    (true, false) => 1.0,
                    (false, true) => -1.0,
                    _ => 0.0,
                }
            }
            Axis::MouseMotion {
                axis,
                limit,
                radius,
            } => {
                let current_position =
                    self.mouse_position.unwrap_or((0.0, 0.0));
                let last_position =
                    self.last_mouse_position.unwrap_or(current_position);
                let delta = match axis {
                    MouseAxis::Horizontal => {
                        last_position.0 - current_position.0
                    }
                    MouseAxis::Vertical => last_position.1 - current_position.1,
                };

                let delta = delta / radius.into_inner() as f64;

                if *limit {
                    delta.clamp(-1.0f64, 1.0f64)
                } else {
                    delta
                }
            }
            Axis::RelativeMouseMotion {
                axis,
                limit,
                radius,
            } => {
                let delta = match axis {
                    MouseAxis::Horizontal => self.mouse_delta.0,
                    MouseAxis::Vertical => self.mouse_delta.1,
                };

                let delta = delta / radius.into_inner() as f64;

                if *limit {
                    delta.clamp(-1.0f64, 1.0f64)
                } else {
                    delta
                }
            }
        }
    }

    ///Get the modifier state.
    ///This can not tell between left and right buttons.
    ///Only use if you dont care which side is pressed.
    pub fn modifiers_state(&self) -> ModifiersState {
        self.modifiers_state
    }

    ///Get the current events that where last processed during update. Clears the internal event array.
    pub fn events(&mut self) -> VecDeque<InputEvent> {
        let events = self.input_events.clone();
        self.input_events.clear();
        events
    }

    ///Get the next pending event. Returns None when empty.
    pub fn pop_event(&mut self) -> Option<InputEvent> {
        self.input_events.pop_front()
    }

    ///Get Physical mouse position.
    /// This value is a f64 and is not calculated against the DPI.
    pub fn physical_mouse_position(&self) -> Option<(f64, f64)> {
        self.mouse_position
    }

    ///Initialize the Input Handler.
    /// bindings: Mapping of actions to take per certain requirements.
    /// click_duration: is the allowed duration
    /// between each click before the click is submitted.
    pub fn new(
        bindings: Bindings<ActionId, AxisId>,
        click_duration: Duration,
    ) -> Self {
        Self {
            bindings,
            keys: AHashMap::new(),
            mouse_buttons: AHashSet::new(),
            mouse_position: None,
            last_mouse_position: None,
            mouse_delta: (0.0, 0.0),
            modifiers: AHashSet::new(),
            modifiers_state: ModifiersState::default(),
            mouse_button_action: MouseButtonAction::None,
            mouse_action_timer: Instant::now(),
            window_focused: true,
            input_events: VecDeque::with_capacity(12),
            click_duration,
        }
    }

    pub fn set_click_duration(&mut self, click_duration: Duration) {
        self.click_duration = click_duration;
    }

    pub fn get_click_duration(&mut self) -> Duration {
        self.click_duration
    }

    ///Update the Input Handler based upon the windows events.
    pub fn window_updates(&mut self, event: &WindowEvent) {
        let mut button_action = None;

        //We clear and reset everything here.
        self.last_mouse_position = self.mouse_position;
        let timer = Instant::now();

        if self.mouse_action_timer <= timer
            && self.mouse_button_action.is_some()
        {
            self.input_events.push_back(InputEvent::mouse_button_action(
                self.mouse_button_action,
            ));
            button_action = self.mouse_button_action.get_button();
            self.mouse_button_action.clear();
        }

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        logical_key,
                        location,
                        text,
                        ..
                    },
                ..
            } => {
                let (key, key_to_add) = match logical_key {
                    keyboard::Key::Named(name) => {
                        if let Some(txt) = text
                            && matches!(
                                name,
                                NamedKey::Enter
                                    | NamedKey::Home
                                    | NamedKey::ArrowDown
                                    | NamedKey::ArrowUp
                                    | NamedKey::ArrowLeft
                                    | NamedKey::ArrowRight
                                    | NamedKey::End
                                    | NamedKey::PageUp
                                    | NamedKey::PageDown
                            )
                            && *location == Location::Numpad
                        {
                            let chars: Vec<char> = txt.chars().collect();

                            if let Some(c) = chars.first() {
                                (
                                    Key::Character(*c),
                                    Key::Character(
                                        c.to_lowercase().next().unwrap_or(*c),
                                    ),
                                )
                            } else {
                                return;
                            }
                        } else {
                            (Key::Named(*name), Key::Named(*name))
                        }
                    }
                    keyboard::Key::Character(str) => {
                        let chars: Vec<char> = str.chars().collect();

                        if let Some(c) = chars.first() {
                            (
                                Key::Character(*c),
                                Key::Character(
                                    c.to_lowercase().next().unwrap_or(*c),
                                ),
                            )
                        } else {
                            return;
                        }
                    }
                    _ => {
                        return;
                    }
                };

                if *state == ElementState::Pressed {
                    self.input_events
                        .push_back(InputEvent::key_input(key, *location, true));
                    self.keys.insert(key_to_add, *location);
                } else if self.keys.remove(&key_to_add).is_some() {
                    self.input_events.push_back(InputEvent::key_input(
                        key, *location, false,
                    ));
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *state == ElementState::Pressed {
                    self.mouse_buttons.insert(*button);
                    self.input_events
                        .push_back(InputEvent::mouse_button(*button, true));

                    if button_action != Some(*button) {
                        if !self.mouse_button_action.contains(*button) {
                            if self.mouse_button_action.is_some() {
                                self.input_events.push_back(
                                    InputEvent::mouse_button_action(
                                        self.mouse_button_action,
                                    ),
                                );
                            }

                            self.mouse_button_action.set_single(*button);
                        } else {
                            match self.mouse_button_action {
                                MouseButtonAction::None
                                | MouseButtonAction::Triple(_)
                                | MouseButtonAction::Single(_) => {
                                    self.mouse_button_action.next(*button)
                                }
                                MouseButtonAction::Double(_) => {
                                    self.mouse_button_action.next(*button);
                                    self.input_events.push_back(
                                        InputEvent::mouse_button_action(
                                            self.mouse_button_action,
                                        ),
                                    );
                                    self.mouse_button_action.clear();
                                }
                            }
                        }

                        self.mouse_action_timer = timer + self.click_duration;
                    }
                } else if self.mouse_buttons.remove(button) {
                    self.input_events
                        .push_back(InputEvent::mouse_button(*button, false));
                }
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                self.input_events
                    .push_back(InputEvent::mouse_position(*x, *y));
                self.mouse_position = Some((*x, *y));
            }
            WindowEvent::Focused(b) => {
                if !b {
                    self.keys.clear();
                    self.mouse_buttons.clear();
                }

                self.input_events.push_back(InputEvent::window_focused(*b));
                self.window_focused = *b;
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers_state = new_modifiers.state();

                for (state, modifier) in [
                    (new_modifiers.lshift_state(), Modifier::LShift),
                    (new_modifiers.rshift_state(), Modifier::RShift),
                    (new_modifiers.lalt_state(), Modifier::LAlt),
                    (new_modifiers.ralt_state(), Modifier::RAlt),
                    (new_modifiers.lcontrol_state(), Modifier::LControl),
                    (new_modifiers.rcontrol_state(), Modifier::RControl),
                    (new_modifiers.lsuper_state(), Modifier::LSuper),
                    (new_modifiers.rsuper_state(), Modifier::RSuper),
                ] {
                    if state == ModifiersKeyState::Pressed {
                        self.input_events
                            .push_back(InputEvent::modifier(modifier, true));
                        self.modifiers.insert(modifier);
                    } else if self.modifiers.remove(&modifier) {
                        self.input_events
                            .push_back(InputEvent::modifier(modifier, false));
                    }
                }
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => (*dx, *dy),
                    MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                        (*x as f32, *y as f32)
                    }
                };

                if x != 0.0 {
                    self.input_events.push_back(InputEvent::mouse_wheel(
                        x.signum(),
                        MouseAxis::Horizontal,
                    ));
                }

                if y != 0.0 {
                    self.input_events.push_back(InputEvent::mouse_wheel(
                        y.signum(),
                        MouseAxis::Vertical,
                    ));
                }
            }
            _ => (),
        }
    }

    pub fn device_updates(&mut self, event: &DeviceEvent) {
        //We clear and reset everything here.
        self.mouse_delta = (0.0, 0.0);

        if let DeviceEvent::MouseMotion { delta } = event {
            self.mouse_delta.0 -= delta.0;
            self.mouse_delta.1 -= delta.1;
        }
    }
}
