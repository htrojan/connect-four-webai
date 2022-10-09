use std::cmp::min;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::sprite::Material2d;
use bevy::window::{close_on_esc, WindowResized};

const COLS: u32 = 7;
const ROWS: u32 = 6;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(highlight_system)
        .add_system(stone_spawn_system)
        .add_system(stone_moving_system)
        .add_system(resize_handler)
        .add_system(close_on_esc)
        .run();
}

#[derive(Component)]
struct Board;

#[derive(Component)]
struct HighlightColumn {
    number: u32,
}

enum Player {
    Player1,
    Player2
}

struct BoardAssets {
    highlight_size: Vec2,
    left_coord: f32,
    mesh_handle: Handle<Mesh>,
    // board: [[Player; COLS as usize]; ROWS as usize]
}

impl BoardAssets {
    fn stone_position(&self, game_position: UVec2) -> Vec2 {
        let down_coord = -self.highlight_size.y / 2.;

        let x_pos = self.left_coord + game_position.x as f32 * self.highlight_size.x;
        let y_pos = down_coord + game_position.y as f32 * (self.highlight_size.y / ROWS as f32);
        Vec2::new(x_pos, y_pos)
    }
}

struct BoardShape {
    size: Vec2,
    rows: u32,
    cols: u32,
}

#[derive(Component)]
struct Stone {
    /// If stone is still moving onto its goal
    moving: bool,
    velocity: f32,
    /// The final board position the stone will land on in the game
    board_position: UVec2,
}

/// Stores information how the player stones should be rendered
struct StoneAssets {
    /// Materials for player one and two
    mat_handle: [Handle<ColorMaterial>; 2],
    mesh_handle: Handle<Mesh>,
}

