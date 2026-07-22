/*
backend/crates/api/src/handlers/page.rs
ページ系のユースケース
*/

// 外部クレート
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 内部ライブラリ
use content::PageType;
use identity::PageId;

// 自クレート
use crate::error::ApiAppError;
use crate::extractor::{AdminUser, AuthUser};
use crate::response::ApiResponse;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListPagesQuery {
  pub r#type: PageType,
  pub page: Option<u32>,
  pub per_page: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageListItemResponse {
  pub id: PageId,
  pub title: String,
  pub is_published: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub display_order: Option<i32>,
  pub author_username: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageDetailResponse {
  pub id: PageId,
  pub r#type: PageType,
  pub title: String,
  pub content_markdown: String,
  pub display_order: Option<i32>,
  pub is_published: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub author_username: String,
}

/// created_byのusernameを解決する(N+1、他ハンドラと同じ方針で許容)
async fn resolve_author_username(
  state: &AppState,
  user_id: identity::UserId,
) -> Result<String, ApiAppError> {
  Ok(
    state
      .user_repository
      .find_by_id(user_id)
      .await
      .map_err(app::AppError::from)?
      .map(|u| u.username)
      .unwrap_or_else(|| "unknown".to_string()),
  )
}

fn display_order_of(page: &content::Page) -> Option<i32> {
  match page.kind {
    content::PageKind::Static { display_order } => Some(display_order),
    content::PageKind::Blog => None,
  }
}

pub async fn list_pages(
  State(state): State<AppState>,
  _auth_user: AuthUser,
  Query(query): Query<ListPagesQuery>,
) -> Result<Json<ApiResponse<Vec<PageListItemResponse>>>, ApiAppError> {
  let pages = app::list_pages(state.page_repository.as_ref(), query.r#type).await?;

  // blogのみページング(02-types/api.md 11章)、staticは全件(Phase 8のフィルタ一覧と同じ簡易ページング方針)
  let pages = if query.r#type == PageType::Blog {
    if let (Some(page_no), Some(per_page)) = (query.page, query.per_page) {
      let start = ((page_no.saturating_sub(1)) * per_page) as usize;
      pages
        .into_iter()
        .skip(start)
        .take(per_page as usize)
        .collect()
    } else {
      pages
    }
  } else {
    pages
  };

  let mut items = Vec::with_capacity(pages.len());

  for p in pages {
    let author_username = resolve_author_username(&state, p.created_by).await?;
    let display_order = display_order_of(&p);

    items.push(PageListItemResponse {
      id: p.id,
      title: p.title,
      is_published: p.is_published,
      created_at: p.created_at,
      updated_at: p.updated_at,
      display_order: display_order,
      author_username,
    });
  }

  Ok(Json(ApiResponse::ok(items)))
}

async fn to_detail_response(
  state: &AppState,
  page: content::Page,
) -> Result<PageDetailResponse, ApiAppError> {
  let author_username = resolve_author_username(state, page.created_by).await?;
  Ok(PageDetailResponse {
    id: page.id,
    r#type: page.kind.page_type(),
    display_order: display_order_of(&page),
    title: page.title,
    content_markdown: page.content_markdown,
    is_published: page.is_published,
    created_at: page.created_at,
    updated_at: page.updated_at,
    author_username,
  })
}

pub async fn get_page(
  State(state): State<AppState>,
  _auth_user: AuthUser,
  Path(page_id): Path<PageId>,
) -> Result<Json<ApiResponse<PageDetailResponse>>, ApiAppError> {
  let page = app::get_page(state.page_repository.as_ref(), page_id).await?;
  Ok(Json(ApiResponse::ok(
    to_detail_response(&state, page).await?,
  )))
}

#[derive(Deserialize)]
pub struct CreatePageRequest {
  pub r#type: PageType,
  pub title: String,
  pub content_markdown: String,
  pub display_order: Option<i32>,
  pub is_published: bool,
}

pub async fn create_page(
  State(state): State<AppState>,
  admin: AdminUser,
  Json(body): Json<CreatePageRequest>,
) -> Result<Json<ApiResponse<PageDetailResponse>>, ApiAppError> {
  let page = app::create_page(
    state.page_repository.as_ref(),
    admin.0.user_id,
    body.r#type,
    body.title,
    body.content_markdown,
    body.display_order,
    body.is_published,
  )
  .await?;

  Ok(Json(ApiResponse::ok(
    to_detail_response(&state, page).await?,
  )))
}

#[derive(Deserialize)]
pub struct UpdatePageRequest {
  pub title: String,
  pub content_markdown: String,
  pub is_published: bool,
}

pub async fn update_page(
  State(state): State<AppState>,
  _admin: AdminUser,
  Path(page_id): Path<PageId>,
  Json(body): Json<UpdatePageRequest>,
) -> Result<Json<ApiResponse<PageDetailResponse>>, ApiAppError> {
  let page = app::update_page(
    state.page_repository.as_ref(),
    page_id,
    body.title,
    body.content_markdown,
    body.is_published,
  )
  .await?;
  Ok(Json(ApiResponse::ok(
    to_detail_response(&state, page).await?,
  )))
}

pub async fn delete_page(
  State(state): State<AppState>,
  _admin: AdminUser,
  Path(page_id): Path<PageId>,
) -> Result<Json<ApiResponse<()>>, ApiAppError> {
  app::delete_page(state.page_repository.as_ref(), page_id).await?;
  Ok(Json(ApiResponse::ok(())))
}

#[derive(Deserialize)]
pub struct UpdatePageOrderRequest {
  pub display_order: i32,
}

pub async fn update_page_order(
  State(state): State<AppState>,
  _admin: AdminUser,
  Path(page_id): Path<PageId>,
  Json(body): Json<UpdatePageOrderRequest>,
) -> Result<Json<ApiResponse<PageDetailResponse>>, ApiAppError> {
  let page =
    app::update_page_order(state.page_repository.as_ref(), page_id, body.display_order).await?;
  Ok(Json(ApiResponse::ok(
    to_detail_response(&state, page).await?,
  )))
}
