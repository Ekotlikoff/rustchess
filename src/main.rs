use futures_core::Stream;
use std::pin::Pin;
use tonic::{transport::Server, Request, Response, Status};

use async_stream::try_stream;
use chess_engine::rust_chess_server::{RustChess, RustChessServer};
use chess_engine::{game_message, game_start, ChessMove, GameMessage, Position};

use chess::{Color, File, Rank, Square};

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
                        let chess_move = chess_eng.choose_move()
                            .expect("Could not find valid move");
                        chess_eng.take_move(chess_move);
                        // send move to stream
                        yield create_game_msg(chess_move)
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
        })),
    };
}

fn chess_engine_move_to_chess_move(chess_move: chess_engine::ChessMove) -> chess::ChessMove {
    let original_position =chess_move.original_position.expect("Expected positon");
    let source_rank = original_position.rank as usize;
    let source_file = original_position.file as usize;
    let new_position = chess_move.new_position.expect("Expected positon");
    let dest_rank = new_position.rank as usize;
    let dest_file = new_position.file as usize;
    return chess::ChessMove::new(
        Square::make_square(Rank::from_index(source_rank), File::from_index(source_file)),
        Square::make_square(Rank::from_index(dest_rank), File::from_index(dest_file)),
        Some(chess::Piece::Queen),
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let engine = ChessEngine::default();

    Server::builder()
        .add_service(RustChessServer::new(engine))
        .serve(addr)
        .await?;

    Ok(())
}
