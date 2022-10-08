use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::{close_on_esc, WindowResized};

const COLUMNS: u32 = 7;
const ROWS: u32 = 6;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(highlight_system)
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

struct HighlightMeta {
    highlight_size: Vec2,
    left_coord: f32,
}

struct BoardShape {
    size: Vec2,
    rows: u32,
    cols: u32,
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

    let highlight_size = Vec2::new(width / COLUMNS as f32, height);
    let left_coord = -width / 2. + highlight_size.x / 2.;

    println!("Creating board mesh");
    let board_mesh = meshes.add(
        Mesh::from(BoardShape {
            size: Vec2::new(1., 1.),
            rows: ROWS,
            cols: COLUMNS,
        })
    );
    let board_material = materials.add(
        ColorMaterial { color: Color::rgba(0., 0., 0., 1.), texture: None }
    );

    commands.spawn_bundle(
        ColorMesh2dBundle {
            mesh: board_mesh.into(),
            material: board_material,
            transform: Transform::default().with_scale(Vec3::new(width, height, 1.)),
            ..default()
            // transform: Default::default(),
            // global_transform: Default::default(),
        }
    ).insert(Board);

    commands.insert_resource(
        HighlightMeta {
            highlight_size,
            left_coord,
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
                  mut meta: ResMut<HighlightMeta>) {
    for ev in ev_reader.iter() {
        let mut t = query.single_mut();
        t.scale = Vec3::new(ev.width, ev.height, 1.);
        meta.highlight_size = Vec2::new(ev.width / COLUMNS as f32, ev.height);
        meta.left_coord = - ev.width / 2. + ev.width/(2*COLUMNS) as f32 ;
    }
}

fn highlight_system(windows: Res<Windows>, mut query: Query<(&mut HighlightColumn, &mut Transform)>, highlight_meta: Res<HighlightMeta>) {
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
            z: 0.0
        };
        col.number = new_column;

        // cursor is inside of window
        // println!("x = {}, y = {}", position.x, position.y);
    } else {
        // Cursor is outside of window
    }
}