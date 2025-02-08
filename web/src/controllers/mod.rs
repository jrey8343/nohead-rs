use async_trait::async_trait;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use nohead_rs_db::{DeserializeOwned, Validate};

use crate::state::AppState;

pub mod home;
pub mod todos;

/// ------------------------------------------------------------------------
/// # A generic Controller trait for implenting a CRUD router for a model
/// ------------------------------------------------------------------------
///
/// Implement the Controller trait for your model's associated
/// controller to receive a complete CRUD set of handlers!
///
/// ## Example
///
/// ```rust
/// #[async_trait]
/// impl Controller for ExampleController {
///     type Id = i64;
///     type View = ExampleView;
///     type EntityChangeset = ExampleChangeset;
///     type Error = ExampleError;
///     
///
///     fn router() -> Router<AppState> {
///         Router::new()
///         .route("/", get(Self::index))
///         .route("/", post(Self::create))
///         .route("/:id", get(Self::show))
///         .route("/:id", put(Self::update))
///         .route("/:id", delete(Self::delete));
///     }
///
///     fn index(
///         State(app_state): State<AppState>,
///         flashes: IncomingFlashes,
///         ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
///         // your handler implementation here
///         Ok((flashes, view))
///         }
///         // ...other methods
/// ```
/// ------------------------------------------------------------------------

#[async_trait]
pub trait Controller {
    type Id: PartialOrd;
    type View: IntoResponse;
    type EntityChangeset: Validate + DeserializeOwned;
    type Error: IntoResponse;

    /// Produces a app router with all methods for the Controller
    fn router() -> Router<AppState>;

    /// Index handler to list all records
    async fn read_all(State(app_state): State<AppState>) -> Result<Self::View, Self::Error>;

    /// Create handler to create a new record
    async fn create(
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<Redirect, Self::Error>;

    async fn create_batch(
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<Redirect, Self::Error>;

    /// Show handler to display a single record
    async fn read_one(
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<Self::View, Self::Error>;

    /// Update handler to update a single record
    async fn update(
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        form: Form<Self::EntityChangeset>,
    ) -> Result<Redirect, Self::Error>;

    /// Delete handler to delete a single record
    async fn delete(
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<Redirect, Self::Error>;
}
