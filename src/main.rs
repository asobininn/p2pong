#![allow(dead_code)]

mod pong;

use std::net::SocketAddr;
use bevy::prelude::*;
use bevy_ggrs::{GGRSApp, GGRSPlugin};
use ggrs::*;
use structopt::StructOpt;
use crate::pong::*;

const INPUT_SIZE: usize = std::mem::size_of::<u8>();
const FPS: u32 = 60;
const ROLLBACK_DEFAULT: &str = "rollback_default";

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    local_port: u16,
    #[structopt(short, long)]
    players: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let opt = Opt::from_args();
    if opt.players.len() != NUM_PLAYERS {
        panic!("");
    }

    let mut p2p_sess = P2PSession::new(NUM_PLAYERS as u32, INPUT_SIZE, opt.local_port)?;
    p2p_sess.set_fps(FPS).expect("Invalid fps");
    p2p_sess.set_sparse_saving(true)?;
    

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "P2Pong".to_string(),
            width: 1200.,
            height: 720.,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(opt)
        .insert_resource(ActiveBalls(0))
        .insert_resource(LastWinner(LEFT_PADDLE))
        .add_plugins(DefaultPlugins)
        .add_plugin(GGRSPlugin)
        .add_startup_system(start_p2p_session)
        .add_startup_system(setup_system)
        .add_startup_system(spawn_ball_system)
        .with_p2p_session(p2p_sess)
        .with_update_frequency(FPS)
        .with_input_system(input)
        .register_rollback_type::<Transform>()
        .insert_rollback_resource(FrameCount { frame: 0 })
        .with_rollback_schedule(
            Schedule::default().with_stage(
                ROLLBACK_DEFAULT, 
                SystemStage::single_threaded()
                    .with_system(spawn_ball_system)
                    .with_system(ball_collision_system)
                    .with_system(move_paddle_system)
                    .with_system(move_ball_system)
                    .with_system(scoreboard_system)
                    .with_system(increase_frame_system)
            ),
        )
        .run();

    Ok(())
}

fn start_p2p_session(
    mut p2p_sess: ResMut<P2PSession>,
    opt: Res<Opt>,
) {
    let mut local_handle = 0;

    // add players
    for (i, player_addr) in opt.players.iter().enumerate() {
        if player_addr == "localhost" {
            p2p_sess.add_player(PlayerType::Local, i).unwrap();
            local_handle = i;
        } else {
            let remote_addr: SocketAddr = 
                player_addr.parse().expect("Invalid remote player address");
            p2p_sess
                .add_player(PlayerType::Remote(remote_addr), i).unwrap();
        }
    }
    p2p_sess.set_frame_delay(2, local_handle).unwrap();
    p2p_sess.start_session().unwrap();
}