pub mod cached;
pub mod mongo;

use crate::errors::CustomResult;
use crate::models::{Board, Task};
use actix_web::web::Bytes;
use tokio::sync::mpsc::Receiver;

pub type EventMsgResult = CustomResult<Bytes>;
pub type EventMsgReceiver = Receiver<EventMsgResult>;

#[async_trait::async_trait]
pub trait BoardsDatabase: Send + Sync {
    async fn create_board(&self, board: Board) -> CustomResult<Board>;
    async fn read_boards(&self) -> CustomResult<Vec<Board>>;
    async fn read_board(&self, id: &str) -> CustomResult<Board>;
    async fn update_board(&self, id: &str, board: Board) -> CustomResult<Board>;
    async fn delete_board(&self, id: &str) -> CustomResult<Board>;

    async fn subscribe_on_board_updates(&self, board_id: &str) -> CustomResult<EventMsgReceiver>;
}

#[async_trait::async_trait]
pub trait TasksDatabase: Send + Sync {
    async fn create_task(&self, board_id: &str, task: Task) -> CustomResult<Task>;
    async fn read_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>>;
    async fn read_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task>;
    async fn update_task(&self, board_id: &str, task_id: &str, task: Task) -> CustomResult<Task>;
    async fn delete_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task>;
}
