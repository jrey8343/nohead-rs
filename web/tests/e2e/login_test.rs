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

        // FIX: Unable to lookup session in the database as it is hashed

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
//
// #[sqlx::test]
// async fn login_throws_400_bad_request_for_invalid_password(pool: SqlitePool) {
//     test_request_with_db::<_, _>(pool.clone(), |request| async move {
//         let mut user = RegisterUserForm {
//             username: "fuddyduddy".to_string(),
//             password: "assAssASSword".to_string(),
//             confirm_password: "assAssASSword".to_string(),
//         };
//
//         user.password = generate_password_hash(&user.password).unwrap();
//
//         User::create(user.clone(), &pool)
//             .await
//             .expect("failed to create user in test db");
//
//         let response = request
//             .post("/auth/login")
//             .form(&LoginUserForm {
//                 username: user.username.clone(),
//                 password: "wrongPa$$word".into(),
//             })
//             .await;
//
//         response.assert_status_bad_request();
//     })
//     .await
// }
// #[tokio::test]
// async fn login_throws_404_not_found_if_user_does_not_exist() {
//     test_request::<_, _>(|request| async move {
//         let response = request
//             .post("/auth/login")
//             .form(&LoginUserForm {
//                 username: "Idonotexisssssst".into(),
//                 password: "a$$$$$worrrrrdd".into(),
//             })
//             .await;
//
//         response.assert_status_not_found();
//     })
//     .await
// }
