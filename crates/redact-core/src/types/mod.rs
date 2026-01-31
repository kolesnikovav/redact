// Copyright (c) 2026 Censgate LLC.
// Licensed under the Business Source License 1.1 (BUSL-1.1).
// See the LICENSE file in the project root for license details,
// including the Additional Use Grant, Change Date, and Change License.

pub mod entity;
pub mod result;

pub use entity::EntityType;
pub use result::{AnalysisMetadata, AnalysisResult, AnonymizedResult, RecognizerResult, Token};
