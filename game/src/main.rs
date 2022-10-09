use std::cmp::min;
use std::ops::Mul;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::sprite::Material2d;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::window::{close_on_esc, WindowResized};
use futures_lite::future;
use c4solver::board::{BitBoard, FieldType};
use c4solver::engine::{SolveResult, SolverType};

const COLS: u32 = c4solver::board::BOARD_WIDTH as u32;
const ROWS: u32 = c4solver::board::BOARD_HEIGHT as u32;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_player_turn)
                .with_system(player_control_system)
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_computer_turn)
                .with_system(computer_task_system)
        )
        .add_system(highlight_system)
        .add_system(stone_moving_system)
        .add_system(resize_handler)
        .add_system(close_on_esc)
        .run();
}

#[derive(Debug, Eq, PartialEq)]
enum MultiplayerKind {
    Player,
    Computer,
    End,
}

fn run_if_player_turn(mode: Res<MultiplayerKind>, mut query: Query<&Stone>) -> ShouldRun {
    let moving = query.iter().any(|x| x.moving);
    if *mode == MultiplayerKind::Player && !moving {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_if_computer_turn(mode: Res<MultiplayerKind>, mut query: Query<&Stone>) -> ShouldRun {
    let moving = query.iter().any(|x| x.moving);
    if *mode == MultiplayerKind::Computer && !moving {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

#[derive(Component)]
struct Board;

#[derive(Component)]
struct HighlightColumn {
    number: u32,
}

enum Player {
    Player1,
    Player2,
}

struct BoardAssets {
    highlight_size: Vec2,
    left_coord: f32,
    mesh_handle: Handle<Mesh>,
    board: BitBoard,
}

impl BoardAssets {
    fn stone_position(&self, game_position: UVec2) -> Vec2 {
        let down_coord = -self.highlight_size.y / 2.;

        let x_pos = self.left_coord + game_position.x as f32 * self.highlight_size.x;
        let y_pos = down_coord + (game_position.y as f32 + 0.5) * (self.highlight_size.y / ROWS as f32);
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

/// Returns the move the engine calculated
#[derive(Component)]
struct ComputeNextMove(Task<SolveResult>);

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

fn computer_task_system(mut commands: Commands,
                        mut compute_tasks: Query<(Entity, &mut ComputeNextMove)>,
                        mut board: ResMut<BoardAssets>, mut game: ResMut<MultiplayerKind>,
                        stone_meta: Res<StoneAssets>) {
    let count = compute_tasks.iter().count();
    if count > 0 {
        for (entity, mut task) in &mut compute_tasks {
            if let Some(next_move) = future::block_on(future::poll_once(&mut task.0)) {
                info!("Next move = {:?}", next_move);
                commands.entity(entity).remove::<ComputeNextMove>();
                let coords = BitBoard::move_to_coords(next_move.mov);
                if let Some(coords) = coords {
                    board.board.set_at(next_move.mov, Some(FieldType::Player));

                    info!("Coords = ({}, {})", coords.0, coords.1);
                    let xpos = coords.0 as f32 * board.highlight_size.x + board.left_coord;
                    commands.spawn_bundle(ColorMesh2dBundle {
                        mesh: stone_meta.mesh_handle.clone().into(),
                        material: stone_meta.mat_handle[1].clone().into(),
                        transform: Transform::from_xyz(xpos, board.highlight_size.y / 2., 10.),
                        ..default()
                    }).insert(Stone {
                        moving: true,
                        velocity: -1000.,
                        board_position: UVec2::new(coords.0, coords.1),
                    });

                    *game = MultiplayerKind::Player;
                } else {
                    *game = MultiplayerKind::End;
                }
            }
        }
    } else {
        let thread_pool = AsyncComputeTaskPool::get();
        let b = board.board.clone();
        let task = thread_pool.spawn(async move {
            c4solver::engine::solve(&b, 17, SolverType::Strong)
        });
        commands.spawn().insert(ComputeNextMove(task));

        info!("Spawned compute task!");
    }
}

fn setup(mut commands: Commands, mut windows: ResMut<Windows>, mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(MultiplayerKind::Player);
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
            board: BitBoard::default(),
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

fn player_control_system(mut commands: Commands, stone_meta: Res<StoneAssets>, mouse: Res<Input<MouseButton>>,
                         mut query: Query<&HighlightColumn>, mut meta: ResMut<BoardAssets>, windows: Res<Windows>,
                         mut state: ResMut<MultiplayerKind>) {
    if mouse.just_pressed(MouseButton::Left) {
        let height = windows.primary().height();
        let col = query.single().number;
        let xpos = col as f32 * meta.highlight_size.x + meta.left_coord;

        let pos = meta.board.stone_position(col as u64);
        if let Some(row) = pos {
            info!("Col = {:?}", pos);
            // The opponent is the human player. We are the computer!
            meta.board = meta.board.play_column(col as u8, FieldType::Opponent).unwrap();
            commands.spawn_bundle(ColorMesh2dBundle {
                mesh: stone_meta.mesh_handle.clone().into(),
                material: stone_meta.mat_handle[0].clone().into(),
                transform: Transform::from_xyz(xpos, height / 2., 10.),
                ..default()
            }).insert(Stone {
                moving: true,
                velocity: -1000.,
                board_position: UVec2::new(col, row as u32),
            });
            *state = MultiplayerKind::Computer;
        }
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
                t.translation.y = goal_pos.y;
                stone.moving = false;
            }
        }
    }
}