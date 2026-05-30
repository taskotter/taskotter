use crate::API_VERSION;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub path: &'static str,
    pub stability: ContractStability,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Patch,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractStability {
    Draft,
    Stable,
}

pub fn control_plane_endpoints() -> Vec<Endpoint> {
    vec![
        Endpoint {
            method: HttpMethod::Get,
            path: "/v1/working-groups/{working_group_id}/issues",
            stability: ContractStability::Draft,
        },
        Endpoint {
            method: HttpMethod::Post,
            path: "/v1/policy/decisions",
            stability: ContractStability::Draft,
        },
        Endpoint {
            method: HttpMethod::Post,
            path: "/v1/usage/events",
            stability: ContractStability::Draft,
        },
        Endpoint {
            method: HttpMethod::Post,
            path: "/v1/runners/{runner_id}/jobs",
            stability: ContractStability::Draft,
        },
        Endpoint {
            method: HttpMethod::Post,
            path: "/v1/gateway/requests",
            stability: ContractStability::Draft,
        },
    ]
}

pub fn versioned_path(path: &str) -> String {
    format!("/{API_VERSION}/{}", path.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_paths_are_normalized() {
        assert_eq!(versioned_path("issues"), "/v1/issues");
        assert_eq!(versioned_path("/issues"), "/v1/issues");
    }
}
