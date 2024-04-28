use super::{
    axis::{Axis, MouseAxis},
    bindings::Bindings,
    button::Button,
    Key, Location, ModifiersState, MouseButton, PhysicalPosition,
};
use ahash::{AHashMap, AHashSet};
use std::{
    hash::Hash,
    time::{Duration, Instant},
};
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyEvent, MouseScrollDelta,
        WindowEvent,
    },
    keyboard::{self, ModifiersKeyState},
    window::Window,
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
    pub fn get_button(&self) -> Option<MouseButton> {
        match self {
            MouseButtonAction::None => None,
            MouseButtonAction::Single(button) => Some(*button),
            MouseButtonAction::Double(button) => Some(*button),
            MouseButtonAction::Triple(button) => Some(*button),
        }
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
    MousePosition,
    MouseWheel,
    WindowFocused(bool),
    Modifier {
        modifier: Modifier,
        pressed: bool,
    },
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
    /// The current mouse position.
    pub physical_mouse_position: Option<PhysicalPosition<f64>>,
    /// The current mouse position.
    pub mouse_position: Option<(f32, f32)>,
    /// The last recorded mouse position.
    pub last_mouse_position: Option<(f32, f32)>,
    /// The mouse delta, i.e. the relative mouse motion.
    pub mouse_delta: (f64, f64),
    /// The current state of the mouse wheel.
    pub mouse_wheel: (f32, f32),
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
    pub input_events: Vec<InputEvent>,
    ///Duration allowed between clicks.
    click_duration: Duration,
}

