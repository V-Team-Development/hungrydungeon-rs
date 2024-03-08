pub mod actors;
pub mod input_parsing;
pub mod rooms;
use actors::{ai::*, organs::OrganPlugin};
use async_channel::{Receiver, Sender};
use bevy::{
    app::{RunMode, ScheduleRunnerPlugin},
    prelude::*,
};
use std::time::Duration;

use input_parsing::parse_game_input;

const GAME_LOOP_MILIS: u64 = 100;

#[derive(Resource)]
struct GameInputReceiver(Receiver<String>);

#[derive(Resource)]
struct GameOutputSender(Sender<String>);

pub fn launch_game(rx: Receiver<String>, tx: Sender<String>) {
    App::new()
        .insert_resource(GameInputReceiver(rx))
        .insert_resource(GameOutputSender(tx))
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin {
                run_mode: RunMode::Loop {
                    wait: Some(Duration::from_millis(GAME_LOOP_MILIS)),
                },
            }),
            AiPlugin,
            OrganPlugin,
        ))
        .add_systems(Startup, spawn_test)
        .add_systems(Update, receive_input)
        .run();
}

fn spawn_test(mut commands: Commands) {
    commands.spawn(rooms::GameRoom).with_children(|mut room| {
        slime::spawn(&mut room);
        slimegirl::spawn(&mut room);
    });
}

fn receive_input(rs: Res<GameInputReceiver>) {
    while let Ok(msg) = rs.0.try_recv() {
        // parse message, send appropriate event
        // future versions will include the id of the sender, not just the message
        let parsed = parse_game_input(&msg);
        match parsed {
            input_parsing::GameInputParseResult::Attack { target } => {
                println!("Received attack target {target}")
            }
            input_parsing::GameInputParseResult::Devour { target, organ } => {
                println!("Received devour {target} with organ {organ}")
            }
            input_parsing::GameInputParseResult::MoveRoom { room } => {
                println!("Received move to room {room}")
            }
            input_parsing::GameInputParseResult::Struggle => println!("Received struggle"),
            input_parsing::GameInputParseResult::Error(e) => println!("Parsing error: {e}"),
            input_parsing::GameInputParseResult::Unknown => println!("Received unknown input"),
        }
    }
}