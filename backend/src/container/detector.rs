use serde::{Deserialize, Serialize};
use std::fmt;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::info;

use crate::container::docker::{find_docker_socket_path, DockerEngine};
use crate::container::engine::ContainerEngine;
use crate::container::podman::{find_podman_socket_path, PodmanEngine};
use crate::container::AnyContainerEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContainerEngineType {
    Podman,
    Docker,
    #[default]
    Auto,
}

impl fmt::Display for ContainerEngineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Podman => write!(f, "podman"),
            Self::Docker => write!(f, "docker"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

impl FromStr for ContainerEngineType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "podman" => Ok(Self::Podman),
            "docker" => Ok(Self::Docker),
            _ => Ok(Self::Auto),
        }
    }
}

pub fn is_socket_accessible(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.file_type().is_socket())
        .unwrap_or(false)
}

/// Detects the available container engine based on preference and active sockets.
pub fn detect_container_engine(preference: ContainerEngineType) -> (ContainerEngineType, PathBuf) {
    match preference {
        ContainerEngineType::Podman => {
            let socket = find_podman_socket_path();
            info!("Container engine set to Podman explicitly at {:?}", socket);
            (ContainerEngineType::Podman, socket)
        }
        ContainerEngineType::Docker => {
            let socket = find_docker_socket_path();
            info!("Container engine set to Docker explicitly at {:?}", socket);
            (ContainerEngineType::Docker, socket)
        }
        ContainerEngineType::Auto => {
            // Auto detection priority:
            // 1. Podman rootless user socket or explicit PODMAN_SOCKET
            let podman_sock = find_podman_socket_path();
            if is_socket_accessible(&podman_sock) {
                info!("Auto-detected active Podman socket at {:?}", podman_sock);
                return (ContainerEngineType::Podman, podman_sock);
            }

            // 2. Docker socket or explicit DOCKER_HOST
            let docker_sock = find_docker_socket_path();
            if is_socket_accessible(&docker_sock) {
                info!("Auto-detected active Docker socket at {:?}", docker_sock);
                return (ContainerEngineType::Docker, docker_sock);
            }

            // Fallback: Podman default path
            info!(
                "No active socket probed; defaulting to Podman socket path at {:?}",
                podman_sock
            );
            (ContainerEngineType::Podman, podman_sock)
        }
    }
}

/// Factory function to build an initialized `AnyContainerEngine` based on preference and optional socket override.
pub fn build_container_engine(
    preference: ContainerEngineType,
    socket_override: Option<PathBuf>,
) -> AnyContainerEngine {
    let (engine_type, resolved_socket) = detect_container_engine(preference);
    let socket = socket_override.unwrap_or(resolved_socket);

    match engine_type {
        ContainerEngineType::Podman | ContainerEngineType::Auto => {
            AnyContainerEngine::Podman(PodmanEngine::new(socket))
        }
        ContainerEngineType::Docker => AnyContainerEngine::Docker(DockerEngine::new(socket)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_engine_types() {
        assert_eq!(
            ContainerEngineType::from_str("podman").unwrap(),
            ContainerEngineType::Podman
        );
        assert_eq!(
            ContainerEngineType::from_str("DOCKER").unwrap(),
            ContainerEngineType::Docker
        );
        assert_eq!(
            ContainerEngineType::from_str("auto").unwrap(),
            ContainerEngineType::Auto
        );
        assert_eq!(
            ContainerEngineType::from_str("unknown").unwrap(),
            ContainerEngineType::Auto
        );
    }
}
