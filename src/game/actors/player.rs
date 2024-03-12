mod parse_input;
mod process_input;
use bevy::prelude::*;
use process_input::{map_input_to_event, ParsedPlayerEvent};

use crate::game::{rooms::GameRoom, SendMessageToBotEvent};

use super::{organs::Organ, Actor};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerInputStringEvent>()
            .add_event::<PlayerAttackEvent>()
            .add_event::<PlayerDevourEvent>()
            .add_event::<PlayerMoveRoomEvent>()
            .add_event::<PlayerStruggleEvent>()
            .add_systems(
                Update,
                (
                    process_event,
                    player_attack,
                    player_devour,
                    player_move_room,
                    player_struggle,
                ),
            );
    }
}

#[derive(Event)]
pub struct PlayerInputStringEvent(pub u64, pub String);

#[derive(Event)]
pub struct PlayerAttackEvent {
    pub player: Entity,
    pub target: Entity,
}

#[derive(Event)]
pub struct PlayerDevourEvent {
    pub player: Entity,
    pub prey: Entity,
    pub organ: Entity,
}

#[derive(Event)]
pub struct PlayerMoveRoomEvent {
    pub player: Entity,
    pub room: Entity,
}

#[derive(Event)]
pub struct PlayerStruggleEvent {
    pub player: Entity,
}

#[derive(Component)]
pub struct Player(pub u64);

#[allow(clippy::too_many_arguments)]
fn process_event(
    q_actors: Query<(Entity, &Player)>,
    q_actor_names: Query<(Entity, &Name), With<Actor>>,
    q_organ_names: Query<(Entity, &Name), With<Organ>>,
    q_room_names: Query<(Entity, &Name), With<GameRoom>>,
    mut reader: EventReader<PlayerInputStringEvent>,
    mut w_attack: EventWriter<PlayerAttackEvent>,
    mut w_devour: EventWriter<PlayerDevourEvent>,
    mut w_move: EventWriter<PlayerMoveRoomEvent>,
    mut w_struggle: EventWriter<PlayerStruggleEvent>,
    mut w_err: EventWriter<SendMessageToBotEvent>,
) {
    for ev in reader.read() {
        // this is a three-step process.
        // First, the string input is parsed to figure out what the player wants to do.
        // Second, the parsed event is checked to make sure the given names are valid entities.
        // Third, the event is passed to individual systems, which check whether the
        // named entities are what they're supposed to be and whether the action is possible.
        match map_input_to_event(
            ev.0,
            &ev.1,
            &q_actors,
            &q_actor_names,
            &q_organ_names,
            &q_room_names,
        ) {
            Ok(parseres) => match parseres {
                ParsedPlayerEvent::Attack(e) => {
                    w_attack.send(e);
                }
                ParsedPlayerEvent::Devour(e) => {
                    w_devour.send(e);
                }
                ParsedPlayerEvent::Move(e) => {
                    w_move.send(e);
                }
                ParsedPlayerEvent::Struggle(e) => {
                    w_struggle.send(e);
                }
            },
            Err(e) => {
                // send an event back to the discord bot to print it
                w_err.send(SendMessageToBotEvent { message: e });
            }
        }
    }
}

fn player_attack(
    mut reader: EventReader<PlayerAttackEvent>,
    mut query: Query<(&Name, &mut Actor)>,
) {
    for ev in reader.read() {
        let actors = query.get_many_mut([ev.player, ev.target]);
        if let Ok([attacker, mut target]) = actors {
            // check if the slime is still active, if the target is still in reach, if its still alive
            // the "is this target valid" check should be the same code both here and above
            target.1.health_current -= attacker.1.attack;
            println!(
                "{} attacks {}, dealing {} damage!",
                attacker.0, target.0, attacker.1.attack
            );
        }
    }
}

fn player_devour(
    mut reader: EventReader<PlayerDevourEvent>,
    q_organs: Query<(Entity, &Organ)>,
    q_names: Query<&Name>,
    mut commands: Commands,
) {
    for ev in reader.read() {
        // do some calculation based on the stats of the parent to determine if success

        // sets parent of target to the organ
        let organ = q_organs
            .get(ev.organ)
            .expect("Organ has vanished before using!");
        commands.entity(ev.prey).set_parent(organ.0);
        let pred = q_names.get(ev.player).expect("Actor should have name!");
        let prey = q_names.get(ev.prey).expect("Actor should have name!");
        let organ_name = q_names.get(organ.0).expect("Organ should have name!");
        println!("{} devours {} with their {}!", pred, prey, organ_name);
    }
}

fn player_move_room(mut reader: EventReader<PlayerMoveRoomEvent>) {
    for ev in reader.read() {
        println!("Move event");
    }
}

fn player_struggle(mut reader: EventReader<PlayerStruggleEvent>) {
    for ev in reader.read() {
        println!("Struggle event");
    }
}