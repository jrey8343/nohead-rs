use axum::{
    Form, Router, async_trait,
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use axum_flash::{Flash, IncomingFlashes};
use nohead_rs_db::{DeserializeOwned, Validate};

use crate::state::SharedAppState;

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
///     fn router() -> Router<SharedAppState> {
///         Router::new()
///         .route("/", get(Self::index))
///         .route("/", post(Self::create))
///         .route("/:id", get(Self::show))
///         .route("/:id", put(Self::update))
///         .route("/:id", delete(Self::delete));
///     }
///
///     fn index(
///         State(app_state): State<SharedAppState>,
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
    fn router() -> Router<SharedAppState>;

    /// Index handler to list all records
    async fn read_all(
        State(app_state): State<SharedAppState>,
        flashes: IncomingFlashes,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error>;

    /// Create handler to create a new record
    async fn create(
        flash: Flash,
        State(app_state): State<SharedAppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error>;

    async fn create_batch(
        flash: Flash,
        State(app_state): State<SharedAppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error>;

    /// Show handler to display a single record
    async fn read_one(
        Path(id): Path<Self::Id>,
        State(app_state): State<SharedAppState>,
        flashes: IncomingFlashes,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error>;

    /// Update handler to update a single record
    async fn update(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<SharedAppState>,
        form: Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error>;

    /// Delete handler to delete a single record
    async fn delete(
        flash: Flash,
        Path(id): Path<i64>,
        State(app_state): State<SharedAppState>,
    ) -> Result<(Flash, Redirect), Self::Error>;
}
