use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerProfileType {
    /// Native ChiPanel custom lazymc hibernating image
    LazymcCustom,
    /// itzg/minecraft-server standard third-party image
    ItzgMinecraft,
    /// Generic OCI container profile (Palworld, Valheim, Custom)
    GenericOci,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPortMapping {
    pub host_port: u16,
    pub container_port: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_protocol() -> String {
    "tcp".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerVolumeMapping {
    pub host_path: String,
    pub container_path: String,
    #[serde(default)]
    pub read_only: bool,
}

/// Comprehensive Minecraft and Game Server container profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftContainerProfile {
    pub profile_type: ContainerProfileType,
    pub container_name: String,
    pub image: String,
    #[serde(default = "default_true")]
    pub eula: bool,
    pub engine_type: String,
    pub version: String,
    pub loader_version: Option<String>,
    pub memory: String,
    pub init_memory: Option<String>,
    pub max_memory: Option<String>,
    #[serde(default = "default_true")]
    pub rcon_enabled: bool,
    #[serde(default = "default_rcon_port")]
    pub rcon_port: u16,
    pub rcon_password: Option<String>,
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default)]
    pub extra_env: BTreeMap<String, String>,
    #[serde(default)]
    pub volumes: Vec<ContainerVolumeMapping>,
    #[serde(default)]
    pub ports: Vec<ContainerPortMapping>,
}

fn default_true() -> bool {
    true
}

fn default_rcon_port() -> u16 {
    25575
}

fn default_server_port() -> u16 {
    25565
}

impl MinecraftContainerProfile {
    /// Creates a default `itzg/minecraft-server` profile.
    pub fn itzg_default(
        container_name: &str,
        engine_type: &str,
        version: &str,
        memory: &str,
        data_dir: &str,
    ) -> Self {
        Self {
            profile_type: ContainerProfileType::ItzgMinecraft,
            container_name: container_name.to_string(),
            image: "docker.io/itzg/minecraft-server:latest".to_string(),
            eula: true,
            engine_type: engine_type.to_uppercase(),
            version: version.to_string(),
            loader_version: None,
            memory: memory.to_string(),
            init_memory: None,
            max_memory: None,
            rcon_enabled: true,
            rcon_port: 25575,
            rcon_password: None,
            server_port: 25565,
            extra_env: BTreeMap::new(),
            volumes: vec![ContainerVolumeMapping {
                host_path: data_dir.to_string(),
                container_path: "/data".to_string(),
                read_only: false,
            }],
            ports: vec![
                ContainerPortMapping {
                    host_port: 25565,
                    container_port: 25565,
                    protocol: "tcp".to_string(),
                },
                ContainerPortMapping {
                    host_port: 25575,
                    container_port: 25575,
                    protocol: "tcp".to_string(),
                },
            ],
        }
    }

    /// Creates a default `lazymc` custom quadlet profile.
    pub fn lazymc_default(
        container_name: &str,
        engine_type: &str,
        version: &str,
        memory: &str,
        data_dir: &str,
    ) -> Self {
        Self {
            profile_type: ContainerProfileType::LazymcCustom,
            container_name: container_name.to_string(),
            image: "docker.io/itzg/minecraft-server:latest".to_string(),
            eula: true,
            engine_type: engine_type.to_uppercase(),
            version: version.to_string(),
            loader_version: None,
            memory: memory.to_string(),
            init_memory: None,
            max_memory: None,
            rcon_enabled: true,
            rcon_port: 25575,
            rcon_password: None,
            server_port: 25565,
            extra_env: BTreeMap::new(),
            volumes: vec![ContainerVolumeMapping {
                host_path: data_dir.to_string(),
                container_path: "/data".to_string(),
                read_only: false,
            }],
            ports: vec![
                ContainerPortMapping {
                    host_port: 25565,
                    container_port: 25565,
                    protocol: "tcp".to_string(),
                },
                ContainerPortMapping {
                    host_port: 25575,
                    container_port: 25575,
                    protocol: "tcp".to_string(),
                },
            ],
        }
    }

