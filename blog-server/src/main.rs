mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

pub mod grpc {
    tonic::include_proto!("grpc");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("post_descriptor");
}

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenvy::dotenv;
use tonic::transport::Server;

use application::{auth_service::AuthService, blog_service::BlogService};
use data::{post_repository::PostgresPostRepository, user_repository::PostgresUserRepository};
use infrastructure::{database, jwt::JwtService, logging};
use presentation::{grpc_service::BlogGrpcService, http_handlers, middleware};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    logging::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = database::create_pool(&database_url).await?;

    database::run_migrations(&pool).await?;

    let jwt_service = Arc::new(JwtService::new(&jwt_secret));
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let post_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new(user_repo.clone(), jwt_service.clone()));
    let blog_service = Arc::new(BlogService::new(post_repo.clone(), user_repo.clone()));

    let auth_data = web::Data::new(auth_service.clone());
    let blog_data = web::Data::new(blog_service.clone());
    let jwt_data = web::Data::new(jwt_service.clone());

    let auth_middleware = HttpAuthentication::bearer(middleware::jwt_validator);

    let server_future = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(auth_data.clone())
            .app_data(blog_data.clone())
            .app_data(jwt_data.clone())
            .wrap(Logger::default())
            .wrap(cors)
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(http_handlers::register))
                            .route("/login", web::post().to(http_handlers::login)),
                    )
                    .service(
                        web::scope("/posts")
                            .route("", web::get().to(http_handlers::list_posts))
                            .route("/{id}", web::get().to(http_handlers::get_post))
                            .service(
                                web::scope("")
                                    .wrap(auth_middleware.clone())
                                    .route("", web::post().to(http_handlers::create_post))
                                    .route("/{id}", web::put().to(http_handlers::update_post))
                                    .route("/{id}", web::delete().to(http_handlers::delete_post)),
                            ),
                    ),
            )
    })
    .bind(("0.0.0.0", 3000))?
    .run();

    let grpc_addr = "0.0.0.0:50051".parse()?;
    let grpc_service = BlogGrpcService::new(
        auth_service.clone(),
        blog_service.clone(),
        jwt_service.clone(),
    );

    let reflector = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(grpc::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let grpc_future = Server::builder()
        .add_service(reflector)
        .add_service(grpc::blog_service_server::BlogServiceServer::new(grpc_service))
        .serve(grpc_addr);

    tokio::select! {
        res = server_future => res.map_err(|e| Box::<dyn std::error::Error>::from(e))?,
        res = grpc_future => res.map_err(|e| Box::<dyn std::error::Error>::from(e))?,
    }

    Ok(())
}
