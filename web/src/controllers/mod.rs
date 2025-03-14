use async_trait::async_trait;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use nohead_rs_db::{DeserializeOwned, Validate};

use crate::{
    initializers::view_engine::engine::{View, ViewEngine},
    middlewares::flash::{Flash, IncomingFlashes},
    state::AppState,
};

pub mod auth;
pub mod home;
pub mod ping;
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
    async fn read_all(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error>;

    /// Create handler to create a new record
    async fn create(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error>;

    async fn create_batch(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error>;

    /// Show handler to display a single record
    async fn read_one(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error>;

    /// Update handler to update a single record
    async fn update(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        form: Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error>;

    /// Delete handler to delete a single record
    async fn delete(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(Flash, Redirect), Self::Error>;
}
