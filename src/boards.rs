use crate::db::{BoardsDatabase, EventMsgReceiver};
use crate::errors::CustomResult;
use crate::models::Board;

pub struct Boards {
    db: Box<dyn BoardsDatabase>,
}

impl Boards {
    pub fn new(db: Box<dyn BoardsDatabase>) -> Self {
        Self { db }
    }

    pub async fn read_boards(&self) -> CustomResult<Vec<Board>> {
        self.db.read_boards().await
    }

    pub async fn create_board(&self, board: Board) -> CustomResult<Board> {
        self.db.create_board(board).await
    }

    pub async fn read_board(&self, id: &str) -> CustomResult<Board> {
        self.db.read_board(id).await
    }

    pub async fn update_board(&self, id: &str, board: Board) -> CustomResult<Board> {
        self.db.update_board(id, board).await
    }

    pub async fn delete_board(&self, id: &str) -> CustomResult<Board> {
        self.db.delete_board(id).await
    }

    pub async fn subscribe_on_board_updates(
        &self,
        board_id: &str,
    ) -> CustomResult<EventMsgReceiver> {
        self.db.subscribe_on_board_updates(board_id).await
    }
}
