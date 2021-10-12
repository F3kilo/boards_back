use crate::boards::Boards;
use crate::errors::{CustomError, CustomResult};
use crate::models::{BoardData, TaskData};
use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpResponse};
use std::sync::Arc;

#[actix_web::get("/boards")]
pub async fn get_boards(boards: web::Data<Arc<Boards>>) -> CustomResult<HttpResponse> {
    let boards = boards.list_boards().await?;
    Ok(HttpResponse::Ok().json(boards))
}

#[actix_web::post("/boards")]
pub async fn post_board(
    board_data: web::Json<BoardData>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let board_data = board_data.into_inner();
    let board = boards.create_board(board_data).await?;
    Ok(HttpResponse::Ok().json(board))
}

#[actix_web::get("/boards/{board_id}")]
pub async fn get_board(
    board_id: web::Path<String>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let id = board_id.into_inner();
    let board = boards.board(&id).await?;
    Ok(HttpResponse::Ok().json(board))
}

#[actix_web::put("/boards/{board_id}")]
pub async fn put_board(
    board_id: web::Path<String>,
    board_data: web::Json<BoardData>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let id = board_id.into_inner();
    let board = boards.put_board(&id, board_data.into_inner()).await?;
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

#[actix_web::get("/boards/{board_id}/tasks")]
pub async fn get_tasks(
    board_id: web::Path<String>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let id = board_id.into_inner();
    let tasks = boards.list_tasks(&id).await?;
    Ok(HttpResponse::Ok().json(tasks))
}

#[actix_web::post("/boards/{board_id}/tasks")]
pub async fn post_task(
    board_id: web::Path<String>,
    task_data: web::Json<TaskData>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let board_id = board_id.into_inner();
    let task_data = task_data.into_inner();
    let task = boards.create_task(&board_id, task_data).await?;
    Ok(HttpResponse::Ok().json(task))
}

#[actix_web::get("/boards/{board_id}/tasks/{task_id}")]
pub async fn get_task(
    ids: web::Path<(String, String)>,
    boards: web::Data<Arc<Boards>>,
) -> CustomResult<HttpResponse> {
    let (board_id, task_id) = ids.into_inner();
    let tasks = boards.get_task(&board_id, &task_id).await?;
    Ok(HttpResponse::Ok().json(tasks))
}

#[actix_web::put("/boards/{board_id}/tasks/{task_id}")]
pub async fn put_task(
    ids: web::Path<(String, String)>,
    task_data: web::Json<TaskData>,
    boards: web::Data<Arc<Boards>>,
) -> Result<HttpResponse, CustomError> {
    let (board_id, task_id) = ids.into_inner();
    let task_data = task_data.into_inner();
    let task = boards.update_task(&board_id, &task_id, task_data).await?;
    Ok(HttpResponse::Ok().json(task))
}

#[actix_web::delete("/boards/{board_id}/tasks/{task_id}")]
pub async fn delete_task(
    ids: web::Path<(String, String)>,
    boards: web::Data<Arc<Boards>>,
) -> Result<HttpResponse, CustomError> {
    let (board_id, task_id) = ids.into_inner();
    let task = boards.delete_task(&board_id, &task_id).await?;
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