    /// Translates profile configuration into environment variables mapped for the specific container runtime.
    pub fn to_environment_map(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();

        match self.profile_type {
            ContainerProfileType::ItzgMinecraft => {
                if self.eula {
                    map.insert("EULA".to_string(), "TRUE".to_string());
                }
                map.insert("TYPE".to_string(), self.engine_type.clone());
                map.insert("VERSION".to_string(), self.version.clone());
                map.insert("MEMORY".to_string(), self.memory.clone());

                if let Some(init) = &self.init_memory {
                    map.insert("INIT_MEMORY".to_string(), init.clone());
                }
                if let Some(max) = &self.max_memory {
                    map.insert("MAX_MEMORY".to_string(), max.clone());
                }

                if self.rcon_enabled {
                    map.insert("ENABLE_RCON".to_string(), "true".to_string());
                    map.insert("RCON_PORT".to_string(), self.rcon_port.to_string());
                    if let Some(pass) = &self.rcon_password {
                        map.insert("RCON_PASSWORD".to_string(), pass.clone());
                    }
                }

                if let Some(loader_ver) = &self.loader_version {
                    match self.engine_type.as_str() {
                        "FABRIC" => {
                            map.insert("FABRIC_LOADER_VERSION".to_string(), loader_ver.clone());
                        }
                        "QUILT" => {
                            map.insert("QUILT_LOADER_VERSION".to_string(), loader_ver.clone());
                        }
                        "FORGE" => {
                            map.insert("FORGE_VERSION".to_string(), loader_ver.clone());
                        }
                        "NEOFORGE" => {
                            map.insert("NEOFORGE_VERSION".to_string(), loader_ver.clone());
                        }
                        "PAPER" => {
                            map.insert("PAPER_BUILD".to_string(), loader_ver.clone());
                        }
                        "PURPUR" => {
                            map.insert("PURPUR_BUILD".to_string(), loader_ver.clone());
                        }
                        "FOLIA" => {
                            map.insert("BUILD_NUMBER".to_string(), loader_ver.clone());
                        }
                        _ => {}
                    }
                }
            }
            ContainerProfileType::LazymcCustom => {
                map.insert("TYPE".to_string(), self.engine_type.clone());
                map.insert("VERSION".to_string(), self.version.clone());
                map.insert("MEMORY".to_string(), self.memory.clone());
                if self.eula {
                    map.insert("EULA".to_string(), "TRUE".to_string());
                }
                if let Some(loader_ver) = &self.loader_version {
                    map.insert("LOADER_VERSION".to_string(), loader_ver.clone());
                }
            }
            ContainerProfileType::GenericOci => {
                map.insert("SERVER_TYPE".to_string(), self.engine_type.clone());
                map.insert("SERVER_VERSION".to_string(), self.version.clone());
            }
        }

        // Merge extra env vars
        for (k, v) in &self.extra_env {
            map.insert(k.clone(), v.clone());
        }

        map
    }

    /// Generates a Systemd Quadlet `.container` definition.
    pub fn to_quadlet_unit(&self) -> String {
        let mut out = String::new();
        out.push_str("[Unit]\n");
        out.push_str(&format!(
            "Description=ChiPanel Managed Container - {}\n",
            self.container_name
        ));
        out.push_str("After=network-online.target\n\n");

        out.push_str("[Container]\n");
        out.push_str(&format!("ContainerName={}\n", self.container_name));
        out.push_str(&format!("Image={}\n", self.image));

        let envs = self.to_environment_map();
        for (k, v) in envs {
            out.push_str(&format!("Environment={}={}\n", k, v));
        }

        for vol in &self.volumes {
            let ro_flag = if vol.read_only { ":ro" } else { ":z" };
            out.push_str(&format!(
                "Volume={}:{}{}\n",
                vol.host_path, vol.container_path, ro_flag
            ));
        }

        for p in &self.ports {
            out.push_str(&format!(
                "PublishPort={}:{}/{}\n",
                p.host_port, p.container_port, p.protocol
            ));
        }

        out.push_str("\n[Service]\n");
        out.push_str("Restart=always\n");
        out.push_str("TimeoutStartSec=300\n");

        out.push_str("\n[Install]\n");
        out.push_str("WantedBy=default.target\n");

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn itzg_profile_maps_environment_variables() {
        let mut profile = MinecraftContainerProfile::itzg_default(
            "minecraft-server",
            "PAPER",
            "1.21.1",
            "4G",
            "/home/user/mc-data",
        );
        profile.loader_version = Some("123".to_string());
        profile.rcon_password = Some("secret".to_string());

        let envs = profile.to_environment_map();
        assert_eq!(envs.get("EULA").map(|s| s.as_str()), Some("TRUE"));
        assert_eq!(envs.get("TYPE").map(|s| s.as_str()), Some("PAPER"));
        assert_eq!(envs.get("VERSION").map(|s| s.as_str()), Some("1.21.1"));
        assert_eq!(envs.get("MEMORY").map(|s| s.as_str()), Some("4G"));
        assert_eq!(envs.get("ENABLE_RCON").map(|s| s.as_str()), Some("true"));
        assert_eq!(envs.get("RCON_PASSWORD").map(|s| s.as_str()), Some("secret"));
        assert_eq!(envs.get("PAPER_BUILD").map(|s| s.as_str()), Some("123"));
    }

    #[test]
    fn quadlet_generation_contains_expected_sections() {
        let profile = MinecraftContainerProfile::itzg_default(
            "minecraft-server",
            "FABRIC",
            "1.20.4",
            "8G",
            "/data/mc",
        );
        let quadlet = profile.to_quadlet_unit();
        assert!(quadlet.contains("[Container]"));
        assert!(quadlet.contains("ContainerName=minecraft-server"));
        assert!(quadlet.contains("Environment=TYPE=FABRIC"));
        assert!(quadlet.contains("Environment=VERSION=1.20.4"));
        assert!(quadlet.contains("PublishPort=25565:25565/tcp"));
    }
}
