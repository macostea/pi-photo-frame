/// Spawn a future on the tokio runtime
#[macro_export]
macro_rules! spawn_tokio {
    ($future:expr) => {
        $crate::RUNTIME.spawn($future)
    };
}

#[macro_export]
macro_rules! spawn {
    ($future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local($future);
    };
    ($priority:expr, $future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local_with_priority($priority, $future);
    };
}
