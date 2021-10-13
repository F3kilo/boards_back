use crate::db::TasksDatabase;
use crate::errors::CustomResult;
use crate::models::Task;

pub struct Tasks {
    db: Box<dyn TasksDatabase>,
}

impl Tasks {
    pub fn new(db: Box<dyn TasksDatabase>) -> Self {
        Self { db }
    }

    pub async fn create_task(&self, board_id: &str, task: Task) -> CustomResult<Task> {
        self.db.create_task(board_id, task).await
    }

    pub async fn read_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        self.db.read_task(board_id, task_id).await
    }

    pub async fn read_board_tasks(&self, board_id: &str) -> CustomResult<Vec<Task>> {
        self.db.read_tasks(board_id).await
    }

    pub async fn update_task(&self, board_id: &str, task_id: &str, task: Task) -> CustomResult<Task> {
        self.db.update_task(board_id, task_id, task).await
    }

    pub async fn delete_task(&self, board_id: &str, task_id: &str) -> CustomResult<Task> {
        self.db.delete_task(board_id, task_id).await
    }
}
