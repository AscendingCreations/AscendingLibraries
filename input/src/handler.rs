use super::axis::{Axis, MouseAxis};
use super::bindings::Bindings;
use super::button::Button;
use super::{Key, Location};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use winit::dpi::PhysicalPosition;
use winit::event::KeyEvent;
use winit::window::Window;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, WindowEvent},
    keyboard::{self, ModifiersState},
};

pub struct InputHandler<ActionId, AxisId>
where
    ActionId: Clone + Eq + Hash + Send + Sync,
    AxisId: Clone + Eq + Hash + Send + Sync,
{
    /// The bindings.
    bindings: Bindings<ActionId, AxisId>,
    /// The set of keys that are currently pressed down by their virtual key code.
    keys: HashMap<Key, Location>,
    /// The set of mouse buttons that are currently pressed down.
    mouse_buttons: HashSet<winit::event::MouseButton>,
    /// The current mouse position.
    physical_mouse_position: Option<PhysicalPosition<f64>>,
    /// The current mouse position.
    mouse_position: Option<(f32, f32)>,
    /// The last recorded mouse position.
    last_mouse_position: Option<(f32, f32)>,
    /// The mouse delta, i.e. the relative mouse motion.
    mouse_delta: (f64, f64),
    /// The current state of the mouse wheel.
    mouse_wheel: (f32, f32),
    //key modifiers.
    modifiers: ModifiersState,
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

    pub fn end_frame(&mut self) {
        self.last_mouse_position = self.mouse_position;
        self.mouse_delta = (0.0, 0.0);
        self.mouse_wheel = (0.0, 0.0);
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
                    buttons.iter().all(|button| self.is_button_down(*button))
                })
            })
            .unwrap_or(false)
    }

    pub fn is_button_down(&self, button: Button) -> bool {
        match button {
            Button::Key(key) => self.is_key_down(key),
            Button::Mouse(button) => self.is_mouse_button_down(button),
        }
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.keys.contains_key(&key)
    }

    pub fn is_mouse_button_down(
        &self,
        button: winit::event::MouseButton,
    ) -> bool {
        self.mouse_buttons.contains(&button)
    }

    fn map_axis_value(&self, axis: &Axis) -> f32 {
        match axis {
            Axis::Emulated { pos, neg, .. } => {
                match (self.is_button_down(*pos), self.is_button_down(*neg)) {
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

    pub fn mouse_position(&self) -> Option<(f32, f32)> {
        self.mouse_position
    }

    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }

    pub fn physical_mouse_position(&self) -> Option<PhysicalPosition<f64>> {
        self.physical_mouse_position
    }

    pub fn mouse_wheel_value(&self, axis: MouseAxis) -> f32 {
        match axis {
            MouseAxis::Horizontal => self.mouse_wheel.0,
            MouseAxis::Vertical => self.mouse_wheel.1,
        }
    }

    pub fn new(bindings: Bindings<ActionId, AxisId>) -> Self {
        Self {
            bindings,
            keys: HashMap::new(),
            mouse_buttons: HashSet::new(),
            physical_mouse_position: None,
            mouse_position: None,
            last_mouse_position: None,
            mouse_delta: (0.0, 0.0),
            mouse_wheel: (0.0, 0.0),
            modifiers: ModifiersState::default(),
        }
    }

    pub fn update(&mut self, window: &Window, event: &Event<()>, hidpi: f32) {
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
                        self.keys.insert(key, *location);
                    } else {
                        self.keys.remove(&key);
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if *state == ElementState::Pressed {
                        self.mouse_buttons.insert(*button);
                    } else {
                        self.mouse_buttons.remove(button);
                    }
                }
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => {
                    self.physical_mouse_position =
                        Some(PhysicalPosition { x: *x, y: *y });
                    self.mouse_position =
                        Some(((*x as f32) * hidpi, (*y as f32) * hidpi));
                }
                WindowEvent::Focused(false) => {
                    self.keys.clear();
                    self.mouse_buttons.clear();
                }
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    self.modifiers = new_modifiers.state();
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
                }
                _ => (),
            },
            _ => (),
        }
    }
}
