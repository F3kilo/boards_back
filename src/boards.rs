use crate::errors::{CustomError, CustomResult};
use crate::models::{Board, BoardData, Task, TaskData, TaskStage};
use actix_web::web::Bytes;
use mongodb::bson::oid::ObjectId;
use tokio::sync::mpsc::{self, Receiver};
use tokio::time::Duration;

pub struct Boards {}

impl Boards {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn list_boards(&self) -> CustomResult<Vec<Board>> {
        // todo
        Ok(vec![Board {
            id: Some(ObjectId::new()),
            name: "моя доска".into(),
            description: "описание моей доски".into(),
            tasks: self.list_tasks("").await.unwrap(),
        }])
    }

    pub async fn board(&self, id: &str) -> CustomResult<Board> {
        // todo
        if id == "2" {
            Ok(Board {
                id: None,
                name: "моя доска".into(),
                description: "описание моей доски".into(),
                tasks: self.list_tasks("").await.unwrap(),
            })
        } else {
            Err(CustomError::NotFound(format!("board with id = {}", 2)))
        }
    }

    pub async fn create_board(&self, board_data: BoardData) -> CustomResult<Board> {
        // todo
        Ok(board_data.into())
    }

    pub async fn update_board(&self, id: &str, board_data: BoardData) -> CustomResult<Board> {
        // todo
        Ok(board_data.into())
    }

    pub async fn delete_board(&self, id: &str) -> CustomResult<Board> {
        // todo
        Ok(Board {
            id: Some(ObjectId::new()),
            name: "моя доска".into(),
            description: "описание моей доски".into(),
            tasks: self.list_tasks("").await.unwrap(),
        })
    }

    pub async fn list_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
        // todo
        Ok(vec![
            Task {
                id: Some(ObjectId::new()),
                name: "моя доска".into(),
                description: "описание моей доски".into(),
                stage: TaskStage::Backlog,
            },
            Task {
                id: Some(ObjectId::new()),
                name: "моя доска".into(),
                description: "описание моей доски".into(),
                stage: TaskStage::Complete,
            },
        ])
    }

    pub async fn get_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        // todo
        Ok(Task {
            id: Some(ObjectId::new()),
            name: "моя доска".into(),
            description: "описание моей доски".into(),
            stage: TaskStage::Complete,
        })
    }

    pub async fn create_task(&self, board_id: &str, task_data: TaskData) -> CustomResult<Task> {
        // todo
        Ok(task_data.into())
    }

    pub async fn update_task(
        &self,
        board_id: &str,
        task_id: &str,
        task_data: TaskData,
    ) -> CustomResult<Task> {
        // todo
        Ok(task_data.into())
    }

    pub async fn delete_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        // todo
        Ok(Task {
            id: Some(ObjectId::new()),
            name: "моя доска".into(),
            description: "описание моей доски".into(),
            stage: TaskStage::Complete,
        })
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
