use bevy::prelude::*;
use chess_engine::gui::*;
use chess_engine::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_scene))
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("chess_board.png")),
        Transform::from_xyz(0.0, 0.0, -1.0).with_scale(Vec3 {
            x: BOARD_SCALE,
            y: BOARD_SCALE,
            z: 1.0,
        }),
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
