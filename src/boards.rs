use crate::db::BoardsDatabase;
use crate::errors::{CustomError, CustomResult};
use crate::models::Board;
use actix_web::web::Bytes;
use tokio::sync::mpsc::{self, Receiver};
use tokio::time::Duration;

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

    pub async fn subscribe_on_board_changes(
        &self,
        board_id: &str,
    ) -> Result<Receiver<Result<Bytes, CustomError>>, CustomError> {
        let (tx, rx) = mpsc::channel(100);
        tx.send(Ok(Bytes::from("subscribed = true\r\n")))
            .await
            .unwrap();

        let id = board_id.to_string();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(2)).await;
                let msg = format!("test event from board {}\r\n", id);
                tx.send(Ok(Bytes::from(msg))).await.unwrap();
            }
        });

        Ok(rx)
    }
}
