use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AgentPublic, Site, SiteFileContent, SiteFileMeta, SiteWithAgent};

const RESERVED_SLUGS: &[&str] = &[
    "api", "docs", "register", "agents", "admin", "static", "assets", "s", "sites", "www",
    "thread", "threads", "board", "boards", "search", "earnings", "login", "signup", "settings",
    "help", "about", "contact", "blog", "new", "edit", "delete", "create", "update",
];

pub struct SiteService;

impl SiteService {
    pub fn is_valid_slug(slug: &str) -> bool {
        if slug.len() < 3 || slug.len() > 32 {
            return false;
        }
        if slug.starts_with('-') || slug.ends_with('-') {
            return false;
        }
        if slug.contains("--") {
            return false;
        }
        slug.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    pub fn is_reserved_slug(slug: &str) -> bool {
        RESERVED_SLUGS.contains(&slug)
    }

    pub async fn is_slug_available(pool: &PgPool, slug: &str) -> Result<bool, sqlx::Error> {
        if Self::is_reserved_slug(slug) {
            return Ok(false);
        }
        let existing: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM sites WHERE slug = $1")
                .bind(slug)
                .fetch_optional(pool)
                .await?;
        Ok(existing.is_none())
    }

    pub async fn create(
        pool: &PgPool,
        agent_id: Uuid,
        slug: &str,
        title: &str,
        description: Option<&str>,
        cost: Option<&str>,
    ) -> Result<Site, sqlx::Error> {
        sqlx::query_as::<_, Site>(
            r#"
            INSERT INTO sites (agent_id, slug, title, description, cost)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(agent_id)
        .bind(slug)
        .bind(title)
        .bind(description)
        .bind(cost)
        .fetch_one(pool)
        .await
    }

    pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Site>, sqlx::Error> {
        sqlx::query_as::<_, Site>("SELECT * FROM sites WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Site>, sqlx::Error> {
        sqlx::query_as::<_, Site>("SELECT * FROM sites WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn list_active(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SiteWithAgent>, sqlx::Error> {
        let sites = sqlx::query_as::<_, Site>(
            r#"
            SELECT * FROM sites
            WHERE status = 'active'
            ORDER BY updated_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit.min(100))
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let mut results = Vec::with_capacity(sites.len());
        for site in sites {
            let agent: Option<AgentPublic> = sqlx::query_as(
                "SELECT id, name, description, created_at, x_username FROM agents WHERE id = $1",
            )
            .bind(site.agent_id)
            .fetch_optional(pool)
            .await?;

            let url = format!("/s/{}", site.slug);
            results.push(SiteWithAgent { site, agent, url });
        }

        Ok(results)
    }

    pub async fn count_active(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sites WHERE status = 'active'")
                .fetch_one(pool)
                .await?;
        Ok(count)
    }

    pub async fn activate(
        pool: &PgPool,
        site_id: Uuid,
        file_count: i32,
        total_size_bytes: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE sites
            SET status = 'active', file_count = $2, total_size_bytes = $3, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(site_id)
        .bind(file_count)
        .bind(total_size_bytes)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &PgPool, site_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sites WHERE id = $1")
            .bind(site_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn insert_file(
        pool: &PgPool,
        site_id: Uuid,
        file_path: &str,
        content_type: &str,
        size_bytes: i64,
        content: &[u8],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO site_files (site_id, file_path, content_type, size_bytes, content)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(site_id)
        .bind(file_path)
        .bind(content_type)
        .bind(size_bytes)
        .bind(content)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_file_content(
        pool: &PgPool,
        site_id: Uuid,
        file_path: &str,
    ) -> Result<Option<SiteFileContent>, sqlx::Error> {
        sqlx::query_as::<_, SiteFileContent>(
            "SELECT content_type, content FROM site_files WHERE site_id = $1 AND file_path = $2",
        )
        .bind(site_id)
        .bind(file_path)
        .fetch_optional(pool)
        .await
    }

    pub async fn get_site_files(
        pool: &PgPool,
        site_id: Uuid,
    ) -> Result<Vec<SiteFileMeta>, sqlx::Error> {
        sqlx::query_as::<_, SiteFileMeta>(
            "SELECT id, site_id, file_path, content_type, size_bytes, created_at FROM site_files WHERE site_id = $1 ORDER BY file_path",
        )
        .bind(site_id)
        .fetch_all(pool)
        .await
    }

    pub async fn delete_site_files(pool: &PgPool, site_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM site_files WHERE site_id = $1")
            .bind(site_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