impl From<BoardShape> for Mesh {
    fn from(board: BoardShape) -> Self {
        let x_space = board.size.x / board.cols as f32;
        let y_space = board.size.y / board.rows as f32;

        let upper_y = board.size.y / 2.;
        let lower_y = -board.size.y / 2.;

        let left_x = -board.size.x / 2.;
        let right_x = board.size.x / 2.;

        let mut lines = Vec::<Vec3>::new();
        for col in 1..board.cols {
            lines.push(Vec3::new(left_x + col as f32 * x_space, lower_y, 0.));
            lines.push(Vec3::new(left_x + col as f32 * x_space, upper_y, 0.));
        }
        for row in 1..board.rows {
            lines.push(Vec3::new(left_x, lower_y + row as f32 * y_space, 0.));
            lines.push(Vec3::new(right_x, lower_y + row as f32 * y_space, 0.));
        }


        let max = (board.cols * 2) + (board.rows * 2);
        let indices = Indices::U32((0..(max)).collect());

        let positions: Vec<_> = lines.iter().map(|p: &Vec3| [p.x, p.y, p.z]).collect();
        let normals: Vec<_> = lines.iter().map(|p| [0., 0., 1.]).collect();
        let uvs: Vec<_> = lines.iter().map(|p| [0., 1.]).collect();

        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

fn setup_board(mut commands: Commands, mut windows: ResMut<Windows>, mut meshes: ResMut<Assets<Mesh>>) {
    shape::Quad::default();
}

fn setup(mut commands: Commands, mut windows: ResMut<Windows>, mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<ColorMaterial>>) {
    let window = windows.get_primary_mut().unwrap();

    let width = window.width();
    let height = window.height();

    let highlight_size = Vec2::new(width / COLS as f32, height);
    let left_coord = -width / 2. + highlight_size.x / 2.;

    let diameter = ((width / COLS as f32).min(height / ROWS as f32));
    let stone_mesh = meshes.add(Mesh::from(shape::Circle::new(diameter / 2.)));
    let stone_mat_1 = materials.add(ColorMaterial {
        color: Color::rgba(0.8, 0.1, 0., 1.),
        texture: None,
    });
    let stone_mat_2 = materials.add(ColorMaterial {
        color: Color::rgba(1., 1., 0., 1.),
        texture: None,
    });

    let stone_meta = StoneAssets {
        mat_handle: [stone_mat_1, stone_mat_2],
        mesh_handle: stone_mesh,
    };
    commands.insert_resource(stone_meta);

    println!("Creating board mesh");
    let board_mesh = meshes.add(
        Mesh::from(BoardShape {
            size: Vec2::new(1., 1.),
            rows: ROWS,
            cols: COLS,
        })
    );
    let board_material = materials.add(
        ColorMaterial { color: Color::rgba(0., 0., 0., 1.), texture: None }
    );

    commands.spawn_bundle(
        ColorMesh2dBundle {
            mesh: board_mesh.clone().into(),
            material: board_material,
            transform: Transform::default().with_scale(Vec3::new(width, height, 1.)),
            ..default()
            // transform: Default::default(),
            // global_transform: Default::default(),
        }
    ).insert(Board);

    commands.insert_resource(
        BoardAssets {
            highlight_size,
            left_coord,
            mesh_handle: board_mesh,
        }
    );

    let highlight_mesh = meshes.add(
        Mesh::from(shape::Quad::new(
            Vec2::new(
                1.,
                1.,
            )
        ))
    );

    let highlight_mat_1 = materials.add(
        ColorMaterial { color: Color::rgba(0.7, 0.7, 0.7, 0.2), texture: None }
    );

    commands.spawn_bundle(ColorMesh2dBundle {
        mesh: highlight_mesh.clone().into(),
        material: highlight_mat_1.clone(),
        transform: Transform::from_xyz(left_coord + highlight_size.x, 0., 10.).with_scale(highlight_size.extend(0.)),
        visibility: Visibility { is_visible: true },
        ..default()
    }).insert(HighlightColumn { number: 0 });

    commands.spawn_bundle(Camera2dBundle {
        camera: Camera {
            ..default()
        },
        ..default()
    });
}

fn resize_handler(mut ev_reader: EventReader<WindowResized>, mut query: Query<&mut Transform, With<Board>>,
                  mut meta: ResMut<BoardAssets>) {
    for ev in ev_reader.iter() {
        let mut t = query.single_mut();
        t.scale = Vec3::new(ev.width, ev.height, 1.);
        meta.highlight_size = Vec2::new(ev.width / COLS as f32, ev.height);
        meta.left_coord = -ev.width / 2. + ev.width / (2 * COLS) as f32;
    }
}

fn highlight_system(windows: Res<Windows>, mut query: Query<(&mut HighlightColumn, &mut Transform)>, highlight_meta: Res<BoardAssets>) {
    let window = windows.get_primary().unwrap();
    if let Some(position) = window.cursor_position() {
        let new_column: u32 = (position.x / highlight_meta.highlight_size.x) as u32;

        let (mut col, mut t) = query.single_mut();
        t.translation = Vec3 {
            x: highlight_meta.left_coord + highlight_meta.highlight_size.x * new_column as f32,
            y: 0.0,
            z: 1.0,
        };
        t.scale = Vec3 {
            x: highlight_meta.highlight_size.x,
            y: highlight_meta.highlight_size.y,
            z: 0.0,
        };
        col.number = new_column;
    } else {
        // Cursor is outside of window
    }
}

fn stone_spawn_system(mut commands: Commands, stone_meta: Res<StoneAssets>, mouse: Res<Input<MouseButton>>,
                      mut query: Query<&HighlightColumn>, meta: Res<BoardAssets>, windows: Res<Windows>) {
    if mouse.just_pressed(MouseButton::Left) {
        let height = windows.primary().height();
        let col = query.single().number;
        let xpos = col as f32 * meta.highlight_size.x + meta.left_coord;

        commands.spawn_bundle(ColorMesh2dBundle {
            mesh: stone_meta.mesh_handle.clone().into(),
            material: stone_meta.mat_handle[0].clone().into(),
            transform: Transform::from_xyz(xpos, height / 2., 10.),
            ..default()
        }).insert(Stone {
            moving: true,
            velocity: -300.,
            board_position: UVec2::new(col, 0),
        });
    }
}

fn stone_moving_system(mut query: Query<(&mut Transform, &mut Stone)>, time: Res<Time>, board: Res<BoardAssets>) {
    for (mut t, mut stone) in query.iter_mut() {
        // t.translation += Vec3::new();
        if stone.moving {
            t.translation += Vec3::new(0., stone.velocity * time.delta_seconds(), 0.);
            let goal_pos = board.stone_position(stone.board_position);
            // info!("goal_pos = {:?}", goal_pos);
            if t.translation.y <= goal_pos.y {
                stone.moving = false;
            }
        }
    }
}