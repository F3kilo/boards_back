use crate::db::Database;
use crate::errors::{CustomError, CustomResult};
use crate::models::{Board, BoardData, Task, TaskData, TaskStage};
use actix_web::web::Bytes;
use mongodb::bson::oid::ObjectId;
use tokio::sync::mpsc::{self, Receiver};
use tokio::time::Duration;

pub struct Boards {
    db: Box<dyn Database>,
}

impl Boards {
    pub fn new(db: Box<dyn Database>) -> Self {
        Self { db }
    }

    pub async fn list_boards(&self) -> CustomResult<Vec<Board>> {
        self.db.get_boards().await
    }

    pub async fn create_board(&self, board_data: BoardData) -> CustomResult<Board> {
        self.db.create_board(board_data).await
    }

    pub async fn board(&self, id: &str) -> CustomResult<Board> {
        self.db.get_board(id).await
    }

    pub async fn put_board(&self, id: &str, board_data: BoardData) -> CustomResult<Board> {
        self.db.put_board(id, board_data).await
    }

    pub async fn delete_board(&self, id: &str) -> CustomResult<Board> {
        self.db.delete_board(id).await
    }

    pub async fn list_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
        self.db.get_tasks(board_id).await
    }

    pub async fn get_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        self.db.get_task(board_id, task_id).await
    }

    pub async fn create_task(&self, board_id: &str, task_data: TaskData) -> CustomResult<Task> {
        self.db.create_task(board_id, task_data).await
    }

    pub async fn update_task(
        &self,
        board_id: &str,
        task_id: &str,
        task_data: TaskData,
    ) -> CustomResult<Task> {
        self.db.put_task(board_id, task_id, task_data).await
    }

    pub async fn delete_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        self.db.delete_task(board_id, task_id).await
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
