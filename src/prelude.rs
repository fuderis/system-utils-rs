#![allow(unused_imports, dead_code)]
pub use crate::error::*;

pub use std::result::Result as StdResult;
pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = StdResult<T, DynError>;

pub use atoman::*;
pub use macron::*;

pub use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

pub use serde::{Deserialize, Serialize};
