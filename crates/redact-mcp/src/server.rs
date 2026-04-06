use tonic::{transport::Server, Request, Response, Status};
use redact_core::{AnalyzerEngine, AnalysisResult};
use redact_mcp::redact::{redact_service_server::{RedactService, RedactServiceServer}, AnalyzeRequest, AnalysisResponse, HealthRequest, HealthResponse, StatsRequest, StatsResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RedactServer {
    engine: Arc<AnalyzerEngine>,
    stats: Arc<Mutex<Stats>>,
}

#[derive(Default)]
struct Stats {
    requests: u64,
    total_time_ms: f64,
}

#[tonic::async_trait]
impl RedactService for RedactServer {
    async fn analyze_text(
        &self,
        request: Request<AnalyzeRequest>,
    ) -> Result<Response<AnalysisResponse>, Status> {
        let req = request.into_inner();
        let start = std::time::Instant::now();

        let res: AnalysisResult = self
            .engine
            .analyze(&req.text, &req.lang, Some(req.conf_threshold))
            .map_err(|e| Status::internal(e.to_string()))?;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        // обновляем статистику
        {
            let mut s = self.stats.lock().await;
            s.requests += 1;
            s.total_time_ms += elapsed;
        }

        Ok(Response::new(AnalysisResponse {
            result_json: serde_json::to_string(&res).unwrap(),
        }))
    }

    async fn health_check(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: "OK".into(),
        }))
    }

    async fn get_stats(
        &self,
        _request: Request<StatsRequest>,
    ) -> Result<Response<StatsResponse>, Status> {
        let s = self.stats.lock().await;
        Ok(Response::new(StatsResponse {
            requests: s.requests,
            avg_time_ms: if s.requests > 0 {
                s.total_time_ms / s.requests as f64
            } else {
                0.0
            },
        }))
    }
}

pub async fn run_mcp_server(addr: std::net::SocketAddr, engine: Arc<AnalyzerEngine>) {
    let server = RedactServer {
        engine,
        stats: Arc::new(Mutex::new(Stats::default())),
    };

    Server::builder()
        .add_service(RedactServiceServer::new(server))
        .serve(addr)
        .await
        .expect("MCP server crashed");
}