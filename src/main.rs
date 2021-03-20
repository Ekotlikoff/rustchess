use futures_core::Stream;
use std::pin::Pin;
use tonic::{transport::Server, Request, Response, Status};

use chess_engine::rust_chess_server::{RustChess, RustChessServer};
use chess_engine::GameMessage;

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
        _request: Request<tonic::Streaming<GameMessage>>,
    ) -> Result<Response<Self::GameStream>, Status> {
        unimplemented!()
    }
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
