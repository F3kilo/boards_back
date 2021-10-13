use crate::boards::Boards;
use crate::errors::{CustomError, CustomResult};
use crate::models::{Board, Task};
use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpResponse};
use std::sync::Arc;
use crate::tasks::Tasks;

#[actix_web::get("/boards")]
pub async fn read_boards(boards: web::Data<Arc<Boards>>) -> CustomResult<HttpResponse> {
    let boards = boards.read_boards().await?;
    Ok(HttpResponse::Ok().json(boards))
}

#[actix_web::post("/boards")]
pub async fn create_board(
    board_data: web::Json<Board>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let board_data = board_data.into_inner();
    let board = boards.create_board(board_data).await?;
    Ok(HttpResponse::Ok().json(board))
}

#[actix_web::get("/boards/{board_id}")]
pub async fn read_board(
    board_id: web::Path<String>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let id = board_id.into_inner();
    let board = boards.read_board(&id).await?;
    Ok(HttpResponse::Ok().json(board))
}

#[actix_web::put("/boards/{board_id}")]
pub async fn update_board(
    board_id: web::Path<String>,
    board: web::Json<Board>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let id = board_id.into_inner();
    let board = board.into_inner();
    let board = boards.update_board(&id, board).await?;
    Ok(HttpResponse::Ok().json(board))
}

#[actix_web::delete("/boards/{board_id}")]
pub async fn delete_board(
    board_id: web::Path<String>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let id = board_id.into_inner();
    let board = boards.delete_board(&id).await?;
    Ok(HttpResponse::Ok().json(board))
}

#[actix_web::get("/tasks")]
pub async fn read_tasks(tasks: web::Data<Arc<Tasks>>) -> CustomResult<HttpResponse> {
    let tasks = tasks.read_tasks().await?;
    Ok(HttpResponse::Ok().json(tasks))
}

#[actix_web::get("/boards/{board_id}/tasks")]
pub async fn read_board_tasks(
    board_id: web::Path<String>,
    tasks: web::Data<Arc<Tasks>>,
) -> CustomResult<HttpResponse> {
    let board_id = board_id.into_inner();
    let tasks = tasks.read_board_tasks(&board_id).await?;
    Ok(HttpResponse::Ok().json(tasks))
}

#[actix_web::post("/tasks")]
pub async fn create_task(
    task: web::Json<Task>,
    tasks: web::Data<Arc<Tasks>>,
) -> CustomResult<HttpResponse> {
    let task = task.into_inner();
    let task = tasks.create_task(task).await?;
    Ok(HttpResponse::Ok().json(task))
}

#[actix_web::get("/tasks/{task_id}")]
pub async fn read_task(
    ids: web::Path<String>,
    tasks: web::Data<Arc<Tasks>>,
) -> CustomResult<HttpResponse> {
    let task_id = ids.into_inner();
    let tasks = tasks.read_task(&task_id).await?;
    Ok(HttpResponse::Ok().json(tasks))
}

#[actix_web::put("/tasks/{task_id}")]
pub async fn update_task(
    task_id: web::Path<String>,
    task: web::Json<Task>,
    tasks: web::Data<Arc<Tasks>>,
) -> Result<HttpResponse, CustomError> {
    let id = task_id.into_inner();
    let task = task.into_inner();
    let task = tasks.update_task(&id, task).await?;
    Ok(HttpResponse::Ok().json(task))
}

#[actix_web::delete("/tasks/{task_id}")]
pub async fn delete_task(
    ids: web::Path<String>,
    tasks: web::Data<Arc<Tasks>>,
) -> Result<HttpResponse, CustomError> {
    let task_id = ids.into_inner();
    let task = tasks.delete_task(&task_id).await?;
    Ok(HttpResponse::Ok().json(task))
}

#[actix_web::get("/boards/{board_id}/updates")]
pub async fn subscribe_board_changes(
    board_id: web::Path<String>,
    boards: web::Data<Arc<Boards>>,
) -> Result<HttpResponse, CustomError> {
    let board_id = board_id.into_inner();
    let updates_stream = boards.subscribe_on_board_changes(&board_id).await?;
    let response_stream = tokio_stream::wrappers::ReceiverStream::new(updates_stream);

    Ok(HttpResponse::build(StatusCode::OK)
        .insert_header(header::ContentType(mime::TEXT_EVENT_STREAM))
        .streaming(response_stream))
}
