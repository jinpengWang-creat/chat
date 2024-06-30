use crate::error::ErrorOutput;
use crate::models::{CreateChat, CreateMessage, ListMessage, SignupUser, UpdateChat};
use crate::AppState;
use crate::{handlers::*, models::SigninUser};
use axum::Router;
use chat_core::{Chat, ChatType, ChatUser, Message, User, Workspace};
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

pub trait OpenApiRouter {
    fn openapi(self) -> Self;
}

#[derive(OpenApi)]
#[openapi(
        paths(
            signup_handler,
            signin_handler,
            get_chat_handler,
            create_chat_handler,
            list_chat_handler,
            update_chat_handler,
            delete_chat_handler,
        ),
        modifiers(&SecurityAddon),
        components(
            schemas(User, Chat, ChatType, ChatUser, Message, Workspace, SignupUser, SigninUser,
                AuthOutput, ErrorOutput, CreateChat, CreateMessage, ListMessage,  UpdateChat),
        ),
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "Authorization",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

impl OpenApiRouter for Router<AppState> {
    fn openapi(self) -> Self {
        self.merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
            .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    }
}
