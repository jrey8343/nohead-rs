use super::test_request_with_db;
use fake::{Fake as _, Faker};
use nohead_rs_db::{
    DbPool, MIGRATOR,
    entities::{
        session::Session,
        user::{RegisterUser, User, UserCredentials},
    },
};

#[sqlx::test(migrator = "MIGRATOR")]
async fn login_creates_session_on_success(pool: DbPool) {
    test_request_with_db::<_, _>(pool.clone(), |request| async move {
        let user: RegisterUser = Faker.fake();

        User::create(user.clone(), &pool).await.unwrap();

        let response = request
            .post("/auth/login")
            .form(&UserCredentials {
                email: user.email,
                password: user.password,
                next: None,
            })
            .await;

        let session_cookie = response.cookie("id").value().to_string();
        // Cookie is signed so the actual session id can be extracted after the '=' symbol
        let session_cookie = session_cookie.split('=').collect::<Vec<&str>>()[1];

        let session = sqlx::query_as!(
            Session,
            "SELECT * FROM sessions WHERE id = ?1",
            session_cookie
        )
        .fetch_optional(&pool)
        .await
        .unwrap()
        .expect("no session found in the database");

        assert_eq!(
            session_cookie, session.id,
            "session cookie did not match the stored token"
        );
    })
    .await
}
#[sqlx::test(migrator = "MIGRATOR")]
async fn login_does_not_create_session_on_invalid_credentials(pool: DbPool) {
    test_request_with_db::<_, _>(pool.clone(), |request| async move {
        let user: RegisterUser = Faker.fake();

        // 😉User is not created in the database

        let response = request
            .post("/auth/login")
            .form(&UserCredentials {
                email: user.email,
                password: user.password,
                next: None,
            })
            .await;

        let session_cookie = response.maybe_cookie("id");

        assert!(
            session_cookie.is_none(),
            "oops a session cookie was created for a non existent user"
        );
    })
    .await
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn login_redirects_to_login_page_for_invalid_password(pool: DbPool) {
    test_request_with_db::<_, _>(pool.clone(), |request| async move {
        let user: RegisterUser = Faker.fake();

        User::create(user.clone(), &pool)
            .await
            .expect("failed to create user in test db");

        let response = request
            .post("/auth/login")
            .form(&UserCredentials {
                email: user.email.clone(),
                password: "wrongPa$$word".into(),
                next: None,
            })
            .await;

        response.assert_status_see_other();

        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header from response")
            .to_str()
            .unwrap();

        assert_eq!(location, "/auth/login", "redirected to the wrong page");
    })
    .await
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn login_redirects_to_login_page_for_invalid_user(pool: DbPool) {
    test_request_with_db::<_, _>(pool.clone(), |request| async move {
        let user: RegisterUser = Faker.fake();

        User::create(user.clone(), &pool)
            .await
            .expect("failed to create user in test db");

        let response = request
            .post("/auth/login")
            .form(&UserCredentials {
                email: "thisisnotauser@fake.com".into(),
                password: user.password,
                next: None,
            })
            .await;

        response.assert_status_see_other();

        let location = response
            .headers()
            .get("location")
            .expect("unable to get redirect location header from response")
            .to_str()
            .unwrap();

        assert_eq!(location, "/auth/login", "redirected to the wrong page");
    })
    .await
}
