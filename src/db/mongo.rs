use crate::db::Database;
use crate::errors::{CustomError, CustomResult};
use crate::models::{Board, BoardData, Task, TaskData};
use actix_web::web::Bytes;
use mongodb::{
    bson::{doc, oid::ObjectId, ser},
    Client, Collection,
};
use std::str::FromStr;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_stream::StreamExt;

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
}

#[async_trait::async_trait]
impl Database for Mongo {
    async fn get_boards(&self) -> CustomResult<Vec<Board>> {
        let collection = self.get_boards_collection();
        let query = doc! {};
        let mut boards = collection.find(query, None).await?;

        let mut boards_vec = Vec::new();
        while let Some(board) = boards.next().await {
            boards_vec.push(board?);
        }

        Ok(boards_vec)
    }

    async fn create_board(&self, data: BoardData) -> CustomResult<Board> {
        let collection = self.get_boards_collection();
        let board: Board = data.into();
        let insert_result = collection.insert_one(board, None).await?;
        log::warn!("id: {}", insert_result.inserted_id);
        let id = insert_result
            .inserted_id
            .as_object_id()
            .unwrap()
            .to_string();
        self.get_board(&id).await
    }

    async fn get_board(&self, id: &str) -> CustomResult<Board> {
        let obj_id = ObjectId::from_str(id)?;
        let collection = self.get_boards_collection();
        let query = doc! { "_id": &obj_id };
        let board = collection.find_one(query, None).await?;
        board.ok_or_else(|| CustomError::NotFound(format!("board with id: {}", id)))
    }

    async fn put_board(&self, id: &str, data: BoardData) -> CustomResult<Board> {
        let obj_id = ObjectId::from_str(id)?;
        let collection = self.get_boards_collection();
        let query = doc! { "_id": &obj_id };
        let update = doc! { "$set": ser::to_bson(&data)? };
        collection.update_one(query, update, None).await?;
        self.get_board(id).await
    }

    async fn delete_board(&self, id: &str) -> CustomResult<Board> {
        let obj_id = ObjectId::from_str(id)?;
        let collection = self.get_boards_collection();
        let query = doc! { "_id": &obj_id };
        let board = collection.find_one_and_delete(query, None).await?;
        board.ok_or_else(|| CustomError::NotFound(format!("board with id: {}", id)))
    }

    async fn get_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
        let board = self.get_board(board_id).await;
        board.map(|b| b.tasks)
    }

    async fn create_task(&self, board_id: &str, data: TaskData) -> CustomResult<Task> {
        let board_obj_id = ObjectId::from_str(board_id)?;
        let collection = self.get_boards_collection();
        let task: Task = data.into();
        let query = doc! { "_id": &board_obj_id };
        let update = doc! { "$addToSet": { "tasks": ser::to_bson(&task)? } };
        let updated = collection.update_one(query, update, None).await?;
        log::warn!("UPDATE_RESULT: {:?}", updated);
        Ok(task)
        // todo! self.get_task(id).await
    }

    async fn get_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        todo!()
        // let board_obj_id = ObjectId::from_str(board_id)?;
        // let task_obj_id = ObjectId::from_str(board_id)?;
        // let collection = self.get_boards_collection();
        // let query = doc! { "_id": &board_obj_id, "tasks._id": &task_obj_id };
        // let board = collection.find_one(query, ).await?;
        // board.ok_or_else(|| CustomError::NotFound(format!("task #{} in board#{}", task_id, board_id))).map(|b| b.)
    }

    async fn put_task(&self, board_id: &str, task_id: &str, data: TaskData) -> CustomResult<Task> {
        todo!()
    }

    async fn delete_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        todo!()
    }

    async fn subscribe_on_board_updates(
        &self,
        board_id: &str,
        tx: Sender<CustomResult<Bytes>>,
    ) -> Receiver<CustomResult<Bytes>> {
        todo!()
    }
}
