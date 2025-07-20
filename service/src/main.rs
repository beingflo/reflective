mod auth;
mod error;
mod image;
mod s3_utils;
mod spa;
mod tag;
mod utils;
mod worker;

use std::{env, sync::Arc, time::Duration};

use async_channel::unbounded;
use auth::{login, signup};
use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::{Request, Response, StatusCode},
    routing::{delete, get, post},
    Router,
};
use dotenv::dotenv;
use error::AppError;
use image::{get_image, search_images, upload_image};
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_sdk::trace::SdkTracerProvider;
use s3::Bucket;
use spa::static_handler;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tag::{add_tags, remove_tags};
use tokio::{signal, sync::Mutex};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{error, info, Span};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use utils::get_bucket;
use uuid::Uuid;
use worker::{start_workers, ImageProcessingJob};

#[derive(Clone, Debug)]
pub struct AppState {
    pool: Pool<Postgres>,
    bucket: Arc<Mutex<Bucket>>,
    job_sender: async_channel::Sender<ImageProcessingJob>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Initialize OpenTelemetry OTLP exporter
    let tracer = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(tracer)
        .build();

    global::set_tracer_provider(provider.clone());

    // Set up tracing with both console output and OpenTelemetry
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(
            OpenTelemetryLayer::new(provider.tracer("reflective-service"))
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .init();

    info!(message = "Starting application");

    let bucket = get_bucket()?;

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(80)
        .connect(db_url.as_str())
        .await?;

    info!(message = "Connected to DB");

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!(message = "Migrations applied");

    // Create job queue for async image processing
    let (job_sender, job_receiver) = unbounded::<ImageProcessingJob>();

    let state = AppState {
        pool: pool.clone(),
        bucket: Arc::new(Mutex::new(bucket)),
        job_sender,
    };

    // Start background workers (default: 2 workers)
    let worker_count = std::env::var("WORKER_COUNT")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<usize>()
        .unwrap_or(2);

    start_workers(state.clone(), job_receiver, worker_count).await;
    info!(message = "Started {} background workers", worker_count);

    let Some((_, port)) = env::vars().find(|v| v.0.eq("SERVE_PORT")) else {
        error!("Port not present in environment");
        panic!()
    };

    let app = Router::new()
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/images", post(upload_image))
        .route("/api/images/search", post(search_images))
        .route("/api/images/{id}", get(get_image))
        .route("/api/tags", post(add_tags))
        .route("/api/tags", delete(remove_tags))
        .fallback(static_handler)
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|_request: &Request<Body>| {
                    let request_id = Uuid::new_v4().to_string();
                    tracing::info_span!("http-request", %request_id)
                })
                .on_request(|request: &Request<Body>, _span: &Span| {
                    info!(
                        message = "request",
                        request = request.method().as_str(),
                        uri = request.uri().path().to_string(),
                        referrer = request
                            .headers()
                            .get("referer")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or(""),
                        user_agent = request
                            .headers()
                            .get("user-agent")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("")
                    )
                })
                .on_response(
                    |response: &Response<Body>, latency: Duration, _span: &Span| {
                        info!(
                            message = "response_status",
                            status = response.status().as_u16(),
                            latency = latency.as_nanos()
                        )
                    },
                )
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        error!(message = "error", error = error.to_string())
                    },
                ),
        )
        .layer(DefaultBodyLimit::disable());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(message = "Starting server", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            error!(message = "Failed to start server", error=%e);
            AppError::Status(StatusCode::SERVICE_UNAVAILABLE)
        })?;

    // Shutdown OpenTelemetry to flush remaining traces
    provider.shutdown()?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {
            info!("Ctrl+C received, shutting down")
        },
        _ = terminate => {
            info!("SIGTERM received, shutting down")
        },
    }
}