impl<ActionId, AxisId> InputHandler<ActionId, AxisId>
where
    ActionId: Clone + Eq + Hash + Send + Sync,
    AxisId: Clone + Eq + Hash + Send + Sync,
{
    pub fn axis_value<A>(&self, id: &A) -> f32
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
    fn map_axis_value(&self, axis: &Axis) -> f32 {
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

                let delta = delta / radius.into_inner();

                if *limit {
                    delta.clamp(-1.0, 1.0)
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
                    MouseAxis::Horizontal => self.mouse_delta.0 as f32,
                    MouseAxis::Vertical => self.mouse_delta.1 as f32,
                };

                let delta = delta / radius.into_inner();

                if *limit {
                    delta.clamp(-1.0, 1.0)
                } else {
                    delta
                }
            }
            Axis::MouseWheel { axis } => self.mouse_wheel_value(*axis),
        }
    }

    ///Get the mouses current position.
    pub fn mouse_position(&self) -> Option<(f32, f32)> {
        self.mouse_position
    }

    ///Get the modifier state.
    ///This can not tell between left and right buttons.
    ///Only use if you dont care which side is pressed.
    pub fn modifiers_state(&self) -> ModifiersState {
        self.modifiers_state
    }

    ///Get the current events that where last processed during update.
    pub fn events(&self) -> &[InputEvent] {
        &self.input_events
    }

    ///Get Physical mouse position.
    /// This value is a f64 and is not calculated against the DPI.
    pub fn physical_mouse_position(&self) -> Option<PhysicalPosition<f64>> {
        self.physical_mouse_position
    }

    ///Get mouse wheel position based on MouseAxis.
    pub fn mouse_wheel_value(&self, axis: MouseAxis) -> f32 {
        match axis {
            MouseAxis::Horizontal => self.mouse_wheel.0,
            MouseAxis::Vertical => self.mouse_wheel.1,
        }
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
            physical_mouse_position: None,
            mouse_position: None,
            last_mouse_position: None,
            mouse_delta: (0.0, 0.0),
            mouse_wheel: (0.0, 0.0),
            modifiers: AHashSet::new(),
            modifiers_state: ModifiersState::default(),
            mouse_button_action: MouseButtonAction::None,
            mouse_action_timer: Instant::now(),
            window_focused: true,
            input_events: Vec::with_capacity(4),
            click_duration,
        }
    }

    ///Update the Input Handler based upon the windows events.
    pub fn update(&mut self, window: &Window, event: &Event<()>, hidpi: f32) {
        let mut button_action = None;

        //We clear and reset everything here.
        self.last_mouse_position = self.mouse_position;
        self.mouse_delta = (0.0, 0.0);
        self.mouse_wheel = (0.0, 0.0);
        self.input_events.clear();

        let timer = Instant::now();

        if self.mouse_action_timer <= timer
            && self.mouse_button_action != MouseButtonAction::None
        {
            self.input_events
                .push(InputEvent::MouseButtonAction(self.mouse_button_action));
            button_action = self.mouse_button_action.get_button();
            self.mouse_button_action = MouseButtonAction::None;
        }

        //we enforce it to loop more often to allow for better latency on input returns.
        if self.mouse_button_action != MouseButtonAction::None {
            window.request_redraw();
        }

        match *event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state,
                            logical_key,
                            location,
                            ..
                        },
                    ..
                } => {
                    let key = match logical_key {
                        keyboard::Key::Named(name) => Key::Named(*name),
                        keyboard::Key::Character(str) => {
                            let chars: Vec<char> = str.chars().collect();

                            if let Some(c) = chars.first() {
                                Key::Character(*c)
                            } else {
                                return;
                            }
                        }
                        _ => return,
                    };

                    if *state == ElementState::Pressed {
                        self.input_events.push(InputEvent::KeyInput {
                            key,
                            location: *location,
                            pressed: true,
                        });
                        self.keys.insert(key, *location);
                    } else {
                        if self.keys.contains_key(&key) {
                            self.input_events.push(InputEvent::KeyInput {
                                key,
                                location: *location,
                                pressed: false,
                            })
                        }
                        self.keys.remove(&key);
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if *state == ElementState::Pressed {
                        self.mouse_buttons.insert(*button);
                        self.input_events.push(InputEvent::MouseButton {
                            button: *button,
                            pressed: true,
                        });

                        if button_action != Some(*button) {
                            match self.mouse_button_action {
                                MouseButtonAction::None => {
                                    self.mouse_button_action =
                                        MouseButtonAction::Single(*button);
                                    self.mouse_action_timer =
                                        timer + self.click_duration;
                                }
                                MouseButtonAction::Single(btn) => {
                                    if btn == *button {
                                        self.mouse_button_action =
                                            MouseButtonAction::Double(btn);
                                        self.mouse_action_timer =
                                            timer + self.click_duration;
                                    } else {
                                        self.input_events.push(
                                            InputEvent::MouseButtonAction(
                                                self.mouse_button_action,
                                            ),
                                        );
                                        self.mouse_button_action =
                                            MouseButtonAction::Single(*button);
                                        self.mouse_action_timer =
                                            timer + self.click_duration;
                                    }
                                }
                                MouseButtonAction::Double(btn) => {
                                    if btn == *button {
                                        self.mouse_button_action =
                                            MouseButtonAction::None;
                                        self.input_events.push(
                                            InputEvent::MouseButtonAction(
                                                MouseButtonAction::Triple(btn),
                                            ),
                                        );
                                    } else {
                                        self.input_events.push(
                                            InputEvent::MouseButtonAction(
                                                self.mouse_button_action,
                                            ),
                                        );
                                        self.mouse_button_action =
                                            MouseButtonAction::Single(*button);
                                        self.mouse_action_timer =
                                            timer + self.click_duration;
                                    }
                                }
                                MouseButtonAction::Triple(_) => {}
                            }
                        }
                    } else {
                        if self.mouse_buttons.contains(button) {
                            self.input_events.push(InputEvent::MouseButton {
                                button: *button,
                                pressed: false,
                            });
                        }
                        self.mouse_buttons.remove(button);
                    }
                }
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => {
                    self.input_events.push(InputEvent::MousePosition);
                    self.physical_mouse_position =
                        Some(PhysicalPosition { x: *x, y: *y });
                    self.mouse_position =
                        Some(((*x as f32) * hidpi, (*y as f32) * hidpi));
                }
                WindowEvent::Focused(b) => {
                    if !b {
                        self.keys.clear();
                        self.mouse_buttons.clear();
                    }

                    self.input_events.push(InputEvent::WindowFocused(*b));
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
                            self.input_events.push(InputEvent::Modifier {
                                modifier,
                                pressed: true,
                            });
                            self.modifiers.insert(modifier);
                        } else {
                            if self.modifiers.contains(&modifier) {
                                self.input_events.push(InputEvent::Modifier {
                                    modifier,
                                    pressed: false,
                                });
                            }

                            self.modifiers.remove(&modifier);
                        }
                    }
                }
                _ => (),
            },
            Event::DeviceEvent { ref event, .. } => match *event {
                DeviceEvent::MouseMotion { delta } => {
                    self.mouse_delta.0 -= delta.0;
                    self.mouse_delta.1 -= delta.1;
                }
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(dx, dy),
                } => {
                    if dx != 0.0 {
                        self.mouse_wheel.0 = dx.signum();
                    }

                    if dy != 0.0 {
                        self.mouse_wheel.1 = dy.signum();
                    }

                    self.input_events.push(InputEvent::MouseWheel);
                }
                DeviceEvent::MouseWheel {
                    delta:
                        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }),
                } => {
                    if x != 0.0 {
                        self.mouse_wheel.0 = x.signum() as f32;
                    }

                    if y != 0.0 {
                        self.mouse_wheel.1 = y.signum() as f32;
                    }

                    self.input_events.push(InputEvent::MouseWheel);
                }
                _ => (),
            },
            _ => (),
        }
    }
}
