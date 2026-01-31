use sqlx::PgPool;

use crate::models::{Board, BoardWithStats};

pub struct BoardService;

impl BoardService {
    pub async fn list(pool: &PgPool) -> Result<Vec<BoardWithStats>, sqlx::Error> {
        let boards = sqlx::query_as::<_, Board>(
            "SELECT * FROM boards ORDER BY id"
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(boards.len());
        for board in boards {
            let thread_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM threads WHERE board_id = $1"
            )
            .bind(board.id)
            .fetch_one(pool)
            .await?;

            result.push(BoardWithStats {
                board,
                thread_count: thread_count.0,
            });
        }

        Ok(result)
    }

    pub async fn get_by_slug(
        pool: &PgPool,
        slug: &str,
    ) -> Result<Option<Board>, sqlx::Error> {
        sqlx::query_as::<_, Board>(
            "SELECT * FROM boards WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await
    }
}
