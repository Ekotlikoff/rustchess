pub struct Engine {
    pub board: chess::Board,
    pub my_color: chess::Color,
}

impl Engine {
    pub fn choose_move(&self) -> Option<chess::ChessMove> {
        let mut iterable = chess::MoveGen::new_legal(&self.board);
        return iterable.next();
    }

    pub fn take_move(&mut self, m: chess::ChessMove) {
        let mut board = chess::Board::default();
        self.board.make_move(m, &mut board);
        self.board = board;
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            board: chess::Board::default(),
            my_color: chess::Color::Black,
        }
    }
}
