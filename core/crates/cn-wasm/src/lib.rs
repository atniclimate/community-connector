//! wasm-bindgen wrapper around cn-api.
//!
//! Native builds re-export cn-api for ADR-003 A-B5 coverage; wasm builds add
//! only the target-gated bindings required by ADR-003 D1-D4.

pub use cn_api::Api;

#[cfg(target_arch = "wasm32")]
mod bindings {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct CnApi {
        inner: cn_api::Api,
    }

    #[wasm_bindgen]
    impl CnApi {
        #[wasm_bindgen(constructor)]
        pub fn new() -> CnApi {
            CnApi {
                inner: cn_api::Api::new(),
            }
        }

        pub fn core_info(&self) -> String {
            self.inner.core_info()
        }

        pub fn load_group_begin(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            template_json: &str,
        ) -> String {
            self.inner
                .load_group_begin(group_id, viewer_ctx_json, template_json)
        }

        pub fn load_ops_chunk(&mut self, group_id: &str, ops_jsonl_chunk: &str) -> String {
            self.inner.load_ops_chunk(group_id, ops_jsonl_chunk)
        }

        pub fn load_group_commit(&mut self, group_id: &str, now_ms: f64) -> String {
            self.inner.load_group_commit(group_id, now_ms as i64)
        }

        pub fn submit_ops(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            ops_json: &str,
            now_ms: f64,
        ) -> String {
            self.inner
                .submit_ops(group_id, viewer_ctx_json, ops_json, now_ms as i64)
        }

        pub fn projection(&mut self, group_id: &str, viewer_ctx_json: &str) -> String {
            self.inner.projection(group_id, viewer_ctx_json)
        }

        pub fn entity_detail(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            entity_id: &str,
        ) -> String {
            self.inner
                .entity_detail(group_id, viewer_ctx_json, entity_id)
        }

        pub fn query_paths(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            request_json: &str,
        ) -> String {
            self.inner
                .query_paths(group_id, viewer_ctx_json, request_json)
        }

        pub fn query_neighborhood(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            request_json: &str,
        ) -> String {
            self.inner
                .query_neighborhood(group_id, viewer_ctx_json, request_json)
        }

        pub fn search(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            query_json: &str,
        ) -> String {
            self.inner.search(group_id, viewer_ctx_json, query_json)
        }

        pub fn validation_report(&self, group_id: &str, viewer_ctx_json: &str) -> String {
            self.inner.validation_report(group_id, viewer_ctx_json)
        }

        pub fn export_snapshot(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            options_json: &str,
        ) -> String {
            self.inner
                .export_snapshot(group_id, viewer_ctx_json, options_json)
        }

        // Read-only intake facade (ADR-005 D4; blueprint section 3). There
        // is deliberately NO approval-write export: the browser has no
        // durable store - all intake mutation lives in native
        // `cn intake apply`.

        pub fn intake_validate_record(&self, group_id: &str, record_json: &str) -> String {
            self.inner.intake_validate_record(group_id, record_json)
        }

        pub fn intake_dedup_check(&self, record_json: &str, existing_keys_json: &str) -> String {
            self.inner
                .intake_dedup_check(record_json, existing_keys_json)
        }

        pub fn intake_near_duplicates(
            &mut self,
            group_id: &str,
            viewer_ctx_json: &str,
            request_json: &str,
        ) -> String {
            self.inner
                .intake_near_duplicates(group_id, viewer_ctx_json, request_json)
        }

        pub fn viewer_roles(&self, group_id: &str, viewer_ctx_json: &str) -> String {
            self.inner.viewer_roles(group_id, viewer_ctx_json)
        }

        pub fn intake_stage_record(&self, payload_json: &str, staged_at_ms: f64) -> String {
            self.inner
                .intake_stage_record(payload_json, staged_at_ms as i64)
        }

        pub fn intake_build_decision(&self, request_json: &str) -> String {
            self.inner.intake_build_decision(request_json)
        }
    }
}
