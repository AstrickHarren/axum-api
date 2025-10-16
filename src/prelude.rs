pub use {
    crate::extractors::*,
    aide::axum::{
        ApiRouter,
        routing::{
            delete_with, get_with, head_with, options_with, patch_with, post_with, put_with,
            trace_with,
        },
    },
};
