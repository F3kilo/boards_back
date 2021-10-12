use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: String,
    pub tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardData {
    pub name: String,
    pub description: String,
}

impl From<BoardData> for Board {
    fn from(bd: BoardData) -> Self {
        Board {
            id: None,
            name: bd.name,
            description: bd.description,
            tasks: Vec::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: String,
    pub stage: TaskStage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskData {
    pub name: String,
    pub description: String,
    pub stage: TaskStage,
}

impl From<TaskData> for Task {
    fn from(td: TaskData) -> Self {
        Task {
            id: None,
            name: td.name,
            description: td.description,
            stage: td.stage,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskStage {
    Backlog,
    InProgress,
    Complete,
}
