use nohead_rs_db::entities::todo::{Todo, TodoChangeset};
use sqlx::SqlitePool;

use crate::{test_request, test_request_with_db};

#[tokio::test]
async fn index_works() {
    test_request::<_, _>(|request| async move {
        let response = request.get("/todos").await;
        response.assert_status_ok()
    })
    .await;
}

#[sqlx::test]
async fn create_redirects_and_displays_new_todo_on_success(pool: SqlitePool) {
    let test_todo = "testing a create".to_string();

    test_request_with_db::<_, _>(pool, |request| async move {
        let response = request
            .post("/todos")
            .form(&TodoChangeset {
                description: test_todo.clone(),
            })
            .await;

        response.assert_status_see_other();

        // Manually follow the redirection and assert UI reflexts new todo
        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header from response")
            .to_str()
            .unwrap();

        let response = request.get(location).await;

        response.assert_text_contains(test_todo);
    })
    .await
}

#[sqlx::test]
async fn create_persists_todo_in_database(pool: SqlitePool) {
    let test_todo = "testing a create".to_string();

    test_request_with_db::<_, _>(pool.clone(), |request| async move {
        let _response = request
            .post("/todos")
            .form(&TodoChangeset {
                description: test_todo.clone(),
            })
            .await;

        let saved_todo =
            sqlx::query_as!(Todo, "SELECT * FROM todos WHERE description = ?", test_todo)
                .fetch_optional(&pool)
                .await
                .unwrap();

        assert!(saved_todo.is_some())
    })
    .await
}

#[sqlx::test]
async fn create_throws_422_for_invalid_form_input(pool: SqlitePool) {
    test_request_with_db::<_, _>(pool, |request| async move {
        let response = request
            .post("/todos")
            .form(&TodoChangeset {
                description: "".to_string(),
            })
            .await;

        response.assert_status_unprocessable_entity();
    })
    .await
}

#[sqlx::test]
async fn delete_works(pool: SqlitePool) {
    let todo = Todo {
        id: 1,
        description: "testing a delete".to_string(),
    };

    test_request_with_db::<_, _>(pool.clone(), |request| async move {
        let response = request.delete(&format!("/todos/{}", todo.id)).await;
        response.assert_status_see_other();

        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header")
            .to_str()
            .unwrap();

        let response = request.get(location).await;

        let deleted_todo = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE id = ?", todo.id)
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(
            deleted_todo.is_none(),
            "the todo should no longer exist in the database"
        );

        assert!(
            !response
                .text()
                .contains(&format!("{}/{}", location, todo.id)),
            "the todo should no longer exist in the UI"
        );
    })
    .await
}

#[sqlx::test]
async fn update_works(pool: SqlitePool) {
    let todo = Todo {
        id: 1,
        description: "testing an update".to_string(),
    };

    test_request_with_db::<_, _>(pool, |request| async move {
        let response = request
            .put(&format!("/todos/{}", todo.id))
            .form(&TodoChangeset {
                description: "testing an update".to_string(),
            })
            .await;

        response.assert_status_ok();
    })
    .await
}
