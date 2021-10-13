use crate::db::{BoardsDatabase, EventMsgReceiver, TasksDatabase};
use crate::errors::{CustomError, CustomResult};
use crate::models::{Board, Task};
use mongodb::{
    bson::{doc, oid::ObjectId, ser, Bson},
    Client, Collection,
};
use serde::de::DeserializeOwned;
use std::str::FromStr;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct Mongo {
    client: Client,
}

impl Mongo {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn get_boards_collection(&self) -> Collection<Board> {
        self.client.database("boards_back").collection("boards")
    }

    pub fn get_tasks_collection(&self) -> Collection<Task> {
        self.client.database("boards_back").collection("tasks")
    }

    async fn get_by_id<T>(&self, collection: Collection<T>, id: Bson) -> CustomResult<T>
    where
        T: DeserializeOwned + Unpin + Send + Sync,
    {
        let query = doc! { "_id": &id };
        let board = collection.find_one(query, None).await?;
        board.ok_or_else(|| CustomError::NotFound(format!("board with id: {}", id)))
    }
}

#[async_trait::async_trait]
impl BoardsDatabase for Mongo {
    async fn create_board(&self, board: Board) -> CustomResult<Board> {
        let collection = self.get_boards_collection();
        let insert_result = collection.insert_one(board, None).await?;
        self.get_by_id(collection, insert_result.inserted_id).await
    }

    async fn read_boards(&self) -> CustomResult<Vec<Board>> {
        let collection = self.get_boards_collection();
        let query = doc! {};
        let mut boards = collection.find(query, None).await?;

        let mut boards_vec = Vec::new();
        while let Some(board) = boards.next().await {
            boards_vec.push(board?);
        }

        Ok(boards_vec)
    }

    async fn read_board(&self, id: &str) -> CustomResult<Board> {
        let collection = self.get_boards_collection();
        let obj_id = ObjectId::from_str(id)?;
        self.get_by_id(collection, obj_id.into()).await
    }

    async fn update_board(&self, id: &str, board: Board) -> CustomResult<Board> {
        let obj_id = ObjectId::from_str(id)?;
        let collection = self.get_boards_collection();
        let query = doc! { "_id": &obj_id };
        let update = doc! { "$set": ser::to_bson(&board)? };
        collection.update_one(query, update, None).await?;
        Ok(board)
    }

    async fn delete_board(&self, id: &str) -> CustomResult<Board> {
        let obj_id = ObjectId::from_str(id)?;
        let collection = self.get_boards_collection();
        let query = doc! { "_id": &obj_id };
        let board = collection.find_one_and_delete(query, None).await?;

        // Delete all board's tasks.
        let tasks_collection = self.get_tasks_collection();
        let query = doc! { "board_id": &obj_id };
        tasks_collection.delete_many(query, None).await?;

        board.ok_or_else(|| CustomError::NotFound(format!("board with id: {}", id)))
    }

    async fn subscribe_on_board_updates(&self, _board_id: &str) -> CustomResult<EventMsgReceiver> {
        Err(CustomError::InternalError(
            "Subscription isn't implemented for mongo".into(),
        ))
    }
}

#[async_trait::async_trait]
impl TasksDatabase for Mongo {
    async fn create_task(&self, board_id: &str, mut task: Task) -> CustomResult<Task> {
        let collection = self.get_tasks_collection();
        task.board_id = Some(ObjectId::from_str(board_id)?);
        let insert_result = collection.insert_one(task, None).await?;
        self.get_by_id(collection, insert_result.inserted_id).await
    }

    async fn read_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
        let collection = self.get_tasks_collection();
        let board_obj_id = ObjectId::from_str(board_id)?;
        let query = doc! { "board_id": &board_obj_id };
        let mut tasks = collection.find(query, None).await?;

        let mut tasks_vec = Vec::new();
        while let Some(task) = tasks.next().await {
            tasks_vec.push(task?);
        }
        Ok(tasks_vec)
    }

    async fn read_task(&self, _: &str, id: &str) -> CustomResult<Task> {
        let collection = self.get_tasks_collection();
        let obj_id = ObjectId::from_str(id)?;
        self.get_by_id(collection, obj_id.into()).await
    }

    async fn update_task(&self, board_id: &str, task_id: &str, mut task: Task) -> CustomResult<Task> {
        let task_obj_id = ObjectId::from_str(task_id)?;
        task.board_id = Some(ObjectId::from_str(board_id)?);
        let collection = self.get_tasks_collection();
        let query = doc! { "_id": &task_obj_id };
        let update = doc! { "$set": ser::to_bson(&task)? };
        log::trace!("QUERY: {}", query);
        log::trace!("UPDATE: {}", update);
        collection.update_one(query, update, None).await?;
        Ok(task)
    }

    async fn delete_task(&self, _: &str, id: &str) -> CustomResult<Task> {
        let collection = self.get_tasks_collection();
        let obj_id = ObjectId::from_str(id)?;
        let query = doc! { "_id": &obj_id };
        let task = collection.find_one_and_delete(query, None).await?;
        task.ok_or_else(|| CustomError::NotFound(format!("board with id: {}", id)))
    }
}
