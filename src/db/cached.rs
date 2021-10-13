use crate::db::{BoardsDatabase, EventMsgReceiver, TasksDatabase};
use crate::errors::CustomResult;
use crate::models::{Board, Task};
use redis::{AsyncCommands, Client};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone)]
pub struct Cached<T: Clone> {
    db: T,
    redis_client: Client,
}

impl<T: Clone> Cached<T> {
    pub fn new(db: T, redis_client: Client) -> Self {
        Self { db, redis_client }
    }

    async fn cache_set<V: Serialize>(&self, key: &str, value: &V) -> CustomResult<()> {
        let mut connection = self.redis_client.get_async_connection().await?;
        let serialized = serde_json::ser::to_string(value)?;
        redis::pipe()
            .set(&key, &serialized)
            .expire(&key, 60)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    async fn cache_get<V: DeserializeOwned>(&self, key: &str) -> CustomResult<V> {
        let mut connection = self.redis_client.get_async_connection().await?;
        let serialized = connection.get::<_, String>(&key).await?;
        let deserialized = serde_json::de::from_str(&serialized)?;
        Ok(deserialized)
    }

    async fn cahce_delete(&self, key: &str) -> CustomResult<()> {
        let mut connection = self.redis_client.get_async_connection().await?;
        connection.del(&key).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: BoardsDatabase + Clone> BoardsDatabase for Cached<T> {
    async fn create_board(&self, data: Board) -> CustomResult<Board> {
        self.db.create_board(data).await
    }

    async fn read_boards(&self) -> CustomResult<Vec<Board>> {
        self.db.read_boards().await
    }

    async fn read_board(&self, id: &str) -> CustomResult<Board> {
        Ok(match self.cache_get(id).await {
            Ok(b) => {
                log::trace!("Read board #{} from cache", id);
                b
            },
            _ => {
                log::trace!("Read board #{} from database", id);
                let board = self.db.read_board(id).await?;
                self.cache_set(id, &board).await?;
                board
            }
        })
    }

    async fn update_board(&self, id: &str, data: Board) -> CustomResult<Board> {
        let updated = self.db.update_board(id, data).await?;
        self.cache_set(id, &updated).await?;
        Ok(updated)
    }

    async fn delete_board(&self, id: &str) -> CustomResult<Board> {
        self.cahce_delete(id).await?;
        self.db.delete_board(id).await
    }

    async fn subscribe_on_board_updates(&self, _board_id: &str) -> CustomResult<EventMsgReceiver> {
        todo!()
    }
}

#[async_trait::async_trait]
impl<T: TasksDatabase + Clone> TasksDatabase for Cached<T> {
    async fn create_task(&self, task: Task) -> CustomResult<Task> {
        self.db.create_task(task).await
    }

    async fn read_tasks(&self) -> CustomResult<Vec<Task>> {
        self.db.read_tasks().await
    }

    async fn read_board_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
        self.db.read_board_tasks(board_id).await
    }

    async fn read_task(&self, id: &str) -> CustomResult<Task> {
        Ok(match self.cache_get(id).await {
            Ok(t) => {
                log::trace!("Read task #{} from cache", id);
                t
            },
            _ => {
                log::trace!("Read task #{} from database", id);
                let task = self.db.read_task(id).await?;
                self.cache_set(id, &task).await?;
                task
            }
        })
    }

    async fn update_task(&self, id: &str, task: Task) -> CustomResult<Task> {
        let updated = self.db.update_task(id, task).await?;
        self.cache_set(id, &updated).await?;
        Ok(updated)
    }

    async fn delete_task(&self, id: &str) -> CustomResult<Task> {
        self.cahce_delete(id).await?;
        self.db.delete_task(id).await
    }
}
