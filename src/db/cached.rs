use crate::db::{BoardsDatabase, EventMsgReceiver, EventMsgResult, TasksDatabase};
use crate::errors::CustomResult;
use crate::models::{Board, Task};
use actix_web::web::Bytes;
use redis::{AsyncCommands, Client, Commands, FromRedisValue};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct Cached<T: Clone> {
    db: T,
    redis_client: Client,
}

impl<T: Clone> Cached<T> {
    pub fn new(db: T, redis_client: Client) -> Self {
        Self { db, redis_client }
    }

    async fn cache_set<V: Serialize>(&self, key: &str, field: &str, value: &V) -> CustomResult<()> {
        let mut connection = self.redis_client.get_async_connection().await?;
        let serialized = serde_json::ser::to_string(value)?;
        redis::pipe()
            .hset(key, field, &serialized)
            .expire(&key, 60)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }

    async fn cache_get<V: DeserializeOwned>(&self, key: &str, field: &str) -> CustomResult<V> {
        let mut connection = self.redis_client.get_async_connection().await?;
        let serialized = connection.hget::<_, _, String>(&key, field).await?;
        let deserialized = serde_json::de::from_str(&serialized)?;
        Ok(deserialized)
    }

    async fn cache_delete_field(&self, key: &str, field: &str) -> CustomResult<()> {
        let mut connection = self.redis_client.get_async_connection().await?;
        connection.hdel(&key, field).await?;
        Ok(())
    }

    async fn cache_delete_key(&self, key: &str) -> CustomResult<()> {
        let mut connection = self.redis_client.get_async_connection().await?;
        connection.del(&key).await?;
        Ok(())
    }

    fn pub_sub_channel_name(board_id: &str) -> String {
        format!("BOARD_EVENT_{}", board_id)
    }

    fn pub_board_updated(&self, board_id: &str) -> CustomResult<()> {
        let channel = Self::pub_sub_channel_name(board_id);
        let mut client = self.redis_client.clone();
        client.publish(&channel, "Board updated")?;
        Ok(())
    }

    fn pub_board_deleted(&self, board_id: &str) -> CustomResult<()> {
        let channel = Self::pub_sub_channel_name(board_id);
        let mut client = self.redis_client.clone();
        client.publish(&channel, "Board deleted")?;
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
        Ok(match self.cache_get(id, "board").await {
            Ok(b) => {
                log::trace!("Read board #{} from cache", id);
                b
            }
            _ => {
                log::trace!("Read board #{} from database", id);
                let board = self.db.read_board(id).await?;
                self.cache_set(id, "board", &board).await?;
                board
            }
        })
    }

    async fn update_board(&self, id: &str, data: Board) -> CustomResult<Board> {
        let updated = self.db.update_board(id, data).await?;
        self.cache_set(id, "board", &updated).await?;
        self.pub_board_updated(id)?;
        Ok(updated)
    }

    async fn delete_board(&self, id: &str) -> CustomResult<Board> {
        self.cache_delete_key(id).await?;
        let board = self.db.delete_board(id).await?;
        self.pub_board_deleted(id)?;
        Ok(board)
    }

    async fn subscribe_on_board_updates(&self, board_id: &str) -> CustomResult<EventMsgReceiver> {
        let _ = self.read_board(board_id).await?;
        let (tx, rx) = mpsc::channel::<EventMsgResult>(100);
        tx.send(Ok("Connected;\n".into()))
            .await
            .expect("Can't send message to board events stream");

        let mut pub_sub = self
            .redis_client
            .get_async_connection()
            .await?
            .into_pubsub();

        pub_sub
            .subscribe(&Self::pub_sub_channel_name(board_id))
            .await?;

        tokio::spawn(async move {
            while let Some(event) = pub_sub.on_message().next().await {
                let payload = event.get_payload().expect("Can't get message payload");
                let payload: String = FromRedisValue::from_redis_value(&payload)
                    .expect("Can't convert event message from redis value");
                let msg = Bytes::from(format!("Board event: {};\n", payload));
                tx.send(Ok(msg)).await.expect("Events stream destroyed");
            }
        });

        Ok(rx)
    }
}

#[async_trait::async_trait]
impl<T: TasksDatabase + Clone> TasksDatabase for Cached<T> {
    async fn create_task(&self, board_id: &str, task: Task) -> CustomResult<Task> {
        let task = self.db.create_task(board_id, task).await?;
        self.pub_board_updated(board_id)?;
        Ok(task)
    }

    async fn read_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
        self.db.read_tasks(board_id).await
    }

    async fn read_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        Ok(match self.cache_get(board_id, task_id).await {
            Ok(t) => {
                log::trace!("Read task #{} from cache", task_id);
                t
            }
            _ => {
                log::trace!("Read task #{} from database", task_id);
                let task = self.db.read_task(board_id, task_id).await?;
                self.cache_set(board_id, task_id, &task).await?;
                task
            }
        })
    }

    async fn update_task(&self, board_id: &str, task_id: &str, task: Task) -> CustomResult<Task> {
        let updated = self.db.update_task(task_id, board_id, task).await?;
        self.cache_set(board_id, task_id, &updated).await?;
        self.pub_board_updated(board_id)?;
        Ok(updated)
    }

    async fn delete_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        let deleted = self.db.delete_task(board_id, task_id).await?;
        self.cache_delete_field(board_id, task_id).await?;
        self.pub_board_updated(board_id)?;
        Ok(deleted)
    }
}
