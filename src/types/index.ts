export interface Endpoint {
  id: string;
  name: string;
  slug: string;
  allowed_methods: string[];
  default_status: number;
  default_headers: Record<string, string>;
  default_body: string;
  default_content_type: string;
  delay_ms: number;
  error_rate_percent: number;
  error_status: number;
  error_body?: string | null;
  redirect_url?: string | null;
  redirect_status?: number | null;
  auto_forward_url?: string | null;
  forward_response?: boolean | null;
  is_active: boolean;
  created_at: string;
  request_count?: number;
}

export interface WebhookRequest {
  id: string;
  endpoint_id: string;
  method: string;
  path: string;
  query_params: Record<string, string>;
  headers: Record<string, string>;
  body: string;
  content_type: string;
  ip_address: string;
  timestamp: string;
  response_status: number;
  response_time_ms: number;
  matched_rule_id?: string | null;
  matched_rule_name?: string | null;
  is_simulated_error?: boolean;
}

export type ConditionType = 'header' | 'path' | 'query' | 'json_body' | 'method';
export type ConditionOperator = 'equals' | 'contains' | 'regex' | 'exists';

export interface ResponseRule {
  id: string;
  endpoint_id: string;
  name: string;
  priority: number;
  condition_type: ConditionType;
  condition_field: string;
  condition_operator: ConditionOperator;
  condition_value: string;
  response_status: number;
  response_headers: Record<string, string>;
  response_body: string;
  response_delay_ms: number;
  is_enabled: boolean;
}

export interface TunnelStatus {
  is_running: boolean;
  public_url: string | null;
  error: string | null;
  started_at: string | null;
  is_installing?: boolean;
  status_message?: string | null;
}

export interface ApiForwarder {
  id: string;
  name: string;
  host: string;
  port: number;
  created_at: string;
  is_running?: boolean;
  public_url?: string | null;
  error?: string | null;
  started_at?: string | null;
  is_loading?: boolean;
}

export interface ServerStatus {
  is_running: boolean;
  port: number;
  local_base_url: string;
}

export interface ReplayRequestPayload {
  request_id: string;
  target_url: string;
  method: string;
  headers: Record<string, string>;
  body: string;
}

export interface ReplayResponse {
  status: number;
  status_text: string;
  headers: Record<string, string>;
  body: string;
  time_ms: number;
  error?: string | null;
}
