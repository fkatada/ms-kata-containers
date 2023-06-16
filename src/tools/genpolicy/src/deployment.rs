// Copyright (c) 2023 Microsoft Corporation
//
// SPDX-License-Identifier: Apache-2.0
//

// Allow K8s YAML field names.
#![allow(non_snake_case)]

use crate::obj_meta;
use crate::pod;
use crate::pod_template;
use crate::policy;
use crate::yaml;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Reference / Kubernetes API / Workload Resources / Deployment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deployment {
    apiVersion: String,
    kind: String,
    pub metadata: obj_meta::ObjectMeta,
    pub spec: DeploymentSpec,

    #[serde(skip)]
    doc_mapping: serde_yaml::Value,
}

/// Reference / Kubernetes API / Workload Resources / Deployment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeploymentSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    replicas: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<yaml::LabelSelector>,

    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<DeploymentStrategy>,

    pub template: pod_template::PodTemplateSpec,
    // TODO: additional fields.
}

/// Reference / Kubernetes API / Workload Resources / Deployment.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeploymentStrategy {
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    rollingUpdate: Option<RollingUpdateDeployment>,
}

/// Reference / Kubernetes API / Workload Resources / Deployment.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RollingUpdateDeployment {
    #[serde(skip_serializing_if = "Option::is_none")]
    maxSurge: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    maxUnavailable: Option<i32>,
}

#[async_trait]
impl yaml::K8sResource for Deployment {
    async fn init(
        &mut self,
        use_cache: bool,
        doc_mapping: &serde_yaml::Value,
        _silent_unsupported_fields: bool,
    ) {
        yaml::k8s_resource_init(&mut self.spec.template.spec, use_cache).await;
        self.doc_mapping = doc_mapping.clone();
    }

    fn get_sandbox_name(&self) -> Option<String> {
        None
    }

    fn get_namespace(&self) -> String {
        self.metadata.get_namespace()
    }

    fn get_container_mounts_and_storages(
        &self,
        policy_mounts: &mut Vec<policy::KataMount>,
        storages: &mut Vec<policy::SerializedStorage>,
        container: &pod::Container,
        agent_policy: &policy::AgentPolicy,
    ) {
        if let Some(volumes) = &self.spec.template.spec.volumes {
            yaml::get_container_mounts_and_storages(
                policy_mounts,
                storages,
                container,
                agent_policy,
                volumes,
            );
        }
    }

    fn generate_policy(&self, agent_policy: &policy::AgentPolicy) -> String {
        agent_policy.generate_policy(self)
    }

    fn serialize(&mut self, policy: &str) -> String {
        yaml::add_policy_annotation(&mut self.doc_mapping, "spec.template.metadata", policy);
        serde_yaml::to_string(&self.doc_mapping).unwrap()
    }

    fn get_containers(&self) -> &Vec<pod::Container> {
        &self.spec.template.spec.containers
    }

    fn get_annotations(&self) -> Option<BTreeMap<String, String>> {
        if let Some(annotations) = &self.spec.template.metadata.annotations {
            return Some(annotations.clone());
        }
        None
    }

    fn use_host_network(&self) -> bool {
        if let Some(host_network) = self.spec.template.spec.hostNetwork {
            return host_network;
        }
        false
    }
}