use crate::domain::entities::{ProviderAccount, UsageSnapshot};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ports::UsageProviderPort;
use crate::domain::value_objects::{
    ModelUsageBreakdown, ProviderCredentials, ProviderKind, QuotaWindow, SyncStatus, TokenMetrics,
};
use crate::infrastructure::providers::aws_sigv4::{sign_aws_request, SigV4Input};
use async_trait::async_trait;
use chrono::{Datelike, Duration, Utc};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;
use uuid::Uuid;

pub struct AmazonBedrockAdapter {
    client: Client,
}

impl AmazonBedrockAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    async fn fetch_cost_explorer(
        &self,
        access_key: &str,
        secret_key: &str,
        session_token: Option<&str>,
    ) -> DomainResult<(f64, f64, Vec<ModelUsageBreakdown>)> {
        let now = Utc::now();
        let month_start = format!("{:04}-{:02}-01", now.year(), now.month());
        let tomorrow = (now + Duration::days(1)).format("%Y-%m-%d").to_string();

        let body = json!({
            "TimePeriod": { "Start": month_start, "End": tomorrow },
            "Granularity": "DAILY",
            "Metrics": ["UnblendedCost"],
            "Filter": {
                "Dimensions": {
                    "Key": "SERVICE",
                    "Values": ["Amazon Bedrock"]
                }
            },
            "GroupBy": [
                { "Type": "DIMENSION", "Key": "USAGE_TYPE" }
            ]
        });

        let payload = serde_json::to_vec(&body).map_err(|e| DomainError::ProviderParseFailed {
            provider: "amazon_bedrock".to_string(),
            message: e.to_string(),
        })?;

        let host = "ce.us-east-1.amazonaws.com";
        let target = "AWSInsightsIndexService.GetCostAndUsage";
        let content_type = "application/x-amz-json-1.1";

        let signed = sign_aws_request(&SigV4Input {
            method: "POST",
            service: "ce",
            region: "us-east-1",
            host,
            canonical_uri: "/",
            canonical_query: "",
            extra_headers: &[("content-type", content_type), ("x-amz-target", target)],
            payload: &payload,
            access_key_id: access_key,
            secret_access_key: secret_key,
            session_token,
            timestamp: now,
        });

        let mut req = self
            .client
            .post(format!("https://{host}/"))
            .header("content-type", content_type)
            .header("x-amz-target", target)
            .header("x-amz-date", signed.amz_date)
            .header("x-amz-content-sha256", signed.payload_hash)
            .header("authorization", signed.authorization)
            .body(payload);

        if let Some(token) = session_token {
            req = req.header("x-amz-security-token", token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "amazon_bedrock".to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(DomainError::ProviderRequestFailed {
                provider: "amazon_bedrock".to_string(),
                message: format!("HTTP {status}: {text}"),
            });
        }

        let json_val: Value =
            resp.json()
                .await
                .map_err(|e| DomainError::ProviderParseFailed {
                    provider: "amazon_bedrock".to_string(),
                    message: e.to_string(),
                })?;

        let mut total_spend = 0.0_f64;
        let mut latest_daily = 0.0_f64;
        let mut model_costs: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();

        if let Some(results) = json_val.get("ResultsByTime").and_then(|v| v.as_array()) {
            for day in results {
                let mut day_total = 0.0_f64;
                if let Some(groups) = day.get("Groups").and_then(|v| v.as_array()) {
                    for grp in groups {
                        let key = grp
                            .get("Keys")
                            .and_then(|k| k.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.as_str())
                            .unwrap_or("Bedrock");
                        let amount = grp
                            .pointer("/Metrics/UnblendedCost/Amount")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        day_total += amount;
                        *model_costs.entry(key.to_string()).or_insert(0.0) += amount;
                    }
                }
                total_spend += day_total;
                latest_daily = day_total;
            }
        }

        let mut breakdown: Vec<ModelUsageBreakdown> = model_costs
            .into_iter()
            .map(|(model_id, spend_usd)| ModelUsageBreakdown {
                model_id,
                spend_usd,
                input_tokens: 0,
                output_tokens: 0,
                request_count: 0,
            })
            .collect();
        breakdown.sort_by(|a, b| {
            b.spend_usd
                .partial_cmp(&a.spend_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        breakdown.truncate(8);

        Ok((total_spend, latest_daily, breakdown))
    }

    async fn fetch_cloudwatch_metric_sum(
        &self,
        region: &str,
        metric_name: &str,
        access_key: &str,
        secret_key: &str,
        session_token: Option<&str>,
    ) -> u64 {
        let now = Utc::now();
        let start = now - Duration::days(30);
        let host = format!("monitoring.{region}.amazonaws.com");
        let query = format!(
            "Action=GetMetricStatistics&EndTime={}&MetricName={}&Namespace=AWS%2FBedrock&Period=86400&StartTime={}&Statistics.member.1=Sum&Version=2010-08-01",
            url_encode_timestamp(now),
            metric_name,
            url_encode_timestamp(start),
        );

        let signed = sign_aws_request(&SigV4Input {
            method: "GET",
            service: "monitoring",
            region,
            host: &host,
            canonical_uri: "/",
            canonical_query: &query,
            extra_headers: &[("accept", "application/json")],
            payload: b"",
            access_key_id: access_key,
            secret_access_key: secret_key,
            session_token,
            timestamp: now,
        });

        let mut req = self
            .client
            .get(format!("https://{host}/?{query}"))
            .header("accept", "application/json")
            .header("x-amz-date", signed.amz_date)
            .header("x-amz-content-sha256", signed.payload_hash)
            .header("authorization", signed.authorization);

        if let Some(token) = session_token {
            req = req.header("x-amz-security-token", token);
        }

        let resp = match req.send().await {
            Ok(r) if r.status().is_success() => r,
            _ => return 0,
        };

        let val: Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => return 0,
        };

        val.pointer("/GetMetricStatisticsResponse/GetMetricStatisticsResult/Datapoints")
            .and_then(|v| v.as_array())
            .map(|pts| {
                pts.iter()
                    .filter_map(|p| p.get("Sum").and_then(|s| s.as_f64()))
                    .sum::<f64>() as u64
            })
            .unwrap_or(0)
    }
}

fn url_encode_timestamp(dt: chrono::DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H%%3A%M%%3A%SZ").to_string()
}

#[async_trait]
impl UsageProviderPort for AmazonBedrockAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::AmazonBedrock
    }

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot> {
        let start = Instant::now();
        let access_key = credentials
            .get("aws_access_key_id")
            .ok_or_else(|| DomainError::MissingCredential("aws_access_key_id".to_string()))?;
        let secret_key = credentials
            .get("aws_secret_access_key")
            .ok_or_else(|| DomainError::MissingCredential("aws_secret_access_key".to_string()))?;
        let session_token = credentials.get("aws_session_token");
        let region = credentials.get("aws_region").unwrap_or("us-east-1");

        let (total_spend, daily_spend, breakdown) = self
            .fetch_cost_explorer(access_key, secret_key, session_token)
            .await?;

        let (input_tokens, output_tokens, invocations) = tokio::join!(
            self.fetch_cloudwatch_metric_sum(
                region,
                "InputTokenCount",
                access_key,
                secret_key,
                session_token
            ),
            self.fetch_cloudwatch_metric_sum(
                region,
                "OutputTokenCount",
                access_key,
                secret_key,
                session_token
            ),
            self.fetch_cloudwatch_metric_sum(
                region,
                "Invocations",
                access_key,
                secret_key,
                session_token
            ),
        );

        let limit = account.monthly_budget_usd.unwrap_or(150.0);
        let utilization_pct = ((total_spend / limit) * 100.0).clamp(0.0, 100.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::AmazonBedrock,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: total_spend,
            daily_spend_usd: daily_spend,
            balance_usd: None,
            monthly_limit_usd: Some(limit),
            utilization_pct,
            tokens: TokenMetrics {
                input_tokens,
                output_tokens,
                cached_tokens: 0,
                total_requests: invocations,
            },
            quota_windows: vec![QuotaWindow {
                label: format!("AWS Cost Explorer ({region})"),
                used: total_spend,
                limit,
                unit: "USD".to_string(),
                resets_at: None,
            }],
            model_breakdown: breakdown,
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: format!("AWS SigV4 · {region}"),
        })
    }
}
