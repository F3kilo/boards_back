use crate::errors::{CustomError, CustomResult};
use crate::models::{Board, Task, TaskStage};
use mongodb::bson::oid::ObjectId;

pub struct Boards {}

impl Boards {
    pub fn new() -> Self {
        Self {}
    }

    pub fn list_boards(&self) -> CustomResult<Vec<Board>> {
        // todo
        Ok(vec![Board {
            id: Some(ObjectId::new()),
            name: "моя доска".into(),
            description: "описание моей доски".into(),
        }])
    }

    pub fn board(&self, id: &str) -> CustomResult<Board> {
        // todo
        if id == "2" {
            Ok(Board {
                id: None,
                name: "моя доска".into(),
                description: "описание моей доски".into(),
            })
        } else {
            Err(CustomError::NotFound(format!("board with id = {}", 2)))
        }
    }

    pub fn create_board(&self, board_data: Board) -> CustomResult<Board> {
        // todo
        Ok(Board {
            id: Some(ObjectId::new()),
            ..board_data
        })
    }

    pub fn update_board(&self, id: &str, board_data: Board) -> CustomResult<Board> {
        // todo
        Ok(Board {
            id: Some(ObjectId::new()),
            ..board_data
        })
    }

    pub fn delete_board(&self, id: &str) -> CustomResult<Board> {
        // todo
        Ok(Board {
            id: Some(ObjectId::new()),
            name: "моя доска".into(),
            description: "описание моей доски".into(),
        })
    }

    pub fn list_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
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

    pub fn get_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        // todo
        Ok(Task {
            id: Some(ObjectId::new()),
            name: "моя доска".into(),
            description: "описание моей доски".into(),
            stage: TaskStage::Complete,
        })
    }

    pub fn create_task(&self, board_id: &str, task: Task) -> CustomResult<Task> {
        // todo
        Ok(Task {
            id: Some(ObjectId::new()),
            ..task
        })
    }

    pub fn update_task(&self, board_id: &str, task_id: &str, task: Task) -> CustomResult<Task> {
        // todo
        Ok(Task {
            id: Some(ObjectId::new()),
            ..task
        })
    }

    pub fn delete_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        // todo
        Ok(Task {
            id: Some(ObjectId::new()),
            name: "моя доска".into(),
            description: "описание моей доски".into(),
            stage: TaskStage::Complete,
        })
    }
}
