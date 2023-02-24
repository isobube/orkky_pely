use bevy::diagnostic::Diagnostics;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::math::vec2;
use bevy::prelude::App;
use bevy::prelude::AssetServer;
use bevy::prelude::Assets;
use bevy::prelude::Camera2dBundle;
use bevy::prelude::Color;
use bevy::prelude::Commands;
use bevy::prelude::Mesh;
use bevy::prelude::NodeBundle;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Rect;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::SystemSet;
use bevy::prelude::TextBundle;
use bevy::prelude::Transform;
use bevy::prelude::Vec2;
use bevy::prelude::Vec3;
use bevy::prelude::shape;
use bevy::sprite;
use bevy::sprite::ColorMaterial;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Sprite;
use bevy::sprite::SpriteBundle;
use bevy::text::Text;
use bevy::text::TextSection;
use bevy::text::TextStyle;
use bevy::ui::PositionType;
use bevy::ui::Size;
use bevy::ui::Style;
use bevy::ui::UiRect;
use bevy::ui::Val;

use crate::types::GameState;



#[derive(Default)]
pub struct GameUiPlugin;


impl Plugin for GameUiPlugin {
	fn build(&self, app: &mut App) {app

        .add_plugin(FrameTimeDiagnosticsPlugin::default())
		.add_plugin(LogDiagnosticsPlugin::default())
        .add_system_set(
            SystemSet::on_enter(GameState::Game)
                .with_system(setupui)
                .with_system(setup_health_ui)
                
   
        )

        .add_system_set(
            SystemSet::on_update(GameState::Game)
            .with_system(fps_display_system)
            
        );
    

    }
}


fn fps_display_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[0].value = format!("FPS: {:.0}", average);
            }
        }
    }
}




fn setupui(mut commands: Commands, asset_server: Res<AssetServer>,) {
    let font = asset_server.load("FiraSans-Bold.ttf");
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "FPS: 0.00".to_string(),
                    style: TextStyle {
                        font: font,
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect {
				top: Val::Px(10.0),
				left: Val::Px(10.0),
				right: Val::Auto,
				bottom: Val::Auto
            },
            ..Default::default()
        },
        ..Default::default()
    });


}






fn setup_health_ui(mut commands: Commands) {

commands.spawn(NodeBundle {
    style: Style {
        size: Size::new(Val::Px(200.0), Val::Px(10.0)),
        position_type: PositionType::Absolute,
        position: UiRect {
            
            left: Val::Px(580.0),
            bottom: Val::Px(550.0),
            ..Default::default()
        },
        border: UiRect::all(Val::Px(20.0)),
        ..Default::default()
    },
    background_color: Color::GREEN.into(),
    ..Default::default()
});


}


struct Health {
    current: f32,
    max: f32,
}



