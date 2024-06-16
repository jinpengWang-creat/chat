mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod router;
mod server;
mod state;
mod utils;
pub use models::User;
pub use server::run;

#[cfg(test)]
mod test_util {
    use sqlx::Executor;
    use sqlx::PgPool;
    use sqlx_db_tester::TestPg;
    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:postgres@localhost:5432".to_string(),
        };

        let test_pg = TestPg::new(url, std::path::Path::new("../migrations"));
        let pool = test_pg.get_pool().await;

        // execute the sql with transaction
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut tx = pool.begin().await.expect("begin failed");
        for q in sql {
            if q.trim().is_empty() {
                continue;
            }
            tx.execute(q).await.expect("execute failed");
        }
        tx.commit().await.expect("commit failed");
        (test_pg, pool)
    }
}
