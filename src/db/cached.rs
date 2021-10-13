use crate::db::BoardsDatabase;
use crate::errors::CustomResult;
use crate::models::{Board, Task};
use actix_web::web::Bytes;
use redis::{AsyncCommands, Client};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Cached<T: BoardsDatabase> {
    db: T,
    redis_client: Client,
}

impl<T: BoardsDatabase> Cached<T> {
    pub fn new(db: T, redis_client: Client) -> Self {
        Self { db, redis_client }
    }

    async fn set_to_cache(&self, key: &str, value: &Board) -> CustomResult<()> {
        let mut connection = self.redis_client.get_async_connection().await?;
        let serialized = serde_json::ser::to_string(value)?;
        redis::pipe()
            .set(&key, &serialized)
            .expire(&key, 60)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    async fn get_from_cache(&self, key: &str) -> CustomResult<Board> {
        let mut connection = self.redis_client.get_async_connection().await?;
        let serialized = connection.get::<_, String>(&key).await?;
        let board = serde_json::de::from_str(&serialized)?;
        Ok(board)
    }

    async fn delete_from_cache(&self, key: &str) -> CustomResult<()> {
        let mut connection = self.redis_client.get_async_connection().await?;
        connection.del(&key).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: BoardsDatabase> BoardsDatabase for Cached<T> {
    async fn create_board(&self, data: Board) -> CustomResult<Board> {
        todo!()
        // let board = self.db.create_board(data).await?;
        // Ok(board)
    }

    async fn read_boards(&self) -> CustomResult<Vec<Board>> {
        self.db.read_boards().await
    }

    async fn read_board(&self, id: &str) -> CustomResult<Board> {
        todo!()
        // Ok(match self.get_from_cache(id).await {
        //     Ok(b) => b,
        //     _ => {
        //         let board = self.db.get_board(id).await?;
        //         self.set_to_cache(id, &board).await?;
        //         board
        //     }
        // })
    }

    async fn update_board(&self, id: &str, data: Board) -> CustomResult<Board> {
        todo!()
        // self.db.put_board(id, data).await
    }

    async fn delete_board(&self, id: &str) -> CustomResult<Board> {
        todo!()
        // self.delete_from_cache(id).await?;
        // self.db.delete_board(id).await
    }

    async fn subscribe_on_board_updates(
        &self,
        board_id: &str,
        tx: Sender<CustomResult<Bytes>>,
    ) -> Receiver<CustomResult<Bytes>> {
        todo!()
    }
}

    // async fn read_board_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
    //     todo!()
    //     // self.get_board(board_id).await.map(|board| board.tasks)
    // }
    //
    // async fn create_task(&self, board_id: &str, data: TaskData) -> CustomResult<Task> {
    //     todo!();
    //     // let task = self.db.create_task(board_id, data).await?;
    //     // self.delete_from_cache(board_id).await?;
    //     // Ok(task)
    // }
    //
    // async fn read_task(&self, task_id: &str) -> CustomResult<Task> {
    //     todo!()
    // }
    //
    // async fn update_task(&self, task_id: &str, data: TaskData) -> CustomResult<Task> {
    //     todo!()
    // }
    //
    // async fn delete_task(&self, task_id: &str) -> CustomResult<Task> {
    //     todo!()
    // }