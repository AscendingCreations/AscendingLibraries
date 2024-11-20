use crate::Vec2;
use cosmic_text::Command;

/// Path command.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MeshCommand {
    /// Begins a new subpath at the specified point.
    MoveTo(Vec2),
    /// A straight line from the previous point to the specified point.
    LineTo(Vec2),
    /// A cubic bezier curve from the previous point to the final point with
    /// two intermediate control points.
    CurveTo(Vec2, Vec2, Vec2),
    /// A quadratic curve from the previous point to the final point with one
    /// intermediate control point.
    QuadTo(Vec2, Vec2),
    /// Closes a subpath, connecting the final point to the initial point.
    Close,
}

impl From<&Command> for MeshCommand {
    fn from(value: &Command) -> Self {
        (*value).into()
    }
}

impl From<Command> for MeshCommand {
    fn from(value: Command) -> Self {
        match value {
            Command::MoveTo(vector) => {
                MeshCommand::MoveTo(Vec2::new(vector.x, vector.y))
            }
            Command::LineTo(vector) => {
                MeshCommand::LineTo(Vec2::new(vector.x, vector.y))
            }
            Command::CurveTo(vector, vector1, vector2) => MeshCommand::CurveTo(
                Vec2::new(vector.x, vector.y),
                Vec2::new(vector1.x, vector1.y),
                Vec2::new(vector2.x, vector2.y),
            ),

            Command::QuadTo(vector, vector1) => MeshCommand::QuadTo(
                Vec2::new(vector.x, vector.y),
                Vec2::new(vector1.x, vector1.y),
            ),
            Command::Close => MeshCommand::Close,
        }
    }
}

/// Glyph Layout in Mesh Command positions.
pub struct GlyphMesh {
    commands: Vec<MeshCommand>,
    offset: Vec2,
}

impl GlyphMesh {
    pub fn new(offset: Vec2, commands: &[Command]) -> Self {
        let mut mesh_commands = Vec::with_capacity(commands.len());

        for command in commands {
            mesh_commands.push(command.into());
        }

        Self {
            commands: mesh_commands,
            offset,
        }
    }
}
