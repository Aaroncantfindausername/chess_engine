use bevy::prelude::*;
use chess_engine::gui::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(gui_plugin)
        .run();
}
