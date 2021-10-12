pub mod mongo;

use crate::errors::CustomResult;
use crate::models::{Board, BoardData, Task, TaskData};
use actix_web::web::Bytes;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait::async_trait]
pub trait Database: Send + Sync {
    async fn get_boards(&self) -> CustomResult<Vec<Board>>;
    async fn create_board(&self, data: BoardData) -> CustomResult<Board>;
    async fn get_board(&self, id: &str) -> CustomResult<Board>;
    async fn put_board(&self, id: &str, data: BoardData) -> CustomResult<Board>;
    async fn delete_board(&self, id: &str) -> CustomResult<Board>;

    async fn get_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>>;
    async fn create_task(&self, board_id: &str, data: TaskData) -> CustomResult<Task>;
    async fn get_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task>;
    async fn put_task(&self, board_id: &str, task_id: &str, data: TaskData) -> CustomResult<Task>;
    async fn delete_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task>;

    async fn subscribe_on_board_updates(
        &self,
        board_id: &str,
        tx: Sender<CustomResult<Bytes>>,
    ) -> Receiver<CustomResult<Bytes>>;
}
