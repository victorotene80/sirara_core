use crate::domain::repository::RepoError;

pub fn map_sqlx(e: sqlx::Error) -> RepoError {
    use sqlx::Error;

    match e {
        Error::RowNotFound => RepoError::NotFound {
            entity: "row".to_string(),
        },

        Error::PoolTimedOut | Error::PoolClosed => RepoError::Transient {
            message: e.to_string(),
        },

        Error::Database(db_err) => {
            let code_owned = db_err.code().map(|c| c.to_string());
            let code = code_owned.as_deref().unwrap_or("");

            match code {
                "23505" => RepoError::Conflict {
                    message: db_err.to_string(),
                },
                "23503" | "23514" | "23502" => RepoError::Integrity {
                    message: db_err.to_string(),
                },
                "40001" | "40P01" | "55P03" => RepoError::Transient {
                    message: db_err.to_string(),
                },
                _ => RepoError::Unexpected {
                    message: db_err.to_string(),
                },
            }
        }

        other => RepoError::Unexpected {
            message: other.to_string(),
        },
    }
}
