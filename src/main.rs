use futures_core::Stream;
use std::pin::Pin;
use tonic::{transport::Server, Request, Response, Status};

use async_stream::try_stream;
use chess_engine::rust_chess_server::{RustChess, RustChessServer};
use chess_engine::{game_message, game_start, promote_piece, PromotePiece, ChessMove, GameMessage, Position};

use chess::{Color, File, Rank, Square};

extern crate syslog;
#[macro_use]
extern crate log;

use syslog::{Facility, Formatter3164, BasicLogger};
use log::{LevelFilter};

mod engine;

pub mod chess_engine {
    tonic::include_proto!("rustchess");
}

#[derive(Debug, Default)]
pub struct ChessEngine {}

#[tonic::async_trait]
impl RustChess for ChessEngine {
    type GameStream =
        Pin<Box<dyn Stream<Item = Result<GameMessage, Status>> + Send + Sync + 'static>>;

    async fn game(
        &self,
        request: Request<tonic::Streaming<GameMessage>>,
    ) -> Result<Response<Self::GameStream>, Status> {
        let mut chess_eng = engine::Engine::default();
        let mut stream = request.into_inner();

        let output = try_stream! {
            while let Some(game_msg) = stream.message().await? {
                match game_msg.request.expect("GameMessage should have a request") {
                    game_message::Request::GameStart(msg) => {
                        if msg.player_color == game_start::Color::Black as i32 {
                            chess_eng.my_color = Color::Black;
                        } else {
                            chess_eng.my_color = Color::White;
                            // make move
                            let chess_move = chess_eng.choose_move()
                                .expect("Could not find valid move");
                            chess_eng.take_move(chess_move);
                            // send move to stream
                            yield create_game_msg(chess_move)
                        }
                    }
                    game_message::Request::ChessMove(msg) => {
                        let opponent_move = chess_engine_move_to_chess_move(msg);
                        chess_eng.take_move(opponent_move);
                        let chess_move_o = chess_eng.choose_move();
                        match chess_move_o {
                            Some(chess_move) => {
                                chess_eng.take_move(chess_move);
                                // send move to stream
                                yield create_game_msg(chess_move);
                            },
                            None => return,
                        }
                    }
                    _ => println!("other"),
                }
            }
        };
        Ok(Response::new(Box::pin(output) as Self::GameStream))
    }
}

fn create_game_msg(chess_move: chess::ChessMove) -> GameMessage {
    return GameMessage {
        request: Some(game_message::Request::ChessMove(ChessMove {
            original_position: Some(Position {
                file: chess_move.get_source().get_file().to_index() as u32,
                rank: chess_move.get_source().get_rank().to_index() as u32,
            }),
            new_position: Some(Position {
                file: chess_move.get_dest().get_file().to_index() as u32,
                rank: chess_move.get_dest().get_rank().to_index() as u32,
            }),
            promote_piece: None,
        })),
    };
}

fn chess_engine_move_to_chess_move(chess_move: ChessMove) -> chess::ChessMove {
    let original_position =chess_move.original_position.expect("Expected positon");
    let source_rank = original_position.rank as usize;
    let source_file = original_position.file as usize;
    let new_position = chess_move.new_position.expect("Expected positon");
    let dest_rank = new_position.rank as usize;
    let dest_file = new_position.file as usize;
    return chess::ChessMove::new(
        Square::make_square(Rank::from_index(source_rank), File::from_index(source_file)),
        Square::make_square(Rank::from_index(dest_rank), File::from_index(dest_file)),
        promote_piece_to_piece(chess_move.promote_piece),
    );
}

fn promote_piece_to_piece(p: Option<PromotePiece>) -> Option<chess::Piece> {
    if p == None {
        return None
    }
    match p.expect("").piece {
        _x if _x == promote_piece::Piece::Queen as i32 =>
            return Some(chess::Piece::Queen{}),
        _x if _x == promote_piece::Piece::Rook as i32 =>
            return Some(chess::Piece::Rook{}),
        _x if _x == promote_piece::Piece::Bishop as i32 =>
            return Some(chess::Piece::Bishop{}),
        _x if _x == promote_piece::Piece::Knight as i32 =>
            return Some(chess::Piece::Knight{}),
        _ => return None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "chessengine".into(),
        pid: 0,
    };

    let logger = syslog::unix(formatter).expect("could not connect to syslog");
    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
            .map(|()| log::set_max_level(LevelFilter::Info)).expect("could not set up logger");

    info!("starting chessengine server on :50051");

    let addr = "[::1]:50051".parse()?;
    let engine = ChessEngine::default();

    Server::builder()
        .add_service(RustChessServer::new(engine))
        .serve(addr)
        .await?;

    Ok(())
}
