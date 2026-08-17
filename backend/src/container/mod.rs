#![allow(unused_imports)]

pub mod client_helper;
pub mod detector;
pub mod docker;
pub mod engine;
pub mod podman;
pub mod profiles;

pub use detector::{
    build_container_engine, detect_container_engine, is_socket_accessible, ContainerEngineType,
};
pub use docker::{demux_docker_logs, find_docker_socket_path, DockerEngine};
pub use engine::{
    BoxFuture, ContainerEngine, ContainerMetrics, ContainerStatusResponse, ServerStatus,
};
pub use podman::{
    calculate_uptime_from_rfc3339, find_podman_socket_path, get_current_uid,
    parse_rfc3339_to_epoch, split_utc_offset, try_systemd_action, PodmanEngine,
};
pub use profiles::{
    ContainerPortMapping, ContainerProfileType, ContainerVolumeMapping, MinecraftContainerProfile,
};

use crate::error::AppError;

/// Zero-cost enum wrapper dispatching to concrete container engine implementations.
#[derive(Debug, Clone)]
pub enum AnyContainerEngine {
    Podman(PodmanEngine),
    Docker(DockerEngine),
}

impl Default for AnyContainerEngine {
    fn default() -> Self {
        build_container_engine(ContainerEngineType::Auto, None)
    }
}

impl ContainerEngine for AnyContainerEngine {
    fn engine_name(&self) -> &str {
        match self {
            Self::Podman(e) => e.engine_name(),
            Self::Docker(e) => e.engine_name(),
        }
    }

    fn get_status<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<ContainerStatusResponse, AppError>> {
        match self {
            Self::Podman(e) => e.get_status(id),
            Self::Docker(e) => e.get_status(id),
        }
    }

    fn get_stats<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<ContainerMetrics, AppError>> {
        match self {
            Self::Podman(e) => e.get_stats(id),
            Self::Docker(e) => e.get_stats(id),
        }
    }

    fn start_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        match self {
            Self::Podman(e) => ContainerEngine::start_container(e, id),
            Self::Docker(e) => ContainerEngine::start_container(e, id),
        }
    }

    fn stop_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        match self {
            Self::Podman(e) => ContainerEngine::stop_container(e, id),
            Self::Docker(e) => ContainerEngine::stop_container(e, id),
        }
    }

    fn restart_container<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), AppError>> {
        match self {
            Self::Podman(e) => ContainerEngine::restart_container(e, id),
            Self::Docker(e) => ContainerEngine::restart_container(e, id),
        }
    }

    fn get_logs<'a>(
        &'a self,
        id: &'a str,
        tail: usize,
    ) -> BoxFuture<'a, Result<Vec<String>, AppError>> {
        match self {
            Self::Podman(e) => ContainerEngine::get_logs(e, id, tail),
            Self::Docker(e) => ContainerEngine::get_logs(e, id, tail),
        }
    }
}
